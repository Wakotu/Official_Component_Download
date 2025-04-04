use color_eyre::eyre::Result;
use colored::*;
use reqwest::{Client, Method, Url};
use tokio::sync::Semaphore;

use crate::llm_api::config::get_parralel_count;

fn my_format(
    write: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> std::io::Result<()> {
    let level = match record.level() {
        log::Level::Error => "ERROR".red().bold(),
        log::Level::Warn => "WARN".yellow().bold(),
        log::Level::Info => "INFO".green().bold(),
        log::Level::Debug => "DEBUG".blue().bold(),
        log::Level::Trace => "TRACE".purple().bold(),
    };
    write!(
        write,
        "[{}] {} - {}",
        now.now().format("%Y-%m-%d %H:%M:%S"),
        level,
        record.args()
    )?;
    Ok(())
}

pub fn init_flexi_logger() -> Result<()> {
    flexi_logger::Logger::try_with_env_or_str("debug")?
        .format(my_format)
        .start()?;
    Ok(())
}

/// return accessibility along with resutl url
pub async fn is_url_accessible(url: &str) -> (bool, Option<String>) {
    let client = Client::new();
    let response = client
        .request(Method::HEAD, url) // Use HEAD request for efficiency
        .timeout(std::time::Duration::from_secs(10)) // Optional: Set a timeout
        .send()
        .await;

    if let Ok(resp) = response {
        let url = resp.url().to_string();
        (resp.status().is_success(), Some(url))
    } else {
        (false, None)
    }
}

pub fn init_report_utils() -> Result<()> {
    init_flexi_logger()?;
    color_eyre::install()?;
    Ok(())
}

pub fn is_absolute_url(url: &str) -> bool {
    let url_par = Url::parse(url);
    url_par.is_ok()
}

pub fn construct_semaphore() -> Semaphore {
    let max_concur = get_parralel_count();
    Semaphore::new(max_concur)
}
