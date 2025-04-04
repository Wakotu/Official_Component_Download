use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::Result;
use eyre::bail;

use crate::config::{get_dl_base_dir, get_username};

fn get_download_dir() -> Result<PathBuf> {
    let dir = Path::new(get_dl_base_dir());
    let uname = get_username();
    let dl_dir = dir.join(uname);
    if !dl_dir.is_dir() {
        bail!(
            "Download dir not found, please create it manually: {:?}",
            dl_dir
        );
    }
    Ok(dl_dir)
}

fn get_github_dl_dir() -> Result<PathBuf> {
    let dl_dir = get_download_dir()?;
    let gh_dir = dl_dir.join("GitHub");
    if !gh_dir.is_dir() {
        bail!(
            "Download dir for Github Components not found: {:?}. Please reorganize your download directory.",
            gh_dir
        );
    }
    Ok(gh_dir)
}

pub fn get_offical_dl_dir() -> Result<PathBuf> {
    let dl_dir = get_download_dir()?;
    let ofi_dir = dl_dir.join("Official");
    if !ofi_dir.is_dir() {
        bail!(
            "Download dir for Official Components not found: {:?}. Please reorganize your download directory.",
            ofi_dir
        );
    }
    Ok(ofi_dir)
}

pub fn get_official_available_page_fpath() -> Result<PathBuf> {
    let ofi_dir = get_offical_dl_dir()?;
    let fpath = ofi_dir.join("available_url_list.json");
    Ok(fpath)
}

pub fn get_official_abnormal_page_fpath() -> Result<PathBuf> {
    let ofi_dir = get_offical_dl_dir()?;
    let fpath = ofi_dir.join("abnormal_url_list.json");
    Ok(fpath)
}

fn get_sub_dir_name_list(dir: &Path) -> Result<Vec<String>> {
    let mut name_list: Vec<String> = vec![];
    let entries = fs::read_dir(dir)?;

    for ent in entries {
        let ent = ent?;
        let path = ent.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().ok_or_else(|| {
            panic!("Failed to get filename from {:?}", path);
        })?;

        let dir_name = dir_name.to_str().ok_or_else(|| {
            panic!("Failed to transform Osstr to str: {:?}", dir_name);
        })?;
        name_list.push(dir_name.to_string());
    }
    Ok(name_list)
}

pub fn create_dir_if_nonexist(dir: &Path) -> Result<()> {
    if !dir.is_dir() {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn get_download_comp_name_list() -> Result<Vec<String>> {
    let gh_dir = get_github_dl_dir()?;
    let res = get_sub_dir_name_list(&gh_dir)?;
    Ok(res)
}
