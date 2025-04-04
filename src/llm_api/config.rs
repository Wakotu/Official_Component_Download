use crate::config::load_config;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ApiConfig {
    key: String,
    model_id: String,
    api_url: String,
    temperature: f64,
    parallel: usize,
    retry: usize,
    retry_delay: usize,
}

pub fn get_temperature() -> &'static f64 {
    let config = load_config();
    &config.api.temperature
}

pub fn get_api_key() -> &'static str {
    let config = load_config();
    &config.api.key
}

pub fn get_api_url() -> &'static str {
    let config = load_config();
    &config.api.api_url
}

pub fn get_model_id() -> &'static str {
    let config = load_config();
    &config.api.model_id
}

pub fn get_parralel_count() -> usize {
    let config = load_config();
    config.api.parallel
}

pub fn get_api_retry() -> usize {
    let config = load_config();
    config.api.retry
}

pub fn get_api_retry_delay() -> usize {
    let config = load_config();
    config.api.retry_delay
}
