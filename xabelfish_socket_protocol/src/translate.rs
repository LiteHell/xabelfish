use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum TranslateMessageType {
    TranslateRequest,
    TranslateResponse,
    // TO-DO: Impl these later...
    //CommaSeparatedSupportedSourceLanguageRequest,
    //CommaSeparatedSupportedTargetLanguageRequest,
    //AutomaticSourceDetectionSupportRequest,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TranslateSourceLanguage {
    Automatic,
    Specified(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TranslateMessage {
    pub message_type: TranslateMessageType,
    pub config: String,
    pub src: TranslateSourceLanguage,
    pub dst: String,
    pub data_text: String,
    pub data_bool: bool,
}
