pub mod download_link;
pub mod download_page;
pub mod file_download;

use std::{str::FromStr, sync::Arc};

use color_eyre::eyre::Result;
use download_link::DLEntryPool;
use download_page::get_download_page;
use file_download::path::get_download_comp_name_list;
use tokio::sync::Semaphore;

use crate::utils::construct_semaphore;

async fn download_worker(comp_name: &str, smph: &Semaphore) -> Result<()> {
    let _permit = smph.acquire().await?;
    let page_url = get_download_page(comp_name).await?;
    if page_url.is_none() {
        return Ok(());
    }
    let page_url = page_url.unwrap();
    let dl_pool = DLEntryPool::from_page_url(&page_url, comp_name).await?;
    dl_pool.download().await?;

    Ok(())
}

pub async fn download() -> Result<()> {
    let comp_name_list = get_download_comp_name_list()?;
    log::info!("{} components found", comp_name_list.len());
    log::info!("example components: {:?}", &comp_name_list[0..5]);
    let mut hdl_set = vec![];
    let smph = Arc::new(construct_semaphore());

    for comp in comp_name_list.iter() {
        let comp_name = String::from_str(comp)?;
        let smph = smph.clone();
        let hdl = tokio::spawn(async move { download_worker(&comp_name, &smph).await });
        hdl_set.push(hdl);
    }

    for hdl in hdl_set {
        let _ = hdl.await;
    }

    Ok(())
}
