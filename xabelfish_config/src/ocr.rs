use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum OcrType {
    Tesseract,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TesseractConfig {
    #[serde(default = "default_data_lang")]
    pub data_lang: String,
    pub dpi: Option<i32>,
    pub psm: Option<i32>,
    pub oem: Option<i32>,
    #[serde(default = "HashMap::new")]
    pub config_variables: HashMap<String, String>,
    #[serde(default)]
    pub positioned_ocr: bool,
}

fn default_data_lang() -> String {
    "jpn".to_string()
}

impl TesseractConfig {
    pub fn from_toml(str: &str) -> Self {
        toml::from_str(str).unwrap()
    }

    pub fn default() -> Self {
        Self {
            data_lang: "jpn".to_string(),
            dpi: Some(150),
            oem: Some(3),
            psm: Some(3),
            config_variables: HashMap::new(),
            positioned_ocr: false,
        }
    }
}
