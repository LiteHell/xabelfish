use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum TranslatorType {
    DeepL,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeepLConfig {
    pub api_key: String,
}

impl DeepLConfig {
    pub fn from_toml(str: &str) -> Self {
        toml::from_str(str).unwrap()
    }

    pub fn default() -> Self {
        Self {
            api_key: "PUT_API_KEY_HERE".to_string(),
        }
    }
}
