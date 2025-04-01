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

    fn match_last_path(ext_fname: &str, re: &Regex, url: &str, comp_name: &str) -> Option<Self> {
        let mat = re.find(ext_fname)?;
        let ver = mat.as_str();
        Some(Self {
            url: url.to_string(),
            fname: format!("{}-{}", comp_name, ver),
            fname_ext: ext_fname.to_string(),
            comp_name: comp_name.to_string(),
        })
    }

    pub fn from_url(url: &str, comp_name: &str) -> Result<Option<Self>> {
        let url_par = Url::parse(url)?;
        match Self::get_last_path(&url_par) {
            Some(ext_fname) => {
                let re = Regex::new(r"\d+(\.\d+)*")?;
                let ent_op = Self::match_last_path(ext_fname, &re, url, comp_name);
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
