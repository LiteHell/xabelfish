use std::{
    sync::{mpsc, Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use crate::{
    screen_capture::{pipewire::pipewire, Capture},
    translate::{deepl, Translator},
};

pub struct XabelFishEngine {
    capture: Option<Arc<Mutex<pipewire::PipeWireScreenCapture>>>,
    translation_cache: Option<(String, String)>,
    ocr_thread: Option<JoinHandle<()>>,
    stopper: Option<Arc<RwLock<bool>>>,
}

impl XabelFishEngine {
    pub fn new() -> Self {
        XabelFishEngine {
            capture: None,
            ocr_thread: None,
            stopper: None,
            translation_cache: None,
        }
    }

    pub fn select_window(&mut self) {
        let mut capture = pipewire::PipeWireScreenCapture::new();
        let start_capture_result = capture.start_capture();

        if start_capture_result.is_ok() {
            self.capture = Some(Arc::new(Mutex::new(capture)));
        }
    }

    pub async fn translate_ocr(&mut self) -> Option<String> {
        let translator = deepl::DeepLTranslator::new();
        let ocr = self.process_ocr();

        if let Some(ocr) = ocr {
            if let Some(cache) = &self.translation_cache {
                if cache.0.eq(&ocr) {
                    return Some(cache.1.clone());
                }
            }
            let translated = translator.translate(&ocr, Some("ja"), "ko").await;
            self.translation_cache = Some((ocr.clone(), translated.clone()));
            Some(translated.to_string())
        } else {
            None
        }
    }

    pub fn process_ocr(&mut self) -> Option<String> {
        if (self.capture.is_none()) {
            return None;
        }

        let capture_cloned = self.capture.as_ref().unwrap().clone();
        let image_option = (*capture_cloned).lock().unwrap().get_captured_image();

        if let Some(image) = image_option {
            let tempfile = tempfile::NamedTempFile::with_suffix(".png").unwrap();
            image.save(&tempfile).expect("Failed to save image");
            let ocr = tesseract::ocr(&tempfile.path().to_str().unwrap(), "jpn");
            let processed = (ocr.unwrap());
            let _ = std::fs::remove_file(tempfile.path());

            Some(processed)
        } else {
            None
        }
    }
}
