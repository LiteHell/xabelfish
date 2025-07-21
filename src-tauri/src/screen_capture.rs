pub mod pipewire;

pub trait Capture {
    fn new() -> Self;
    fn start_capture(&mut self) -> Result<(), ()>;
    fn get_captured_image(&mut self) -> Option<image::RgbaImage>;
}
