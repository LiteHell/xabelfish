mod engine;

use std::sync::mpsc;

fn main() {
    let mut engine = engine::XabelFishEngine::new();

    let (mut rw, tx) = mpsc::channel();
    engine.start_nonblocking(&mut rw);

    loop {}
}
