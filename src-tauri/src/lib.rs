// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use font_kit::source::SystemSource;
use tauri::{AppHandle, Emitter, Manager};

use crate::config::{get_xabelfish_config, set_xabelfish_config, XabelFishConfig};

mod config;
mod engine;
mod screen_capture;
mod translate;

#[tauri::command]
fn get_config() -> XabelFishConfig {
    get_xabelfish_config()
}

#[tauri::command]
fn set_config(app: AppHandle, config: XabelFishConfig) {
    set_xabelfish_config(config);
    app.emit("config_changed", get_xabelfish_config())
        .expect("Failed to emit config_changed event");
}

#[tauri::command]
fn set_window(app: AppHandle) {
    let cloned_app = app.clone();
    let engine_state = cloned_app.state::<Arc<Mutex<engine::XabelFishEngine>>>();
    {
        println!("Trying to acquire lock");
        let mut engine = engine_state.lock().unwrap();
        println!("Acquired lock");
        engine.select_window();
    }
}

#[tauri::command]
fn open_config_window(app: AppHandle) {
    let config_window = app.get_webview_window("config").unwrap();

    config_window.show();
    config_window.set_focus();
}

#[tauri::command]
fn get_fonts() -> Vec<String> {
    let source = SystemSource::new();
    let families = source.all_families().expect("Failed to get font families");

    return families;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let engine = Mutex::new(engine::XabelFishEngine::new());
            let arc = Arc::new(engine);
            app.manage(arc.clone());

            let handle = app.handle().clone();
            {
                let engine = arc.clone();

                thread::spawn(move || {
                    let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    tokio_runtime.block_on(async {
                        loop {
                            {
                                let mut lock = engine.lock().unwrap();

                                if let Some(i) = lock.translate_ocr().await {
                                    let _ = handle.emit("translated", i);
                                } else {
                                }
                            }

                            // Prevent race condition with select window menu
                            thread::sleep(Duration::from_millis(2));
                        }
                    });
                });
            }

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_window,
            get_config,
            set_config,
            open_config_window,
            get_fonts
        ])
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                if window.label() == "config" {
                    api.prevent_close();
                    window.hide().unwrap();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
