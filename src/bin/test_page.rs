use color_eyre::eyre::Result;
use official_download::{
    download::download_page::get_download_page_batch, utils::init_report_utils,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_report_utils()?;
    let comp_name_list = ["whois", "which", "wireless", "wireshark"];
    let url_list = get_download_page_batch(&comp_name_list).await?;
    log::debug!("url list: {:?}", url_list);
    Ok(())
}
