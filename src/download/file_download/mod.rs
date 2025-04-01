use std::{
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
    pub async fn download(&self) -> Result<()> {
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
        Ok(())
    }
}

impl DLEntry {
    pub async fn download_worker(&self, smph: &Semaphore) -> Result<()> {
        let _permit = smph.acquire().await?;
        self.download().await?;
        Ok(())
    }

    fn get_comp_download_path(&self) -> Result<PathBuf> {
        let ofi_dir = get_offical_dl_dir()?;
        Ok(ofi_dir.join(&self.comp_name))
    }

    fn get_download_path(&self) -> Result<PathBuf> {
        let comp_dir = self.get_comp_download_path()?;
        create_dir_if_nonexist(&comp_dir)?;
        Ok(comp_dir.join(&self.fname_ext))
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
