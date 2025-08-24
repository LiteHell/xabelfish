use std::{
    default,
    fs::{self, create_dir_all},
    io,
    path::PathBuf,
    sync::{Mutex, OnceLock},
};

use dirs::config_dir;
use serde::{Deserialize, Serialize};

mod default_values;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct XabelFishConfig {
    pub deepl_api_key: String,
    #[serde(default = "default_values::default_font_family")]
    pub font_family: String,
    #[serde(default = "default_values::default_font_size")]
    pub font_size: u32,
    #[serde(default = "default_values::default_background_color")]
    pub background_color: String,
    #[serde(default = "default_values::default_font_color")]
    pub font_color: String,
}

impl XabelFishConfig {
    fn new() -> Self {
        Self {
            deepl_api_key: String::new(),
            font_family: default_values::default_font_family(),
            font_size: default_values::default_font_size(),
            background_color: default_values::default_background_color(),
            font_color: default_values::default_font_color(),
        }
    }
}

fn get_xabelfish_config_dir_path() -> PathBuf {
    let mut config_file_path = config_dir().unwrap_or(PathBuf::from("."));
    config_file_path.push("xabelfish");

    config_file_path
}

fn get_xabelfish_config_file_path() -> PathBuf {
    let mut config_file_path = get_xabelfish_config_dir_path();
    config_file_path.push("config.toml");

    config_file_path
}

fn read_xabelfish_config() -> XabelFishConfig {
    let config_file_path = get_xabelfish_config_file_path();
    let config_file_content_result = fs::read_to_string(config_file_path);

    if let Ok(config_file_content) = config_file_content_result {
        let deserialized = toml::from_str::<XabelFishConfig>(&config_file_content.as_str());

        if let Ok(deserialized) = deserialized {
            return deserialized;
        }
    }

    return XabelFishConfig::new();
}

static XABELFISH_CONFIG_CACHE: Mutex<Option<XabelFishConfig>> = Mutex::new(None);

pub fn get_xabelfish_config() -> XabelFishConfig {
    let mut cache_write_lock = XABELFISH_CONFIG_CACHE
        .lock()
        .expect("Error on lock acquision for xabelfish config reading");
    if cache_write_lock.is_none() {
        *cache_write_lock = Some(read_xabelfish_config());
    }

    let config = cache_write_lock.clone().unwrap();
    config
}

pub fn set_xabelfish_config(config: XabelFishConfig) {
    println!("got config data from fe: {:#?}", config);

    let mut cache_write_lock = XABELFISH_CONFIG_CACHE
        .lock()
        .expect("Error on lock acquision for xabelfish config reading");
    let config_file_path = get_xabelfish_config_file_path();
    let serialized = toml::to_string(&config).expect("Failed to serialize config");

    println!("saving config: {serialized}");

    let _ = create_dir_all(get_xabelfish_config_dir_path());

    let result = fs::write(config_file_path, serialized);
    if result.is_ok() {
        *cache_write_lock = Some(read_xabelfish_config());
    } else {
        *cache_write_lock = None;
    }
}
