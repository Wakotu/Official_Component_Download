use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::utils::construct_semaphore;

use super::download_link::{DLEntryPool, entities::DLEntry};
use color_eyre::eyre::Result;
use eyre::bail;
use path::{create_dir_if_nonexist, get_offical_dl_dir};
use reqwest::Client;
use tokio::{io::AsyncWriteExt, sync::Semaphore};

pub mod path;

impl DLEntryPool {
    fn get_comp_dir(&self) -> Result<PathBuf> {
        let ofi_dir = get_offical_dl_dir()?;
        let comp_dir = ofi_dir.join(&self.comp_name);
        create_dir_if_nonexist(&comp_dir)?;
        Ok(comp_dir)
    }
    fn get_download_link_file_path(&self) -> Result<PathBuf> {
        let comp_dir = self.get_comp_dir()?;
        let fpath = comp_dir.join("downloadlinks.txt");
        Ok(fpath)
    }

    fn write_download_links(&self) -> Result<()> {
        let fpath = self.get_download_link_file_path()?;
        let mut file = std::fs::File::create(&fpath)?;

        for ent in self.entries.iter() {
            writeln!(file, "{}: {}", ent.fname_ext, ent.url)?;
        }
        Ok(())
    }

    pub async fn download(&self) -> Result<()> {
        if self.is_empty() {
            return Ok(());
        }
        let mut hdl_set = vec![];
        let smph = Arc::new(construct_semaphore());
        for ent in self.entries.iter() {
            let ent = ent.clone();
            let smph = smph.clone();
            let hdl = tokio::spawn(async move { ent.download_worker(&smph).await });
            hdl_set.push(hdl);
        }
        for hdl in hdl_set {
            let _ = hdl.await;
        }
        self.write_download_links()?;
        Ok(())
    }
}

impl DLEntry {
    pub async fn download_worker(&self, smph: &Semaphore) -> Result<()> {
        let _permit = smph.acquire().await?;
        self.download().await?;
        Ok(())
    }

    fn get_comp_dir(&self) -> Result<PathBuf> {
        let ofi_dir = get_offical_dl_dir()?;
        let comp_dir = ofi_dir.join(&self.comp_name);
        create_dir_if_nonexist(&comp_dir)?;
        Ok(comp_dir)
    }

    fn get_comp_repo_dir(&self) -> Result<PathBuf> {
        let comp_dir = self.get_comp_dir()?;
        let repo_dir = comp_dir.join("repos");
        create_dir_if_nonexist(&repo_dir)?;
        Ok(repo_dir)
    }

    fn get_download_path(&self) -> Result<PathBuf> {
        let repo_dir = self.get_comp_repo_dir()?;
        Ok(repo_dir.join(&self.fname_ext))
    }

    async fn download(&self) -> Result<()> {
        let fpath = self.get_download_path()?;
        Self::download_file(&self.url, &fpath).await?;
        Ok(())
    }

    async fn download_file(url: &str, fpath: &Path) -> Result<()> {
        log::info!("Download {} to {:?}", url, fpath);
        let cli = Client::new();
        let resp = cli.get(url).send().await?;

        if !resp.status().is_success() {
            bail!("Failed to download {}", url);
        }

        let mut file = tokio::fs::File::create(fpath).await?;
        let content = resp.bytes().await?;
        file.write_all(&content).await?;

        Ok(())
    }
}
