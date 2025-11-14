pub mod ocr;
pub mod translator;

use std::{collections::HashMap, fs, path::PathBuf};

use dirs::config_dir;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{ocr::OcrType, translator::TranslatorType};

#[derive(Serialize, Deserialize, Clone)]
pub struct XabelFishEngineConfig {
    pub translator_type: TranslatorType,
    pub ocr_type: OcrType,
    translator_configs: HashMap<TranslatorType, String>,
    ocr_configs: HashMap<OcrType, String>,
}

impl XabelFishEngineConfig {
    pub fn default() -> Self {
        XabelFishEngineConfig::load().unwrap_or(Self {
            translator_type: TranslatorType::DeepL,
            ocr_type: OcrType::Tesseract,
            translator_configs: HashMap::new(),
            ocr_configs: HashMap::new(),
        })
    }

    pub fn set_ocr_config<T: Serialize>(&mut self, ocr_type: OcrType, config: T) {
        self.ocr_configs.insert(
            ocr_type,
            toml::to_string(&config).expect("Failed to serialize ocr config"),
        );
    }

    pub fn get_ocr_config<T: DeserializeOwned>(&mut self, ocr_type: OcrType) -> Option<T> {
        if let Some(deserialized) = self.ocr_configs.get(&ocr_type) {
            Some(toml::from_str(deserialized.as_str()).expect("Failed to deserialize ocr config"))
        } else {
            None
        }
    }
    pub fn set_translator_config<T: Serialize>(
        &mut self,
        translator_type: TranslatorType,
        config: T,
    ) {
        self.translator_configs.insert(
            translator_type,
            toml::to_string(&config).expect("Failed to serialize translator config"),
        );
    }

    pub fn get_translator_config<T: DeserializeOwned>(
        &mut self,
        translator_type: TranslatorType,
    ) -> Option<T> {
        if let Some(deserialized) = self.translator_configs.get(&translator_type) {
            Some(
                toml::from_str(deserialized.as_str())
                    .expect("Failed to deserialize translator config"),
            )
        } else {
            None
        }
    }

    pub fn save(&self) {
        let config_path = make_config_filepath("config.toml".to_string());
        let serialized = toml::to_string_pretty(&self).unwrap();

        fs::write(config_path, serialized);
    }

    fn load() -> Option<Self> {
        let config_path = make_config_filepath("config.toml".to_string());
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
