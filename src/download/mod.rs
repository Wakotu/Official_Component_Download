pub mod download_link;
pub mod download_page;
pub mod file_download;

use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fs, str::FromStr, sync::Arc};

use color_eyre::eyre::Result;
use download_link::DLEntryPool;
use download_page::{entities::PageAns, get_download_page};
use file_download::path::{
    get_download_comp_name_list, get_official_abnormal_page_fpath,
    get_official_available_page_fpath,
};
use serde::Serialize;
use serde_json::Serializer;
use serde_json::ser::PrettyFormatter;
use tokio::sync::Semaphore;

use crate::utils::construct_semaphore;

async fn download_worker(comp_name: &str, smph: &Semaphore) -> Result<Option<PageAns>> {
    let _permit = smph.acquire().await?;
    let page_ans = get_download_page(comp_name).await?;
    if page_ans.is_none() {
        return Ok(None);
    }
    let mut page = page_ans.unwrap();
    let dl_pool = DLEntryPool::from_page(&mut page, comp_name).await?;
    dl_pool.download().await?;

    Ok(Some(page))
}

fn save_page_json_pretty(page_ans_list: &[PageAns], fpath: &Path) -> Result<()> {
    let file = fs::File::create(fpath)?;
    let mut writer = BufWriter::new(file);

    let fmter = PrettyFormatter::with_indent(b"    ");
    let mut ser = Serializer::with_formatter(&mut writer, fmter);

    page_ans_list.serialize(&mut ser)?;

    writer.flush()?;

    log::info!("site url list has been written to {:?}", fpath);

    Ok(())
}

fn save_available_pages(page_ans_list: &[PageAns]) -> Result<()> {
    let fpath = get_official_available_page_fpath()?;

    save_page_json_pretty(page_ans_list, &fpath)?;
    Ok(())
}

fn save_abnormal_pages(page_ans_list: &[PageAns]) -> Result<()> {
    let fpath = get_official_abnormal_page_fpath()?;

    save_page_json_pretty(page_ans_list, &fpath)?;
    Ok(())
}

pub async fn download() -> Result<()> {
    let comp_name_list = get_download_comp_name_list()?;
    log::info!("{} components found", comp_name_list.len());
    log::info!("example components: {:?}", &comp_name_list[0..5]);
    let mut hdl_set = vec![];
    let smph = Arc::new(construct_semaphore());
    let mut page_ans_list = vec![];
    let mut abn_page_ans_list = vec![];

    for comp in comp_name_list.iter() {
        let comp_name = String::from_str(comp)?;
        let smph = smph.clone();
        let hdl = tokio::spawn(async move { download_worker(&comp_name, &smph).await });
        hdl_set.push(hdl);
    }

    for hdl in hdl_set {
        let res = hdl.await?;
        let ans_op = res?;
        if let Some(page_ans) = ans_op {
            page_ans_list.push(page_ans.clone());
            if page_ans.abnoarmal {
                abn_page_ans_list.push(page_ans);
            }
        }
    }

    save_available_pages(&page_ans_list)?;
    save_abnormal_pages(&abn_page_ans_list)?;

    Ok(())
}
