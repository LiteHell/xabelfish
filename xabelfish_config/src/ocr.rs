use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum OcrType {
    Tesseract,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TesseractConfig {
    pub data_lang: String,
}
