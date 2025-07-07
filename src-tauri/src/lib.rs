// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use tauri::{AppHandle, Emitter, Manager};

mod engine;
mod screen_capture;
mod translate;

#[tauri::command]
fn set_window(app: AppHandle) {
    let cloned_app = app.clone();
    let engine_state = cloned_app.state::<Arc<Mutex<engine::LugataEngine>>>();
    {
        println!("Trying to acquire lock");
        let mut engine = engine_state.lock().unwrap();
        println!("Acquired lock");
        engine.select_window();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let engine = Mutex::new(engine::LugataEngine::new());
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

                                thread::sleep(Duration::from_millis(2));
                            }
                        }
                    });

                    // Prevent race condition with select window menu
                });
            }

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_window])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
