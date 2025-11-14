use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum TranslatorType {
    DeepL,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DeepLConfig {
    pub api_key: String,
}
