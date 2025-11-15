use std::sync::mpsc;

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use xabelfish_engine::XabelFishEngine;

fn main() {
    let (mut tx, rx) = mpsc::channel();

    let config = xabelfish_config::XabelFishEngineConfig::get_config();
    config.save();

    let mut engine = XabelFishEngine::new(&mut tx);
    engine.start_nonblocking();

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let header_color_spec = {
        let mut specs = ColorSpec::new();
        specs.set_fg(Some(Color::Black)).set_bg(Some(Color::White));
        specs
    };
    let body_color_sepc = {
        let mut specs = ColorSpec::new();
        specs.set_fg(None).set_bg(None);
        specs
    };
    for i in rx.iter() {
        let _ = clearscreen::clear();

        stdout.set_color(&header_color_spec);
        writeln!(&mut stdout, "Translation");

        stdout.set_color(&body_color_sepc);
        match i {
            xabelfish_engine::XabelFishTranslation::Positioned(positioned_translations) => {
                for translation in positioned_translations {
                    writeln!(
                        &mut stdout,
                        "position: coord={:#?}, w={}, h={}, x={}, y={}",
                        translation.coordinate_system,
                        translation.width,
                        translation.height,
                        translation.x,
                        translation.y,
                    );
                    writeln!(&mut stdout, "{}", translation.text);
                }
            }
            xabelfish_engine::XabelFishTranslation::String(text) => {
                writeln!(&mut stdout, "{}", text);
            }
        }
    }
}
