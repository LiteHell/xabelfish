use std::{io::{Read, Write}, os::unix::net::UnixStream};

use serde_json::{self, Error};
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
    pub fn parse_from_str(str: &str) -> Result<Self, Error> {
        serde_json::from_str(str)
    }
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
    pub fn to_nul_ended_vec(&self) -> Vec<u8> {
        let str = self.clone().to_string();
        let bytes = str.as_bytes();
        let mut vec = Vec::from(bytes);
        vec.push(0);

        return vec;
    }
    pub fn write_to(&self, writer: &mut dyn Write) -> Result<(), Error> {
        serde_json::to_writer(writer, &self)
    }
}

impl From<&str> for ContinuousCaptureMessage {
    fn from(str: &str) -> Self {
        serde_json::from_str(str).expect("Failed to parse protocol")
    }
}

impl From<String> for ContinuousCaptureMessage {
    fn from(str: String) -> Self {
        serde_json::from_str(str.as_str()).expect("Failed to parse protocol")
    }
}

impl ToString for ContinuousCaptureMessage {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize")
    }
}
