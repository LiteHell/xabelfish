// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::{sync::mpsc, thread};

use tauri::{AppHandle, Emitter};

mod screen_capture;
mod translate;
mod engine;

#[tauri::command]
fn set_window(app: AppHandle) {
    let (sender, receiver) = mpsc::channel();
    unsafe {
        engine::ENGINE.select_window(sender);
    }

    thread::spawn(move || {
        loop {
            if let Ok(i) = receiver.recv() {
                app.emit("translated", i).unwrap();
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
