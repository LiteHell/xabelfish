use std::time::SystemTime;

use crate::screen_capture::Capture;

mod screen_capture;

pub fn main() {
    let mut pipewire_capture = screen_capture::pipewire::pipewire::PipeWireScreenCapture::new();
    pipewire_capture.start_capture();

    loop {
        let image = pipewire_capture.get_captured_image();
        if let Some(unwrapped_image) = image {
            println!("saving...");
            let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
            unwrapped_image.save(format!("./test{}.png", timestamp)).expect("failed to save image");
        } else {
            //println!("Oops, no image");
        }
    }
}