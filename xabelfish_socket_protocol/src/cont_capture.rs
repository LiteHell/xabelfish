use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum ContinuousCaptureMessageType {
    // Cont Capture
    StartCapture,
    CaptureInitSuccess,
    CaptureInitFail,
    CaptureImageBytes,
    InvalidMessage,
    Ping,
    Pong,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContinuousCaptureMessage {
    pub message_type: ContinuousCaptureMessageType,
    pub extra_str: String,
    pub extra_bytes: Vec<u8>,
}

impl ContinuousCaptureMessage {
    pub fn invalid_message() -> Self {
        ContinuousCaptureMessage {
            message_type: ContinuousCaptureMessageType::InvalidMessage,
            extra_str: "".to_string(),
            extra_bytes: vec![]
        }
    }
    pub fn pong(extra: String) -> Self {
        ContinuousCaptureMessage {
            message_type: ContinuousCaptureMessageType::Pong,
            extra_str: extra,
            extra_bytes: vec![]
        }
    }
    pub fn png(bytes: Vec<u8>) -> Self {
        ContinuousCaptureMessage {
            message_type: ContinuousCaptureMessageType::CaptureImageBytes,
            extra_str: "png".to_string(),
            extra_bytes: bytes
        }
    }
}