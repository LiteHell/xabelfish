use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TranslateRequest {
    pub config: String,
    pub text: String,
    pub image_path: String,
}

pub struct TranslateResponse {
    pub result: String,
}
