mod engine;
mod max_sized_deque;

use std::sync::mpsc;

fn main() {
    let (mut tx, rx) = mpsc::channel();

    let mut engine = engine::XabelFishEngine::new(&mut tx);
    engine.start_nonblocking();

    for i in rx.iter() {
        println!("translated: {i}");
    }
}
