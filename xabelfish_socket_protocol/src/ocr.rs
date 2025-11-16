use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcrRequestBody {
    pub config: String,
    pub image_type: String,
    pub image_bytes: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OcrZeroCoordinatePosition {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OcrBoundedBoxText {
    pub coordinate_system: OcrZeroCoordinatePosition,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OcrMessage {
    OcrRequest(OcrRequestBody),
    OcrTextResponseBody(String),
    OcrBoundedBoxText(Vec<OcrBoundedBoxText>),
}
