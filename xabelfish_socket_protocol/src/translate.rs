use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum TranslateSourceLanguage {
    Automatic,
    Specified(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TranslationRequestBody {
    pub config: String,
    pub src: TranslateSourceLanguage,
    pub dst: String,
    pub texts: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TranslateMessage {
    TranslationRequest(TranslationRequestBody),
    TranslationResponse(Vec<String>),
}
