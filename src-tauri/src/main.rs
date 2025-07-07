// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod screen_capture;
mod translate;

fn main() {
    lugata_lib::run()
}
