use std::time::Duration;

use color_eyre::eyre::Result;
use colored::*;
use eyre::bail;
use reqwest::{Client, Method, Response, Url};
use tokio::sync::Semaphore;

use crate::llm_api::{
    config::{get_api_check_retry, get_api_retry_delay, get_api_timeout, get_parralel_count},
    entities::ReqBody,
};

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
    let retry = get_api_check_retry();
    let delay = get_api_retry_delay();
    let time_out = get_api_timeout();

    for _ in 0..retry {
        let resp = client
            .request(Method::HEAD, url) // Use HEAD request for efficiency
            .timeout(std::time::Duration::from_secs(time_out as u64)) // Optional: Set a timeout
            .send()
            .await;

        if let Ok(resp) = resp {
            let url = resp.url().to_string();
            return (resp.status().is_success(), Some(url));
        } else {
            log::warn!(
                "Url Access Check: failed to access url {}, retry after {} seconds...",
                url,
                delay
            );
            tokio::time::sleep(Duration::from_secs(delay as u64)).await;
            continue;
        }
    }

    (false, None)
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

pub async fn get_with_retry(
    cli: &Client,
    url: &str,
    retry: usize,
    retry_delay: usize,
) -> Result<Response> {
    let mut att = 0;

    while att < retry {
        att += 1;

        let resp_res = cli.get(url).send().await;
        match resp_res {
            Err(e) => {
                log::warn!("Failed to request url {}: {}.", url, e);
                log::warn!("Retry after {} seconds...", retry_delay);
                tokio::time::sleep(Duration::from_secs(retry_delay as u64)).await;
                continue;
            }
            Ok(resp) => {
                return Ok(resp);
            }
        }
    }

    bail!(
        "Failed to get to get reponse from url {} with max retry of {}",
        url,
        retry
    );
}

pub async fn post_with_retry(
    cli: &Client,
    url: &str,
    payload: &ReqBody,
    retry: usize,
    retry_delay: usize,
) -> Result<Response> {
    let mut att = 0;

    while att < retry {
        att += 1;

        let resp_res = cli.post(url).json(payload).send().await;
        match resp_res {
            Err(e) => {
                log::warn!("Failed to request url {}: {}.", url, e);
                log::warn!("Retry after {} seconds...", retry_delay);
                tokio::time::sleep(Duration::from_secs(retry_delay as u64)).await;
                continue;
            }
            Ok(resp) => {
                return Ok(resp);
            }
        }
    }

    bail!(
        "Failed to get to get reponse from url {} with max retry of {}",
        url,
        retry
    );
}
