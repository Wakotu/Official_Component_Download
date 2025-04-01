use std::{str::FromStr, sync::Arc};

use crate::{
    config::get_ver_cnt,
    llm_api::get_llm_completion,
    utils::{construct_semaphore, is_absolute_url},
};
use color_eyre::eyre::Result;
use entities::DLEntry;
use handlebars::Handlebars;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde_json::json;
use tokio::sync::Semaphore;

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
    Ok(ans.to_lowercase().contains("yes"))
}

async fn get_page_content(page_url: &str) -> Result<String> {
    let cli = Client::new();
    log::info!("fetch content for page {}", page_url);
    let resp = cli.get(page_url).send().await?;
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
    log::info!("get links for page {}", url);
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

        let prompt_tempalate = r#"
Is the download URL {{url}} for the source code of the open-source component {{comp_name}}?
Please reply with a simple 'yes' or 'no'.
        "#;
        let reg = Handlebars::new();
        let data = json!({
        "comp_name": comp_name,
        "url":url,
        });
        let prmp = reg.render_template(prompt_tempalate, &data)?;
        let ans = get_llm_completion(&prmp).await?;
        Ok(ans.to_lowercase().contains("yes"))
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
        Ok(ent)
    }

    pub async fn from_page_url(page_url: &str, comp_name: &str) -> Result<Self> {
        let content = get_page_content(page_url).await?;
        let url_list = get_all_links(&content, page_url)?;
        let mut pool = Self { entries: vec![] };

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
                    pool.push_ent(ent);
                }
            }
        }

        let cnt = get_ver_cnt();
        pool.entries.sort_by(|a, b| b.cmp(a));

        let entries = if pool.len() > cnt {
            pool.entries[0..cnt].to_vec()
        } else {
            pool.entries.clone()
        };
        log::info!("{} download entries collected.", entries.len());
        log::debug!("Download entries: {:?}", entries);
        let res_pool = Self { entries };

        Ok(res_pool)
    }
}
