use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum OcrMessageType {
    OcrRequest,
    OcrResponse,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OcrMessage {
    pub message_type: OcrMessageType,
    pub config: String,
    pub text: String,
    pub image_type: String,
    pub image_bytes: Vec<u8>,
}
