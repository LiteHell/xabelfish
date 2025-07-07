use std::{sync::{mpsc, Arc, Mutex, RwLock}, thread::{self, JoinHandle}};

use crate::screen_capture::{pipewire::pipewire, Capture};

pub struct LugataEngine {
    capture: Option<Arc<Mutex<pipewire::PipeWireScreenCapture>>>,
    ocr_thread: Option<JoinHandle<()>>,
    stopper: Option<Arc<RwLock<bool>>>
}

impl LugataEngine {
    pub fn select_window(&mut self, sender: mpsc::Sender<String>) {
        if (self.capture.is_some()) {
            // Do some dropping here
        }

        let mut capture = pipewire::PipeWireScreenCapture::new();
        capture.start_capture();
        self.capture = Some(Arc::new(Mutex::new(capture)));
        self.init(sender);
    }
    fn init(&mut self, sender: mpsc::Sender<String>) {
        let mut capture_cloned = self.capture.as_ref().unwrap().clone();
        let thread = thread::spawn(move || {
            loop {
                let image_option = (*capture_cloned).lock().unwrap().get_captured_image();
                
                if let Some(image) = image_option {
                    let tempfile = tempfile::NamedTempFile::with_suffix(".png").unwrap();
                    image.save(&tempfile);
                    let ocr = tesseract::ocr(&tempfile.path().to_str().unwrap(), "jpn");
                    sender.send(ocr.unwrap());
                    std::fs::remove_file(tempfile.path());
                }
            }
        });
    }
}

pub static  mut ENGINE: LugataEngine = LugataEngine {
    capture: None,
    ocr_thread: None,
    stopper: None
};