use std::{str::FromStr, sync::Arc};

use color_eyre::eyre::Result;
use sanitize::sanitize_ans;
use serde_json::json;
use tokio::sync::Semaphore;

use crate::{llm_api::get_llm_completion, utils::construct_semaphore};
use entities::PageAns;

pub mod entities;
pub mod sanitize;

const PROMPT_TEMPLATE: &str = r#"
I'm looking for the official download URL for source code of the open-source component {{comp_name}}. 
If it's integrated into a larger project or an official download site isn't accessible, please let me know it's unavailable. 
If there are several download options, please prioritize official sources like GNU or Coreutils.
Please Reply with following json format:
```json
{
    "component_name": "<full name of the component>",
    "available": true,
    "site_url":"<url of offical download site>"
}
```
For example:
```json
{
    "component_name": "coreutils",
    "available": true,
    "site_url":"https://ftp.gnu.org/gnu/coreutils/"
}
```
"#;

fn get_prompt_for_comp(comp_name: &str) -> Result<String> {
    let reg = handlebars::Handlebars::new();
    let data = json!({
    "comp_name": comp_name
    });
    let prmp = reg.render_template(PROMPT_TEMPLATE, &data)?;
    Ok(prmp)
}

pub async fn get_download_page(comp_name: &str) -> Result<Option<PageAns>> {
    log::info!("query download page url for {}", comp_name);
    let query = get_prompt_for_comp(comp_name)?;
    let ans = get_llm_completion(&query).await?;
    let ans = sanitize_ans(&ans)?;
    log::debug!("Query Ans: {}", ans);
    let ans: PageAns = serde_json::from_str(&ans)?;
    log::info!("query for {} finished", comp_name);
    ans.refrac_with_valid_url().await
}

async fn page_worker(comp_name: &str, semp: &Semaphore) -> Result<Option<PageAns>> {
    let _permit = semp.acquire().await?;
    let page_ans = get_download_page(comp_name).await?;
    Ok(page_ans)
}

pub async fn get_download_page_batch(comp_name_list: &[&str]) -> Result<Vec<String>> {
    let semp = Arc::new(construct_semaphore());
    let mut hdl_set = vec![];
    let mut url_list = vec![];

    for comp_name in comp_name_list.iter() {
        let comp_name = String::from_str(comp_name)?;
        let semp = semp.clone();
        let hdl = tokio::spawn(async move { page_worker(&comp_name, &semp).await });
        hdl_set.push(hdl);
    }

    for hdl in hdl_set {
        let res = hdl.await?;
        let res = res?;
        if let Some(page) = res {
            let url = page.get_url();
            url_list.push(url);
        }
    }
    Ok(url_list)
}
