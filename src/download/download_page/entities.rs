use color_eyre::eyre::Result;
use regex::Regex;
use reqwest::Url;
use serde::{Deserialize, Serialize};

use crate::utils::is_url_accessible;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageAns {
    component_name: String,
    available: bool,
    site_url: Option<String>,
    #[serde(skip_deserializing, default)]
    pub abnoarmal: bool,
}

impl PageAns {
    pub async fn refrac_with_valid_url(&self) -> Result<Option<PageAns>> {
        if !self.available || self.site_url.is_none() {
            log::warn!("component {} is not available", self.component_name);
            return Ok(None);
        }
        if self.site_url.is_none() {
            return Ok(None);
        }

        let url = self.site_url.as_ref().unwrap().clone();
        let (flag, url_op) = is_url_accessible(&url).await;
        if !flag {
            log::warn!(
                "url {} of component {} is not accessible",
                url,
                self.component_name
            );
            return Ok(None);
        }

        let res_url = url_op.unwrap();

        if !Self::is_official_url(&res_url)? {
            log::warn!(
                "url {url} of component {} is not url of official site",
                self.component_name
            );
            return Ok(None);
        }
        log::info!("url for component {} if {}", self.component_name, res_url);
        Ok(Some(Self {
            component_name: self.component_name.clone(),
            available: true,
            site_url: Some(res_url),
            abnoarmal: false,
        }))
    }

    pub fn get_url(&self) -> String {
        self.site_url
            .as_ref()
            .unwrap_or_else(|| {
                panic!("Get url from an unavailable PageAns");
            })
            .clone()
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
