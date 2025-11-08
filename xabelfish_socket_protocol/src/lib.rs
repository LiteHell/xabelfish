use std::io::Write;

use serde_json::{self, Error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum MessageType {
    // Cont Capture
    StartCapture,
    CaptureInitSuccess,
    CaptureInitFail,
    CaptureImageBytes,
    InvalidMessage,
    Ping,
    Pong,

    // OCR
    OcrRequest,
    OcrResponse
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub message_type: MessageType,
    pub extra_str: String,
    pub extra_bytes: Vec<u8>,
}

impl Message {
    pub fn parse_from_str(str: &str) -> Result<Self, Error> {
        serde_json::from_str(str)
    }
    pub fn invalid_message() -> Self {
        Message {
            message_type: MessageType::InvalidMessage,
            extra_str: "".to_string(),
            extra_bytes: vec![]
        }
    }
    pub fn pong(extra: String) -> Self {
        Message {
            message_type: MessageType::Pong,
            extra_str: extra,
            extra_bytes: vec![]
        }
    }
    pub fn png(bytes: Vec<u8>) -> Self {
        Message {
            message_type: MessageType::CaptureImageBytes,
            extra_str: "png".to_string(),
            extra_bytes: bytes
        }
    }
    pub fn write_to(&self, writer: &mut dyn Write) -> Result<(), Error> {
        serde_json::to_writer(writer, &self)
    }
}

impl From<&str> for Message {
    fn from(str: &str) -> Self {
        serde_json::from_str(str).expect("Failed to parse protocol")
    }
}

impl From<String> for Message {
    fn from(str: String) -> Self {
        serde_json::from_str(str.as_str()).expect("Failed to parse protocol")
    }
}

impl ToString for Message {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}
