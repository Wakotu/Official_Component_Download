use color_eyre::eyre::Result;
use regex::Regex;
use reqwest::{Client, Method, Url};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PageAns {
    component_name: String,
    available: bool,
    site_url: Option<String>,
}

impl PageAns {
    pub async fn get_valid_url(&self) -> Result<Option<String>> {
        if !self.available || self.site_url.is_none() {
            log::warn!("component {} is not available", self.component_name);
            return Ok(None);
        }
        if self.site_url.is_none() {
            return Ok(None);
        }

        let url = self.site_url.as_ref().unwrap().clone();
        if !Self::is_url_accessible(&url).await || !Self::is_official_url(&url)? {
            log::warn!("component {} is not available", self.component_name);
            return Ok(None);
        }
        Ok(Some(url))
    }

    async fn is_url_accessible(url: &str) -> bool {
        let client = Client::new();
        let response = client
            .request(Method::HEAD, url) // Use HEAD request for efficiency
            .timeout(std::time::Duration::from_secs(10)) // Optional: Set a timeout
            .send()
            .await;

        if let Ok(resp) = response {
            resp.status().is_success()
        } else {
            false
        }
    }

    fn is_official_url(url: &str) -> Result<bool> {
        let url_par = Url::parse(url)?;
        let host_str = url_par.host_str();
        match host_str {
            None => {
                log::warn!("Failed to get host part of the url: {:?}", url);
                Ok(false)
            }
            Some(val) => {
                let re = Regex::new(r"gitlab\.(\w+\.)?com")?;
                if val.contains("github.com") || re.is_match(val) {
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::init_report_utils;

    use super::*;
    use color_eyre::eyre::Result;

    #[test]
    fn test_gitlab_regex() -> Result<()> {
        init_report_utils()?;
        let re = Regex::new(r"gitlab\.(\w+\.)?com")?;
        let hay = "gitlab.unitslink.com";
        assert!(re.is_match(hay));
        Ok(())
    }
}
