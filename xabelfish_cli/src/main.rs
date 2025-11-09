use std::sync::mpsc;

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use xabelfish_engine::XabelFishEngine;

fn main() {
    let (mut tx, rx) = mpsc::channel();

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
        writeln!(&mut stdout, "{}", i);
    }
}
