pub mod ocr;
pub mod translator;

use std::{fs, path::PathBuf};

use dirs::config_dir;
use serde::{Deserialize, Serialize};

use crate::{
    ocr::{OcrType, TesseractConfig},
    translator::{DeepLConfig, TranslatorType},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct XabelFishEngineConfig {
    pub translator_type: TranslatorType,
    pub ocr_type: OcrType,
    #[serde(default = "DeepLConfig::default")]
    deepL: DeepLConfig,
    #[serde(default = "TesseractConfig::default")]
    tesseract: TesseractConfig,
}

impl XabelFishEngineConfig {
    pub fn get_config() -> Self {
        XabelFishEngineConfig::load().unwrap_or(Self {
            translator_type: TranslatorType::DeepL,
            ocr_type: OcrType::Tesseract,
            deepL: DeepLConfig::default(),
            tesseract: TesseractConfig::default(),
        })
    }

    pub fn get_ocr_config_string(&self) -> String {
        toml::to_string(match self.ocr_type {
            OcrType::Tesseract => &self.tesseract,
        })
        .unwrap()
    }

    pub fn get_translator_config_string(&self) -> String {
        toml::to_string(match self.translator_type {
            TranslatorType::DeepL => &self.deepL,
        })
        .unwrap()
    }

    pub fn save(&self) {
        let config_path = make_config_filepath("engine.toml".to_string());
        let serialized = toml::to_string_pretty(&self).unwrap();

        fs::write(config_path, serialized);
    }

    fn load() -> Option<Self> {
        let config_path = make_config_filepath("engine.toml".to_string());
        let config_file_content_result = fs::read_to_string(config_path);

        if let Ok(config_file_content) = config_file_content_result {
            let deserialized =
                toml::from_str::<XabelFishEngineConfig>(&config_file_content.as_str());

            if let Ok(deserialized) = deserialized {
                return Some(deserialized);
            }
        }

        return None;
    }
}

fn make_config_filepath(filename: String) -> PathBuf {
    let mut config_filepath = get_config_dir();
    config_filepath.push(filename);

    return config_filepath;
}

fn get_config_dir() -> PathBuf {
    let mut config_dir = config_dir().unwrap_or(PathBuf::from("."));
    config_dir.push("xabelfish");

    return config_dir;
}
