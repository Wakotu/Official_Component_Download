use color_eyre::eyre::Result;
use reqwest::Response;

use super::config::get_model_id;

pub fn is_search_api() -> bool {
    let model_id = get_model_id();
    model_id.contains("search")
}

pub async fn show_search_resp_content(resp: Response) -> Result<()> {
    let text = resp.text().await?;
    log::debug!("Raw Text: {}", text);
    Ok(())
}
