use std::{io, time::SystemTime};

use crate::screen_capture::Capture;

mod screen_capture;

pub fn main() {
    let mut pipewire_capture = screen_capture::pipewire::pipewire::PipeWireScreenCapture::new();
    pipewire_capture.start_capture();

    loop {
        let image = pipewire_capture.get_captured_image();
        if let Some(unwrapped_image) = image {
            println!("saving...");
            let filename = tempfile::NamedTempFile::with_suffix(".png").expect("Failed to create temp image file");
            unwrapped_image.save(&filename.path()).expect("failed to save image");
            
            // TO-DO: Add support of EasyOCR
            let recognized = tesseract::ocr(&filename.path().to_str().unwrap(), "jpn").unwrap();
            println!("recognized: {}", recognized);
            std::fs::remove_file(filename).expect("Failed to delete file");
        } else {
            //println!("Oops, no image");
        }
    }
}