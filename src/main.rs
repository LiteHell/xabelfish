use std::{io, time::SystemTime};

use crate::{screen_capture::Capture, translate::{deepl::DeepLTranslator, Translator}};

mod screen_capture;
mod translate;

#[tokio::main]
pub async fn main() {
    let mut pipewire_capture = screen_capture::pipewire::pipewire::PipeWireScreenCapture::new();
    pipewire_capture.start_capture();

    let translator = DeepLTranslator::new();
    let mut previous_recognition = String::new();
    let mut previous_translation = String::new();

    loop {
        let image = pipewire_capture.get_captured_image();
        if let Some(unwrapped_image) = image {
            println!("saving...");
            let filename = tempfile::NamedTempFile::with_suffix(".png").expect("Failed to create temp image file");
            unwrapped_image.save(&filename.path()).expect("failed to save image");
            
            // TO-DO: Add support of EasyOCR
            let recognized = tesseract::ocr(&filename.path().to_str().unwrap(), "jpn").unwrap();
            println!("recognized: {}", &recognized);
            std::fs::remove_file(filename).expect("Failed to delete file");

            if (previous_recognition == recognized) {
                println!("Reusing translation: {}", previous_translation);
                continue;
            }

            let translated = translator.translate(&recognized, None::<String>, "ko").await;
            println!("translated: {}", translated);

            previous_recognition = recognized.clone();
            previous_translation = String::from(translated);
        } else {
            //println!("Oops, no image");
        }
    }
}