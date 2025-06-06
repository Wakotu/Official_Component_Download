use std::{str::FromStr, sync::Arc};

use crate::{
    config::get_ver_cnt,
    llm_api::{
        config::{get_api_retry, get_api_retry_delay},
        get_llm_completion,
    },
    utils::{construct_semaphore, get_with_retry, is_absolute_url, is_url_accessible},
};
use color_eyre::eyre::Result;
use entities::DLEntry;
use handlebars::Handlebars;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde_json::json;
use tokio::sync::Semaphore;

use super::download_page::entities::PageAns;

pub mod entities;

async fn is_url_related_to_comp(url: &str, comp_name: &str) -> Result<bool> {
    let prompt_tempalate = r#"
Is the download url {{url}} related to the opensource component {{comp_name}}?
Please reply with a simple yes or no.
        "#;
    let reg = Handlebars::new();
    let data = json!({
    "comp_name": comp_name,
    "url":url,
    });
    let prmp = reg.render_template(prompt_tempalate, &data)?;
    let ans = get_llm_completion(&prmp).await?;

    let flag = ans.to_lowercase().contains("yes");

    #[cfg(debug_assertions)]
    {
        if url.eq("https://github.com/opencv/opencv/archive/refs/tags/4.11.0.zip") {
            log::warn!("opencv link related check: {}, ans: {}", url, flag);
        }
    }
    Ok(flag)
}

async fn get_page_content(page_url: &str) -> Result<String> {
    let cli = Client::new();
    log::info!("fetch content for page {}", page_url);
    let resp = get_with_retry(&cli, page_url, get_api_retry(), get_api_retry_delay()).await?;
    let text = resp.text().await?;
    Ok(text)
}

fn transform_href(href: &str, url: &str) -> Result<String> {
    if is_absolute_url(href) {
        return Ok(href.to_string());
    }

    let link = if href.starts_with("/") {
        let mut url_par = Url::parse(url)?;
        url_par.set_path(href);
        url_par.to_string()
    } else {
        format!("{}{}", url, href)
    };
    assert!(
        is_absolute_url(&link),
        "Result of url transformation invalid: {}",
        link
    );

    Ok(link)
}

fn get_all_links(page_content: &str, url: &str) -> Result<Vec<String>> {
    log::info!("start to get links for page {}", url);
    let doc = Html::parse_document(page_content);
    let sltr = Selector::parse("a").unwrap_or_else(|e| {
        panic!("Faield to construct a css selector: {e}");
    });

    let mut link_list = vec![];
    for a_ele in doc.select(&sltr) {
        if let Some(href) = a_ele.value().attr("href") {
            let url = transform_href(href, url)?;
            link_list.push(url);
        }
    }
    log::info!("{} links extracted from page {}", link_list.len(), url);
    log::debug!("link list: {:?}", link_list);
    Ok(link_list)
}

pub struct DLEntryPool {
    pub comp_name: String,
    pub entries: Vec<DLEntry>,
}

impl DLEntryPool {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn is_entry_seen(&self, ent: &DLEntry) -> bool {
        for it in self.entries.iter() {
            if it.eq(ent) {
                return true;
            }
        }
        false
    }

    pub fn push_ent(&mut self, ent: DLEntry) -> bool {
        if self.is_entry_seen(&ent) {
            return false;
        }
        self.entries.push(ent);
        true
    }

    async fn is_source_link(url: &str, comp_name: &str) -> Result<bool> {
        if url.contains(".sig") || url.contains(".exe") {
            return Ok(false);
        }

        let (flag, _) = is_url_accessible(url).await;
        if !flag {
            log::warn!(
                "extracted link {} for component {} is not accessible",
                url,
                comp_name
            );
            return Ok(false);
        }

        let prompt_tempalate = r#"
Does the URL {{url}} point to a compressed package containing the source code of the open-source component {{component}}?
Please reply with a simple 'yes' or 'no'.
        "#;
        let reg = Handlebars::new();
        let data = json!({
        "comp_name": comp_name,
        "url":url,
        });
        let prmp = reg.render_template(prompt_tempalate, &data)?;
        let ans = get_llm_completion(&prmp).await?;

        let flag = ans.to_lowercase().contains("yes");

        #[cfg(debug_assertions)]
        {
            if url.eq("https://github.com/opencv/opencv/archive/refs/tags/4.11.0.zip") {
                log::warn!("opencv link source check: {}, ans: {}", url, flag);
            }
        }

        Ok(flag)
    }

    async fn filter_url_worker(
        url: &str,
        comp_name: &str,
        smph: &Semaphore,
    ) -> Result<Option<DLEntry>> {
        let _permit = smph.acquire().await?;
        if !Self::is_source_link(url, comp_name).await?
            || !is_url_related_to_comp(url, comp_name).await?
        {
            return Ok(None);
        }

        let ent = DLEntry::from_url(url, comp_name)?;

        #[cfg(debug_assertions)]
        {
            if url.eq("https://github.com/opencv/opencv/archive/refs/tags/4.11.0.zip") {
                log::warn!("ent for opencv-411zip: {:?}", ent);
            }
        }
        Ok(ent)
    }

    pub async fn from_page(page_ans: &mut PageAns, comp_name: &str) -> Result<Self> {
        let page_url = page_ans.get_url();
        let (dl_pool, abn) = Self::from_page_url(&page_url, comp_name).await?;
        page_ans.abnoarmal = abn;
        Ok(dl_pool)
    }

    pub async fn from_page_url(page_url: &str, comp_name: &str) -> Result<(Self, bool)> {
        let content = get_page_content(page_url).await?;
        let url_list = get_all_links(&content, page_url)?;
        let mut pool = Self {
            entries: vec![],
            comp_name: comp_name.to_string(),
        };

        let mut hdl_set = vec![];
        let smph = Arc::new(construct_semaphore());

        for url in url_list.iter() {
            let url = url.clone();
            let comp_name = String::from_str(comp_name)?;
            let smph = smph.clone();

            let hdl =
                tokio::spawn(async move { Self::filter_url_worker(&url, &comp_name, &smph).await });
            hdl_set.push(hdl);
        }

        for hdl in hdl_set {
            let res = hdl.await?;
            let ent = res?;

            match ent {
                None => {
                    continue;
                }
                Some(ent) => {
                    #[cfg(debug_assertions)]
                    {
                        if ent
                            .url
                            .eq("https://github.com/opencv/opencv/archive/refs/tags/4.11.0.zip")
                        {
                            log::warn!("ent passed in join  for opencv-411zip: {:?}", ent);
                        }
                    }
                    pool.push_ent(ent);
                }
            }
        }

        let cnt = get_ver_cnt();
        pool.entries.sort_by(|a, b| b.cmp(a));

        let abn = pool.is_empty();

        let entries = if pool.len() > cnt {
            pool.entries[0..cnt].to_vec()
        } else {
            pool.entries.clone()
        };
        log::info!(
            "{} download entries collected for {}",
            entries.len(),
            comp_name
        );
        log::debug!("Download entries: {:?}", entries);
        let res_pool = Self {
            entries,
            comp_name: comp_name.to_string(),
        };

        Ok((res_pool, abn))
    }
}
