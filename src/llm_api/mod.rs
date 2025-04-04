use config::{
    get_api_key, get_api_retry, get_api_retry_delay, get_api_url, get_model_id, get_temperature,
};
use entities::{LLMMsg, ReqBody, RespBody};
use eyre::bail;
use reqwest::{
    Response,
    header::{HeaderMap, HeaderValue},
};
use search::{is_search_api, show_search_resp_content};
use std::{sync::OnceLock, time::Duration};

use color_eyre::eyre::Result;
use reqwest::{
    Client, ClientBuilder,
    header::{AUTHORIZATION, CONTENT_TYPE},
};

pub mod config;
pub mod entities;
pub mod search;

async fn post_with_retry(
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
                log::warn!("Retry after {} seconds", retry_delay);
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

pub async fn get_llm_completion(query: &str) -> Result<String> {
    let model_id = get_model_id();
    let tmptr = get_temperature();
    let payload = ReqBody {
        model: model_id.to_owned(),
        temperature: *tmptr,
        messages: vec![LLMMsg {
            role: entities::AllowedRole::USER,
            content: query.to_owned(),
        }],
    };

    let client = get_llm_api_client();
    let url = get_api_url();
    let resp = post_with_retry(
        client,
        url,
        &payload,
        get_api_retry(),
        get_api_retry_delay(),
    )
    .await?;

    #[cfg(debug_assertions)]
    {
        if is_search_api() {
            show_search_resp_content(resp).await?;
            bail!("Stopped for search api");
        }
    }

    let resp_json: RespBody = resp.json().await?;
    let ans = resp_json.get_content()?;
    Ok(ans)
}

fn get_llm_api_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| match construct_llm_api_client() {
        Ok(cli) => cli,
        Err(e) => {
            panic!("Failed to construct client: {}", e);
        }
    })
}

fn construct_llm_api_client() -> Result<Client> {
    let mut headers = HeaderMap::new();
    let api_key = get_api_key();
    let auth_val = format!("Bearer {}", api_key);
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_val)?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = ClientBuilder::new().default_headers(headers).build()?;
    Ok(client)
}
