use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OcrRequest {
    pub config: String,
    pub text: String,
    pub image_path: String
}

pub struct OcrResponse {
    pub result: String
}