use std::fs::{self};
use std::sync::OnceLock;

use clap::Parser;
use file_config::AppConfig;

const CONFIG_FILENAME: &str = "config/config.toml";

pub mod file_config {
    use crate::llm_api::config::ApiConfig;
    use serde::Deserialize;

    #[derive(Deserialize)]
    /// app config primarily from config file
    pub struct AppConfig {
        pub api: ApiConfig,
        pub download: DLConfig,
    }

    #[derive(Deserialize)]
    pub struct DLConfig {
        pub username: String,
        pub max_version_count: usize,
        pub base_dir: String,
    }
}

pub mod cli_config {
    use clap::Parser;
    #[derive(Debug, Parser)]
    pub struct CliConfig {
        /// base dir for download
        #[arg(short, long)]
        pub base_dir: Option<String>,
    }
}

pub fn get_ver_cnt() -> usize {
    let config = load_config();
    config.download.max_version_count
}

pub fn get_username() -> &'static str {
    let config = load_config();
    &config.download.username
}

pub fn get_dl_base_dir() -> &'static str {
    let config = load_config();
    &config.download.base_dir
}

pub fn load_config() -> &'static AppConfig {
    static CONFIG: OnceLock<AppConfig> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let toml_str = fs::read_to_string(CONFIG_FILENAME).unwrap();
        let cli = cli_config::CliConfig::parse();
        let base_dir_op = cli.base_dir;
        let mut app_config: AppConfig = toml::from_str(&toml_str).unwrap();
        if let Some(base_dir) = base_dir_op {
            app_config.download.base_dir = base_dir;
        }
        app_config
    })
}
