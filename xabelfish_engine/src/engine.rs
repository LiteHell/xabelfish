use std::{
    env::current_exe,
    fs::remove_file,
    os::unix::net::UnixListener,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{Arc, mpsc},
    thread,
};

use xabelfish_sized_lockfree_stack::RoughlySizedLockFreeStack;
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessage;
use xabelfish_unix_socket::{
    unix_socket_client::UnixSocketClient, unix_socket_server::UnixSocketServer,
};

pub enum XabelFishOcrType {
    Tesseract(UnixSocketServer),
}

pub struct XabelFishEngine {
    started: bool,
    executable_base_dir: PathBuf,
    cont_capture_process: Option<Child>,
    ocr_control_listener: Option<XabelFishOcrType>,
    ocr_control_sock_path: String,
    image_stack: Arc<RoughlySizedLockFreeStack<ContinuousCaptureMessage>>,
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
            ocr_control_listener: None,
            ocr_control_sock_path: String::new(),
            image_stack: Arc::new(RoughlySizedLockFreeStack::new(10)),
        }
    }

    pub fn start_nonblocking(&mut self, translation_recevier: &mut mpsc::Sender<String>) {
        if self.started {
            panic!("xabelFish engine already started!");
        } else {
            self.started = true;
        }

        self.start_capture();
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

        let mut image_stack = self.image_stack.clone();

        let thread = thread::spawn(move || {
            loop {
                println!("Accepting...");
                let mut cont_capture = cont_capture_control_listener
                    .accept()
                    .expect("Failed to accept continous capture connection");

                println!("Accepted cont capture connection");
                println!("Sending capture init command...");
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
                        println!("Oops, failed to init capture!");
                    }
                    xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType::CaptureInitSuccess => {
                        println!("Connecting data socket");

                        let mut data_client = UnixSocketClient::connect(&Path::new(&response.extra_str)).expect("Failed to connect data socket");
                        loop {
                            let data: ContinuousCaptureMessage = data_client.recv().expect("Failed to get data");

                            println!("Got message: {:#?} type", data.message_type);
                            match image_stack.push(data) {
                                xabelfish_sized_lockfree_stack::RoughlySizedLockFreeStackPushResult::Done => println!("Pushed message"),
                                xabelfish_sized_lockfree_stack::RoughlySizedLockFreeStackPushResult::Full => {
                                    println!("stack full");
                                }
                            }
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
