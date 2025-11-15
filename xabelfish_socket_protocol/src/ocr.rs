use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OcrRequestBody {
    pub config: String,
    pub image_type: String,
    pub image_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OcrZeroCoordinatePosition {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OcrBoundedBoxText {
    pub coordinate_system: OcrZeroCoordinatePosition,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum OcrMessage {
    OcrRequest(OcrRequestBody),
    OcrTextResponseBody(String),
    OcrBoundedBoxText(Vec<OcrBoundedBoxText>),
}
