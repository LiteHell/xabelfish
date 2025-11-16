mod tesseract_data_to_paragraph;

use std::path::Path;
use std::{fs::File, io::*};

use clap::Parser;
use rusty_tesseract::{Data, Image};
use xabelfish_config::ocr::TesseractConfig;
use xabelfish_socket_protocol::ocr::{OcrBoundedBoxText, OcrMessage, OcrZeroCoordinatePosition};
use xabelfish_unix_socket::unix_socket_client::UnixSocketClient;

use crate::tesseract_data_to_paragraph::TesseractParagraph;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandArgs {
    #[arg(short, long)]
    socket_path: String,
    #[arg(short, long)]
    lock_file: String,
}

fn main() {
    let args = CommandArgs::parse();

    let lockfile = File::open(args.lock_file).unwrap();
    lockfile.lock().expect("Failed to get a lock");

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

                if config.positioned_ocr {
                    let tesseract_data = rusty_tesseract::image_to_data(&image, &tesseract_args)
                        .expect("Failed to perform tesseract ocr");
                    let paragraphs = TesseractParagraph::from_tesseract_data(tesseract_data.data);

                    let responses: Vec<OcrBoundedBoxText> = paragraphs
                        .into_iter()
                        .map(|i| OcrBoundedBoxText {
                            coordinate_system: OcrZeroCoordinatePosition::RightTop,
                            x: i.left,
                            y: i.top,
                            width: i.width as u32,
                            height: i.height as u32,
                            text: i.words_to_string(),
                        })
                        .collect();

                    client
                        .send(&OcrMessage::OcrBoundedBoxText(responses))
                        .unwrap();
                } else {
                    let result = rusty_tesseract::image_to_string(&image, &tesseract_args).unwrap();

                    client
                        .send(&OcrMessage::OcrTextResponseBody(result))
                        .unwrap();
                }
            }
            _ => continue,
        }
    }
}
