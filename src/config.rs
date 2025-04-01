use crate::llm_api::config::ApiConfig;
use serde::Deserialize;
use std::fs::{self};
use std::sync::OnceLock;

const CONFIG_FILENAME: &str = "config/config.toml";

#[derive(Deserialize)]
pub struct AppConfig {
    pub api: ApiConfig,
    pub download: DLConfig,
}

#[derive(Deserialize)]
pub struct DLConfig {
    username: String,
    max_version_count: usize,
}

pub fn load_config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let toml_str = fs::read_to_string(CONFIG_FILENAME).unwrap();
        toml::from_str(&toml_str).unwrap()
    })
}

pub fn get_ver_cnt() -> usize {
    let config = load_config();
    config.download.max_version_count
}

pub fn get_username() -> &'static str {
    let config = load_config();
    &config.download.username
}
