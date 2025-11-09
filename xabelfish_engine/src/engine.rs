use std::{
    env::current_exe,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{Arc, mpsc},
    thread,
};

use xabelfish_socket_protocol::{cont_capture::ContinuousCaptureMessage, ocr::OcrMessage};
use xabelfish_unix_socket::{
    unix_socket_client::UnixSocketClient, unix_socket_server::UnixSocketServer,
};

use crate::max_sized_deque::RoughlySizeConstraintDeque;

pub enum XabelFishOcrType {
    Tesseract,
}

pub struct XabelFishEngine {
    started: bool,
    executable_base_dir: PathBuf,
    cont_capture_process: Option<Child>,
    ocr_process: Option<Child>,
    ocr_type: XabelFishOcrType,
    image_stack: Arc<RoughlySizeConstraintDeque<ContinuousCaptureMessage>>,
}

impl XabelFishEngine {
    fn get_executable(&self, name: PathBuf) -> PathBuf {
        self.executable_base_dir.join(name)
    }

    pub fn new() -> Self {
        let executable_base_dir = {
            let mut exe_path = current_exe().unwrap();
            exe_path.pop();
            exe_path
        };

        Self {
            started: false,
            executable_base_dir,
            cont_capture_process: None,
            ocr_process: None,
            ocr_type: XabelFishOcrType::Tesseract,
            image_stack: Arc::new(RoughlySizeConstraintDeque::new(10)),
        }
    }

    pub fn start_nonblocking(&mut self, translation_recevier: &mut mpsc::Sender<String>) {
        if self.started {
            panic!("xabelFish engine already started!");
        } else {
            self.started = true;
        }

        self.start_capture();
        self.start_ocr();
    }

    fn start_ocr(&mut self) {
        let exec_path = Arc::new(
            self.get_executable(PathBuf::from(match self.ocr_type {
                XabelFishOcrType::Tesseract => "xabelfish_ocr_tesseract",
            }))
            .as_mut_os_string()
            .clone(),
        );

        let (mut ocr_listener, ocr_sock_path) =
            UnixSocketServer::create().expect("Failed to create control socket for ocr");

        let ocr_process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(ocr_sock_path.clone())
            .spawn()
            .expect("Failed to run continous capture");

        self.ocr_process = Some(ocr_process);

        let image_stack = self.image_stack.clone();

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

                    let ocr_result = response.text;
                    
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

        self.cont_capture_process = Some(cont_capture_process);

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
                    xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitFail => {
                        
                    }
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
        if let Some(cont_capture_process) = self.cont_capture_process.as_mut() {
            cont_capture_process.kill();
        }
    }
}
