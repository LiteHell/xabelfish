use std::collections::HashMap;
use std::io::*;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use rusty_tesseract::Image;
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessage;
use xabelfish_socket_protocol::cont_capture::ContinuousCaptureMessageType;
use xabelfish_socket_protocol::ocr::OcrMessage;
use xabelfish_socket_protocol::ocr::OcrMessageType;
use xabelfish_unix_socket::unix_socket_client::UnixSocketClient;
use xabelfish_unix_socket::unix_socket_server::UnixSocketServer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandArgs {
    #[arg(short, long)]
    socket_path: String,
}

fn main() {
    let args = CommandArgs::parse();

    let mut client =
        UnixSocketClient::connect(Path::new(&args.socket_path)).expect("Failed to connect socket");

    loop {
        let request: OcrMessage = client.recv().expect("Failed to get request");

        let dynamic_image = image::ImageReader::new(Cursor::new(request.image_bytes.as_slice()))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();
        let image = Image::from_dynamic_image(&dynamic_image).unwrap();

        let tesseract_args = rusty_tesseract::Args {
            lang: "jpn".to_string(),
            dpi: Some(150),
            psm: Some(3),
            oem: Some(3),
            config_variables: HashMap::new(),
        };

        let result = rusty_tesseract::image_to_string(&image, &tesseract_args).unwrap();

        client
            .send(&OcrMessage {
                config: String::new(),
                image_bytes: Vec::new(),
                image_type: String::new(),
                message_type: OcrMessageType::OcrResponse,
                text: result,
            })
            .unwrap();
    }
}
