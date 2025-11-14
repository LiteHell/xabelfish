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
    cont_capture::ContinuousCaptureMessage,
    ocr::{OcrMessage, OcrRequestBody},
    translate::TranslateMessage,
};
use xabelfish_unix_socket::{
    unix_socket_client::UnixSocketClient, unix_socket_server::UnixSocketServer,
};

use crate::max_sized_deque::RoughlySizeConstraintDeque;

struct ListenerPidAndSockPath<T> {
    pub process: Option<Pid>,
    pub socket_path: Option<String>,
    pub process_type: Option<T>,
}

impl<T> ListenerPidAndSockPath<T> {
    pub fn none() -> Self {
        Self {
            process: None,
            process_type: None,
            socket_path: None,
        }
    }

    pub fn create_process(exec_path: PathBuf, process_type: T) -> (UnixSocketServer, Self) {
        let (mut translate_listener, socket_path) =
            UnixSocketServer::create().expect("Failed to create control socket for translate");

        let process = Command::new(exec_path.as_os_str())
            .arg("--socket-path")
            .arg(socket_path.clone())
            .spawn()
            .expect("Failed to run translate");

        (
            translate_listener,
            Self {
                process: Some(Pid::from_child(&process)),
                process_type: Some(process_type),
                socket_path: Some(socket_path),
            },
        )
    }

    pub fn kill(&mut self, sigkill: bool) -> rustix::io::Result<()> {
        if let Some(pid) = self.process {
            kill_process(pid, if sigkill { Signal::KILL } else { Signal::TERM })?;
        }

        Ok(())
    }
}

pub struct XabelFishEngine {
    started: bool,
    stopping: Arc<AtomicBool>,
    executable_base_dir: PathBuf,
    cont_capture_process: ListenerPidAndSockPath<()>,
    ocr_process: ListenerPidAndSockPath<String>,
    translate_process: ListenerPidAndSockPath<String>,
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
            cont_capture_process: ListenerPidAndSockPath::none(),
            ocr_process: ListenerPidAndSockPath::none(),
            translate_process: ListenerPidAndSockPath::none(),
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
        let exec_path = self.get_translate_executable_path();

        let (mut translate_listener, process_info) =
            ListenerPidAndSockPath::create_process(exec_path, "deepL".to_string());
        let translation_tw = self.translation_tw.clone();
        self.translate_process = process_info;

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
                        .send(&TranslateMessage::TranslationRequest(xabelfish_socket_protocol::translate::TranslationRequestBody  {
                            config: String::new(),
                            texts: vec![ocr_text],
                            dst: String::from("ko"),
                            src:  xabelfish_socket_protocol::translate::TranslateSourceLanguage::Automatic
                        }))
                        .expect("Failed to send translate req command");

                    let response: TranslateMessage =
                        cont_capture.recv().expect("Failed to receive response");

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
            ListenerPidAndSockPath::create_process(exec_path, "tesseract".to_string());

        self.ocr_process = ocr_process_info;

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
                        .send(&OcrMessage::OcrRequest(OcrRequestBody {
                            config: String::new(),
                            image_bytes: image.extra_bytes,
                            image_type: image.extra_str,
                        }))
                        .expect("Failed to send ocr req command");

                    let response: OcrMessage =
                        cont_capture.recv().expect("Failed to receive response");

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
            ListenerPidAndSockPath::create_process(exec_path, ());

        self.cont_capture_process = cont_capture_info;

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
        let _ = self.cont_capture_process.kill(true);
        let _ = self.ocr_process.kill(true);
        let _ = self.translate_process.kill(true);
    }
}
