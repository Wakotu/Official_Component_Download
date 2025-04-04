use color_eyre::eyre::Result;
use regex::Regex;
use reqwest::Url;

#[derive(Debug, Clone)]
pub struct DLEntry {
    pub url: String,
    pub fname: String,
    pub fname_ext: String,
    pub comp_name: String,
}

impl DLEntry {
    fn get_last_path(url: &Url) -> Option<&str> {
        let mut seg_iter = url.path_segments()?;
        seg_iter.next_back()
    }

    fn get_entry_from_fname(fname: &str, url: &str, comp_name: &str) -> Result<Option<Self>> {
        let ver_op = Self::get_version_from_fname(fname)?;
        if ver_op.is_none() {
            return Ok(None);
        }

        let ver = ver_op.unwrap();
        Ok(Some(Self {
            url: url.to_string(),
            fname: format!("{}-{}", comp_name, ver),
            fname_ext: fname.to_string(),
            comp_name: comp_name.to_string(),
        }))
    }

    fn get_version_str_from_match(re: &Regex, fname: &str) -> Option<String> {
        let mat = re.find(fname)?;
        Some(mat.as_str().to_string())
    }

    fn get_version_from_fname(fname: &str) -> Result<Option<String>> {
        let re = Regex::new(r"\d+(\.\d+([[:alnum:]])?)")?;
        let res_op = Self::get_version_str_from_match(&re, fname);
        Ok(res_op)
    }

    pub fn from_url(url: &str, comp_name: &str) -> Result<Option<Self>> {
        let url_par = Url::parse(url)?;
        match Self::get_last_path(&url_par) {
            Some(ext_fname) => {
                let ent_op = Self::get_entry_from_fname(ext_fname, url, comp_name)?;
                Ok(ent_op)
            }
            None => Ok(None),
        }
    }
}

impl PartialEq for DLEntry {
    fn eq(&self, other: &Self) -> bool {
        self.fname.eq(&other.fname)
    }
}

impl Eq for DLEntry {}

impl PartialOrd for DLEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DLEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.fname.cmp(&other.fname)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::init_report_utils;
    use color_eyre::eyre::Result;

    use super::*;

    #[test]
    fn test_version_regex() -> Result<()> {
        init_report_utils()?;
        let re = Regex::new(r"\d+(\.\d+([[:alnum:]]+)?)")?;
        let mat = re.find("iftop-1.0pre4.tar.gz");
        log::debug!("mat: {:?}", mat);
        assert!(mat.is_some());
        Ok(())
    }
}
