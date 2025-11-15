use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum TranslatorType {
    DeepL,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeepLConfig {
    #[serde(default = "default_api_key")]
    pub api_key: String,
}

fn default_api_key() -> String {
    "PUT_API_KEY_HERE".to_string()
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
