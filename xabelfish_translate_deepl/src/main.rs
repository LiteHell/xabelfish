use std::path::Path;

use clap::Parser;
use serde::{Deserialize, Serialize};
use xabelfish_config::translator::DeepLConfig;
use xabelfish_socket_protocol::translate::TranslateMessage;
use xabelfish_unix_socket::unix_socket_client::UnixSocketClient;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CommandArgs {
    #[arg(short, long)]
    socket_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeepLTranslateRequestBody {
    text: Vec<String>,
    target_lang: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeepLTranslateResponseTranslation {
    detected_source_language: Option<String>,
    text: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeepLTranslateResponse {
    translations: Vec<DeepLTranslateResponseTranslation>,
}

fn main() {
    let args = CommandArgs::parse();

    let mut client =
        UnixSocketClient::connect(Path::new(&args.socket_path)).expect("Failed to connect socket");

    let http_client = reqwest::blocking::Client::new();
    loop {
        let request: TranslateMessage = {
            let request = client.recv().expect("Failed to get request");
            if let Some(request) = request {
                request
            } else {
                continue;
            }
        };
        match request {
            TranslateMessage::TranslationRequest(request) => {
                let config = DeepLConfig::from_toml(&request.config);
                let request_body = serde_json::to_string(&DeepLTranslateRequestBody {
                    target_lang: String::from("ko"),
                    text: request.texts.clone(),
                })
                .unwrap();

                let http_response = http_client
                    .post("https://api-free.deepl.com/v2/translate")
                    .header(
                        "Authorization",
                        format!("DeepL-Auth-Key {}", config.api_key),
                    )
                    .header("User-Agent", "XabelFish/0.1.0")
                    .header("Content-Type", "application/json")
                    .body(request_body)
                    .send()
                    .unwrap();

                let response_text = http_response.text().unwrap();
                let response_parsed: DeepLTranslateResponse =
                    serde_json::from_str(&response_text).unwrap();

                client.send(&TranslateMessage::TranslationResponse(
                    response_parsed
                        .translations
                        .iter()
                        .map(|i| i.text.clone())
                        .collect(),
                ));
            }
            _ => continue,
        }
    }
}
