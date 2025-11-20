use std::{thread, time::{Duration, Instant}};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    let main_window_weak = main_window.as_weak();
    
    {
        std::thread::spawn(move || {
            
            loop {
                let main_window_handle = main_window_weak.clone();
                let formatted_string = format!("Hello from thread! {:#?}", Instant::now());
                slint::invoke_from_event_loop(move || {
                    main_window_handle.unwrap().set_translation(formatted_string.into());
                }).expect("Fuck!");

                // gui will freeze if there's no delay...
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    main_window.run()
}