use std::{io::{Read, Write}, os::unix::net::UnixStream};

use serde_json::{self, Error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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

pub fn read_line_for_unix_stream(stream: &mut UnixStream, buffer: &mut String) -> Result<(), std::io::Error> {
    let mut vec = vec![];
    let mut buf = [0; 256];
    loop {
        let read = stream.read(&mut buf)?;
        
        let mut terminated = false;
        for i in 0..read {
            if buf[i] == 0 {
                terminated = true;
            }
        }

        vec.extend_from_slice(&buf[0..read]);
        if terminated {
            vec.pop(); // pop nul terminator
            *buffer = String::from_utf8(vec).expect("Failed to parse utf8 string");
            break;
        }
    }

    Ok(())
}
