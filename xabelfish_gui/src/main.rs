use std::{
    rc::Rc,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};

use slint::{ModelRc, SharedString, VecModel};
use xabelfish_engine_lib::{XabelFishEngine, XabelFishTranslation};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    prepare_xabelfish(&main_window);

    main_window.run()?;

    std::process::exit(0);
}

fn calc_good_font_size(text_len: usize, width: u32, height: u32) -> f32 {
    let max_font_size = std::cmp::min(width, height);

    // really stupid calc...
    // assuming a character = square (fontsize * fontsize)
    for i in 0..max_font_size {
        let i = max_font_size - i;
        let max_chars_per_line = width as f32 / i as f32;
        let max_lines = height as f32 / i as f32;

        println!("{} * {} >= {} if font-size = {}?", max_chars_per_line, max_lines, text_len, i);

        if max_chars_per_line * max_lines >= text_len as f32 {
            return i as f32;
        }
    }

    return 8.0; // fallback
}

fn prepare_xabelfish(window: &MainWindow) {
    let engine: Arc<Mutex<RwLock<Option<XabelFishEngine>>>> =
        Arc::new(Mutex::new(RwLock::new(None)));
    let translation_recv: Arc<Mutex<RwLock<Option<Receiver<XabelFishTranslation>>>>> =
        Arc::new(Mutex::new(RwLock::new(None)));
    {
        let engine = engine.clone();
        let translation_recv = translation_recv.clone();
        window.on_start_translation(move || {
            let engine_lock = engine.lock().unwrap();
            let mut engine_rwlock = engine_lock.write().unwrap();

            let (mut translation_tw, translation_rw) = mpsc::channel::<XabelFishTranslation>();
            let engine = XabelFishEngine::new(&mut translation_tw);
            *engine_rwlock = Some(engine);

            let translation_recv_lock = translation_recv.lock().unwrap();
            let mut translation_recv_rwlock = translation_recv_lock.write().unwrap();
            *translation_recv_rwlock = Some(translation_rw);

            engine_rwlock.as_mut().unwrap().start_nonblocking();
        });
    }
    {
        let engine = engine.clone();
        window.on_stop_translation(move || {
            let engine_lock = engine.lock().unwrap();
            let mut engine_rwlock = engine_lock.write().unwrap();

            *engine_rwlock = None;
        });
    }

    let window_weak: slint::Weak<MainWindow> = window.as_weak();
    {
        let engine = engine.clone();
        let translation_recv = translation_recv.clone();
        let window_weak = window_weak.clone();

        thread::spawn(move || {
            loop {
                let engine = engine.clone();
                {
                    let translation_recv = translation_recv.lock().unwrap();
                    let mut translation_recv = translation_recv.write().unwrap();

                    if translation_recv.is_some() {
                        let translation = translation_recv
                            .as_mut()
                            .unwrap()
                            .recv_timeout(Duration::from_millis(10));
                        if let Ok(translation) = translation {
                            let window_handle = window_weak.clone();

                            let _ = slint::invoke_from_event_loop(move || {
                                match translation {
                                    XabelFishTranslation::Positioned(translations) => {
                                        let back_image = {
                                            let engine_lock = engine.lock().unwrap();
                                            let engine_rwlock = engine_lock.read().unwrap();
                                            let image = loop {
                                                let image = (*engine_rwlock)
                                                    .as_ref()
                                                    .unwrap()
                                                    .get_uncropped_image();
                                                if let Some(image) = image {
                                                    break image;
                                                } else {
                                                    continue;
                                                }
                                            };

                                            image
                                        };

                                        let rgb8_image = back_image.to_rgb8();
                                        let image_width = back_image.width();
                                        let image_height = back_image.height();
                                        let image_buffer =
                                            slint::SharedPixelBuffer::clone_from_slice(
                                                &rgb8_image,
                                                back_image.width(),
                                                back_image.height(),
                                            );

                                        {
                                            let handle = window_handle.unwrap();
                                            handle.set_is_positioned_translation(true);
                                            handle.set_game_image(slint::Image::from_rgb8(
                                                image_buffer,
                                            ));
                                            handle.set_game_image_width(image_width as f32);
                                            handle.set_game_image_height(image_height as f32);
                                            handle.set_positioned_translations(ModelRc::from(
                                                Rc::new(VecModel::from(
                                                    translations
                                                        .into_iter()
                                                        .map(|i| XabelFishUiPositionedText {
                                                            height: i.height as f32,
                                                            text: SharedString::from(i.text.clone()),
                                                            width: i.width as f32,
                                                            x: i.x as f32,
                                                            y: i.y as f32,
                                                            font_size: calc_good_font_size(i.text.clone().len(), i.width, i.height)
                                                        })
                                                        .collect::<Vec<XabelFishUiPositionedText>>()
                                                )),
                                            ));
                                        }
                                    }
                                    XabelFishTranslation::String(string) => {
                                        let handle = window_handle.unwrap();
                                        handle.set_is_positioned_translation(false);
                                        handle.set_translation(string.into());
                                        handle.set_positioned_translations(ModelRc::default());
                                    }
                                };
                            });
                        }
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        });
    }
}
