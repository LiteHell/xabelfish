mod max_sized_deque;
mod listener_pid_and_sock_path;

use std::{
    env::current_exe,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread,
};

use rustix::process::{Pid, Signal, kill_process};
use xabelfish_config::{XabelFishEngineConfig, ocr::OcrType, translator::TranslatorType};
use xabelfish_socket_protocol::{
    cont_capture::ContinuousCaptureMessage,
    ocr::{OcrMessage, OcrRequestBody},
    translate::TranslateMessage,
};
use xabelfish_unix_socket::{
    unix_socket_client::UnixSocketClient, unix_socket_server::UnixSocketServer,
};

use crate::{listener_pid_and_sock_path::ListenerPidAndSockPath, max_sized_deque::RoughlySizeConstraintDeque};

pub struct XabelFishEngine {
    started: bool,
    stopping: Arc<AtomicBool>,
    executable_base_dir: PathBuf,
    cont_capture_process: Option<ListenerPidAndSockPath<()>>,
    ocr_process: Option<ListenerPidAndSockPath<OcrType>>,
    translate_process: Option<ListenerPidAndSockPath<TranslatorType>>,
    translation_tw: Sender<String>,
    image_stack: Arc<RoughlySizeConstraintDeque<ContinuousCaptureMessage>>,
    ocr_stack: Arc<RoughlySizeConstraintDeque<String>>,
    config: xabelfish_config::XabelFishEngineConfig
}

impl XabelFishEngine {
    fn get_executable(&self, name: PathBuf) -> PathBuf {
        self.executable_base_dir.join(name)
    }

    pub fn new(translation_tw: &mut mpsc::Sender<String>) -> Self {
        let executable_base_dir = {
            let mut exe_path = current_exe().unwrap();
            exe_path.pop();
            exe_path
        };

        Self {
            started: false,
            stopping: Arc::new(AtomicBool::new(false)),
            executable_base_dir,
            cont_capture_process: ListenerPidAndSockPath::none(),
            ocr_process: ListenerPidAndSockPath::none(),
            translate_process: ListenerPidAndSockPath::none(),
            image_stack: Arc::new(RoughlySizeConstraintDeque::new(10)),
            ocr_stack: Arc::new(RoughlySizeConstraintDeque::new(10)),
            translation_tw: translation_tw.clone(),
            config: XabelFishEngineConfig::default()
        }
    }

    pub fn config(&self) -> XabelFishEngineConfig {
        self.config.clone()
    }

    pub fn set_config(&mut self, new_config: XabelFishEngineConfig) {
        self.config = new_config;
        let ocr_path = self.get_ocr_executable_path().clone();
        let translator_path = self.get_translate_executable_path().clone();

        if let Some(ocr_process) = &mut self.ocr_process {
            let current_extra = ocr_process.extra().clone();
            if current_extra != self.config.ocr_type {
                ocr_process.change_process(ocr_path, self.config.ocr_type.clone());
            }
        }

        if let Some(translator_process) = &mut self.translate_process {
            let current_extra = translator_process.extra().clone();
            if current_extra != self.config.translator_type {
                translator_process.change_process(translator_path, self.config.translator_type.clone());
            }
        }
    }

    pub fn start_nonblocking(&mut self) {
        if self.started {
            panic!("xabelFish engine already started!");
        } else {
            self.started = true;
        }

        self.start_capture();
        self.start_ocr();
        self.start_translate();
    }

    fn get_ocr_executable_path(&self) -> PathBuf {
        return self.get_executable(PathBuf::from("xabelfish_ocr_tesseract"));
    }

    fn get_translate_executable_path(&self) -> PathBuf {
        return self.get_executable(PathBuf::from("xabelfish_translate_deepl"));
    }

    fn start_translate(&mut self) {
        let exec_path = self.get_translate_executable_path();

        let (mut translate_listener, process_info) =
            ListenerPidAndSockPath::create_with_process(exec_path, self.config.translator_type.clone());
        let translation_tw = self.translation_tw.clone();
        self.translate_process = Some(process_info);

        let ocr_stack = self.ocr_stack.clone();

        thread::spawn(move || {
            'accept_loop: loop {
                let mut translate_socket = translate_listener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                loop {
                    let ocr_text = match ocr_stack.last() {
                        Some(text) => text,
                        None => continue,
                    };

                    translate_socket
                        .send(&TranslateMessage::TranslationRequest(xabelfish_socket_protocol::translate::TranslationRequestBody  {
                            config: String::new(),
                            texts: vec![ocr_text],
                            dst: String::from("ko"),
                            src:  xabelfish_socket_protocol::translate::TranslateSourceLanguage::Automatic
                        }))
                        .expect("Failed to send translate req command");

                    let response: TranslateMessage = {
                        let response = translate_socket.recv().expect("Failed to receive response");

                        if let Some(response) = response {
                            response
                        } else if translate_socket.is_closed() {
                            continue 'accept_loop;
                        } else {
                            panic!("Response receive feailure");
                        }
                    };

                    match response {
                        TranslateMessage::TranslationResponse(strings) => {
                            translation_tw.send(strings[0].clone());
                        }
                        _ => continue,
                    }
                }
            }
        });
    }

    fn start_ocr(&mut self) {
        let exec_path = self.get_ocr_executable_path();

        let (mut ocr_listener, ocr_process_info) =
            ListenerPidAndSockPath::create_with_process(exec_path, self.config.ocr_type.clone());

        self.ocr_process = Some(ocr_process_info);

        let image_stack = self.image_stack.clone();
        let ocr_stack = self.ocr_stack.clone();

        thread::spawn(move || {
            'accept_loop: loop {
                let mut ocr_socket = ocr_listener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                loop {
                    let image = match image_stack.last() {
                        Some(image) => image,
                        None => continue,
                    };

                    ocr_socket
                        .send(&OcrMessage::OcrRequest(OcrRequestBody {
                            config: String::new(),
                            image_bytes: image.extra_bytes,
                            image_type: image.extra_str,
                        }))
                        .expect("Failed to send ocr req command");

                    let response: OcrMessage = {
                        let response = ocr_socket.recv().expect("Failed to receive response");

                        if let Some(response) = response {
                            response
                        } else if ocr_socket.is_closed() {
                            continue 'accept_loop;
                        } else {
                            panic!("Response receive feailure");
                        }
                    };

                    match response {
                        OcrMessage::OcrTextResponseBody(text) => {
                            ocr_stack.push(text);
                        }
                        _ => todo!("not supported yet..."),
                    }
                }
            }
        });
    }

    fn start_capture(&mut self) {
        let exec_path = self.get_executable(PathBuf::from("xabelfish_cont_capture"));
        let (mut cont_capture_clistener, cont_capture_info) =
            ListenerPidAndSockPath::create_with_process(exec_path, ());

        self.cont_capture_process = Some(cont_capture_info);

        let image_stack = self.image_stack.clone();

        thread::spawn(move || {
            loop {
                let mut cont_capture = cont_capture_clistener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                cont_capture.send(&ContinuousCaptureMessage {
                    message_type:
                        xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::StartCapture,
                    extra_bytes: vec![],
                    extra_str: String::new(),
                }).expect("Failed to send init command");

                let response: ContinuousCaptureMessage = {
                    let response = cont_capture.recv().expect("Failed to receive response");

                    if let Some(response) = response {
                        response
                    } else {
                        continue;
                    }
                };

                match response.message_type {
                    xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitFail => {}
                    xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitSuccess => {
                        let mut data_client = UnixSocketClient::connect(&Path::new(&response.extra_str)).expect("Failed to connect data socket");
                        loop {
                            let data: ContinuousCaptureMessage = {
                                let response = data_client.recv().expect("Failed to receive response");

                                if let Some(response) = response {
                                    response
                                } else {
                                    continue;
                                }
                            };
                            
                            image_stack.push(data);
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}

impl Drop for XabelFishEngine {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Relaxed);
        if let Some(process) = &mut self.cont_capture_process {
            process.kill(true);
        }
        if let Some(process) = &mut self.ocr_process {
            process.kill(true);
        }
        if let Some(process) = &mut self.translate_process {
            process.kill(true);
        }
    }
}
