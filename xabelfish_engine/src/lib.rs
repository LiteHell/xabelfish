mod max_sized_deque;
mod listener_pid_and_sock_path;
mod executable_paths;

use std::{
    path::Path,
    sync::{
        Arc, RwLock, atomic::{AtomicBool, Ordering}, mpsc::{self, Sender}
    },
    thread, time::Duration,
};

use xabelfish_config::{XabelFishEngineConfig, ocr::OcrType, translator::TranslatorType};
use xabelfish_socket_protocol::{
    cont_capture::ContinuousCaptureMessage,
    ocr::{OcrMessage, OcrRequestBody},
    translate::TranslateMessage,
};
use xabelfish_unix_socket::unix_socket_client::UnixSocketClient;

use crate::{executable_paths::{get_cont_capture, get_ocr, get_translator}, listener_pid_and_sock_path::ListenerPidAndSockPath, max_sized_deque::RoughlySizeConstraintDeque};

pub struct XabelFishEngine {
    started: bool,
    stopping: Arc<AtomicBool>,
    cont_capture_process: Arc<RwLock<Option<ListenerPidAndSockPath<()>>>>,
    ocr_process: Arc<RwLock<Option<ListenerPidAndSockPath<OcrType>>>>,
    translate_process: Arc<RwLock<Option<ListenerPidAndSockPath<TranslatorType>>>>,
    translation_tw: Sender<String>,
    image_stack: Arc<RoughlySizeConstraintDeque<ContinuousCaptureMessage>>,
    ocr_stack: Arc<RoughlySizeConstraintDeque<String>>,
}

impl XabelFishEngine {

    pub fn new(translation_tw: &mut mpsc::Sender<String>) -> Self {
        

        Self {
            started: false,
            stopping: Arc::new(AtomicBool::new(false)),
            cont_capture_process: Arc::new(RwLock::new(ListenerPidAndSockPath::none())),
            ocr_process: Arc::new(RwLock::new(ListenerPidAndSockPath::none())),
            translate_process: Arc::new(RwLock::new(ListenerPidAndSockPath::none())),
            image_stack: Arc::new(RoughlySizeConstraintDeque::new(10)),
            ocr_stack: Arc::new(RoughlySizeConstraintDeque::new(10)),
            translation_tw: translation_tw.clone(),
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

    fn start_heartbeat(&mut self) {
        let cont_capture_process = self.cont_capture_process.clone();
        let ocr_process = self.ocr_process.clone();
        let translate_process = self.translate_process.clone();
        let stopping = self.stopping.clone();
        
        thread::spawn(move || {
            loop {
                
                if stopping.load(Ordering::Relaxed) {
                    let mut cont_capture_process = cont_capture_process.write().unwrap();
                    let mut ocr_process = ocr_process.write().unwrap();
                    let mut translate_process = translate_process.write().unwrap();
                    if let Some(ocr_process) = ocr_process.as_mut() {
                        ocr_process.kill(true);
                    }
                    if let Some(translate_process) = translate_process.as_mut() {
                        translate_process.kill(true);
                    }
                    if let Some(cont_capture_process) = cont_capture_process.as_mut() {
                        cont_capture_process.kill(true);
                    }

                    break;
                } else {
                    let (latest_ocr_type, latest_translator_type) = { let config = XabelFishEngineConfig::get_config();
                        (config.ocr_type, config.translator_type)
                    };

                    let mut ocr_process = ocr_process.write().unwrap();
                    let mut translate_process = translate_process.write().unwrap();
                    if let Some(ocr_process) = ocr_process.as_mut() {
                        if ocr_process.extra() != latest_ocr_type {
                        ocr_process.change_process(get_ocr(), latest_ocr_type);
                        }
                    }
                    if let Some(translate_process) = translate_process.as_mut() {
                        if translate_process.extra() != latest_translator_type {
                        translate_process.change_process(get_ocr(), latest_translator_type);
                        }
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn start_translate(&mut self) {
        let exec_path = get_translator();

        let (mut translate_listener, process_info) =
            ListenerPidAndSockPath::create_with_process(exec_path, XabelFishEngineConfig::get_config().translator_type.clone());
        let translation_tw = self.translation_tw.clone();
        *self.translate_process.write().unwrap() = Some(process_info);

        let ocr_stack = self.ocr_stack.clone();
        let stopping = self.stopping.clone();

        thread::spawn(move || {
            'accept_loop: loop {
                if stopping.load(Ordering::Relaxed) {
                    return;
                }
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
                            config: XabelFishEngineConfig::get_config().get_translator_config_string(),
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
        let exec_path = get_ocr();

        let (mut ocr_listener, ocr_process_info) =
            ListenerPidAndSockPath::create_with_process(exec_path, XabelFishEngineConfig::get_config().ocr_type.clone());

        *self.ocr_process.write().unwrap() = Some(ocr_process_info);

        let image_stack = self.image_stack.clone();
        let ocr_stack = self.ocr_stack.clone();

        let stopping = self.stopping.clone();

        thread::spawn(move || {
            'accept_loop: loop {
                if stopping.load(Ordering::Relaxed) {
                    return;
                }
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
                            config: XabelFishEngineConfig::get_config().get_ocr_config_string(),
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
        let exec_path = get_cont_capture();
        let (mut cont_capture_clistener, cont_capture_info) =
            ListenerPidAndSockPath::create_with_process(exec_path, ());

        *self.cont_capture_process.write().unwrap() = Some(cont_capture_info);

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
                    _ => break
                }
            }
        });
    }
}

impl Drop for XabelFishEngine {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Relaxed)
    }
}
