use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum OcrType {
    Tesseract,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TesseractConfig {
    pub data_lang: String,
    pub dpi: Option<i32>,
    pub psm: Option<i32>,
    pub oem: Option<i32>,
    pub config_variables: HashMap<String, String>,
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
        }
    }
}
