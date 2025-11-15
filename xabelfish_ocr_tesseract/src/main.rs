use std::io::*;
use std::path::Path;

use clap::Parser;
use rusty_tesseract::Image;
use xabelfish_config::ocr::TesseractConfig;
use xabelfish_socket_protocol::ocr::OcrMessage;
use xabelfish_unix_socket::unix_socket_client::UnixSocketClient;

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
        let request: OcrMessage = {
            let request = client.recv().expect("Failed to get request");
            if let Some(request) = request {
                request
            } else {
                continue;
            }
        };

        match request {
            OcrMessage::OcrRequest(request) => {
                let config = TesseractConfig::from_toml(&request.config);
                let dynamic_image =
                    image::ImageReader::new(Cursor::new(request.image_bytes.as_slice()))
                        .with_guessed_format()
                        .unwrap()
                        .decode()
                        .unwrap();
                let image = Image::from_dynamic_image(&dynamic_image).unwrap();

                let tesseract_args = rusty_tesseract::Args {
                    lang: config.data_lang,
                    dpi: config.dpi,
                    psm: config.psm,
                    oem: config.oem,
                    config_variables: config.config_variables,
                };

                let result = rusty_tesseract::image_to_string(&image, &tesseract_args).unwrap();

                client
                    .send(&OcrMessage::OcrTextResponseBody(result))
                    .unwrap();
            }
            _ => continue,
        }
    }
}
