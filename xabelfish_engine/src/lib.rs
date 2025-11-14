mod max_sized_deque;

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
use xabelfish_socket_protocol::{
    cont_capture::ContinuousCaptureMessage, ocr::OcrMessage, translate::TranslateMessage,
};
use xabelfish_unix_socket::{
    unix_socket_client::UnixSocketClient, unix_socket_server::UnixSocketServer,
};

use crate::max_sized_deque::RoughlySizeConstraintDeque;

pub struct XabelFishEngine {
    started: bool,
    stopping: Arc<AtomicBool>,
    executable_base_dir: PathBuf,
    cont_capture_process: Option<Pid>,
    ocr_process: Option<Pid>,
    translate_process: Option<Pid>,
    cont_capture_socket_path: Option<String>,
    ocr_socket_path: Option<String>,
    translate_socket_path: Option<String>,
    ocr_type: String,
    translation_tw: Sender<String>,
    image_stack: Arc<RoughlySizeConstraintDeque<ContinuousCaptureMessage>>,
    ocr_stack: Arc<RoughlySizeConstraintDeque<String>>,
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
            cont_capture_process: None,
            ocr_process: None,
            translate_process: None,
            cont_capture_socket_path: None,
            ocr_socket_path: None,
            translate_socket_path: None,
            ocr_type: String::new(),
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

    fn get_ocr_executable_path(&self) -> PathBuf {
        return self.get_executable(PathBuf::from("xabelfish_ocr_tesseract"));
    }

    fn get_translate_executable_path(&self) -> PathBuf {
        return self.get_executable(PathBuf::from("xabelfish_translate_deepl"));
    }

    fn start_translate(&mut self) {
        let exec_path = Arc::new(
            self.get_translate_executable_path()
                .as_mut_os_string()
                .clone(),
        );

        let (mut translate_listener, translate_sock_path) =
            UnixSocketServer::create().expect("Failed to create control socket for translate");
        let translation_tw = self.translation_tw.clone();
        self.translate_socket_path = Some(translate_sock_path.clone());

        let translate_process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(translate_sock_path.clone())
            .spawn()
            .expect("Failed to run translate");

        self.translate_process = Some(Pid::from_child(&translate_process));

        let image_stack = self.image_stack.clone();
        let ocr_stack = self.ocr_stack.clone();

        thread::spawn(move || {
            loop {
                let mut cont_capture = translate_listener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                loop {
                    let ocr_text = match ocr_stack.last() {
                        Some(text) => text,
                        None => continue,
                    };

                    cont_capture
                        .send(&TranslateMessage {
                            message_type:
                                xabelfish_socket_protocol::translate::TranslateMessageType::TranslateRequest,
                            config: String::new(),
                            data_bool: false,
                            data_text: ocr_text,
                            dst: String::from("ko"),
                            src:  xabelfish_socket_protocol::translate::TranslateSourceLanguage::Automatic
                        })
                        .expect("Failed to send translate req command");

                    let response: TranslateMessage =
                        cont_capture.recv().expect("Failed to receive response");

                    translation_tw.send(response.data_text);
                }
            }
        });
    }

    fn start_ocr(&mut self) {
        let exec_path = Arc::new(self.get_ocr_executable_path().as_mut_os_string().clone());

        let (mut ocr_listener, ocr_sock_path) =
            UnixSocketServer::create().expect("Failed to create control socket for ocr");

        let ocr_process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(ocr_sock_path.clone())
            .spawn()
            .expect("Failed to run continous capture");

        self.ocr_process = Some(Pid::from_child(&ocr_process));
        self.ocr_socket_path = Some(ocr_sock_path.clone());

        let image_stack = self.image_stack.clone();
        let ocr_stack = self.ocr_stack.clone();

        thread::spawn(move || {
            loop {
                let mut cont_capture = ocr_listener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                loop {
                    let image = match image_stack.last() {
                        Some(image) => image,
                        None => continue,
                    };

                    cont_capture
                        .send(&OcrMessage {
                            message_type:
                                xabelfish_socket_protocol::ocr::OcrMessageType::OcrRequest,
                            config: String::new(),
                            text: String::new(),
                            image_bytes: image.extra_bytes,
                            image_type: image.extra_str,
                        })
                        .expect("Failed to send ocr req command");

                    let response: OcrMessage =
                        cont_capture.recv().expect("Failed to receive response");

                    ocr_stack.push(response.text);
                }
            }
        });
    }

    fn start_capture(&mut self) {
        let exec_path = Arc::new(
            self.get_executable(PathBuf::from("xabelfish_cont_capture"))
                .as_mut_os_string()
                .clone(),
        );
        let (mut cont_capture_control_listener, cont_capture_control_sock_path) =
            UnixSocketServer::create().expect("Failed to create control socket for capture");

        let cont_capture_process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(cont_capture_control_sock_path.clone())
            .spawn()
            .expect("Failed to run continous capture");

        self.cont_capture_process = Some(Pid::from_child(&cont_capture_process));
        self.cont_capture_socket_path = Some(cont_capture_control_sock_path.clone());

        let image_stack = self.image_stack.clone();

        thread::spawn(move || {
            loop {
                let mut cont_capture = cont_capture_control_listener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                cont_capture.send(&ContinuousCaptureMessage {
                    message_type:
                        xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::StartCapture,
                    extra_bytes: vec![],
                    extra_str: String::new(),
                }).expect("Failed to send init command");

                let response: ContinuousCaptureMessage =
                    cont_capture.recv().expect("Failed to receive response");

                match response.message_type {
                    xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitFail => {}
                    xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitSuccess => {
                        let mut data_client = UnixSocketClient::connect(&Path::new(&response.extra_str)).expect("Failed to connect data socket");
                        loop {
                            let data: ContinuousCaptureMessage = data_client.recv().expect("Failed to get data");
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
        if let Some(cont_capture_process) = self.cont_capture_process.as_mut() {
            kill_process(*cont_capture_process, Signal::TERM);
        }
        if let Some(ocr_process) = self.ocr_process.as_mut() {
            kill_process(*ocr_process, Signal::TERM);
        }
        if let Some(translate_process) = self.translate_process.as_mut() {
            kill_process(*translate_process, Signal::TERM);
        }
    }
}
