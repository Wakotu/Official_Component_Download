use color_eyre::eyre::Result;
use official_download::{download, utils::init_report_utils};

#[tokio::main]
async fn main() -> Result<()> {
    init_report_utils()?;
    download::download().await?;
    Ok(())
}
