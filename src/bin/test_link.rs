use color_eyre::eyre::Result;
use official_download::{download::download_link::DLEntryPool, utils::init_report_utils};

#[tokio::main]
async fn main() -> Result<()> {
    init_report_utils()?;
    let pool =
        DLEntryPool::from_page_url("https://www.wireshark.org/download.html", "wireshark").await?;
    log::debug!(
        "len of pool: {}, example entry: {:?}",
        pool.len(),
        pool.entries[0]
    );

    let pool =
        DLEntryPool::from_page_url("https://ftp.gnu.org/gnu/coreutils/", "coreutils").await?;

    log::debug!(
        "len of pool: {}, example entry: {:?}",
        pool.len(),
        pool.entries[0]
    );
    Ok(())
}
