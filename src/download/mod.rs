pub mod download_link;
pub mod download_page;
pub mod file_download;

use color_eyre::eyre::Result;
use download_page::get_download_page;
use file_download::path::get_download_comp_name_list;

fn download() -> Result<()> {
    let comp_name_list = get_download_comp_name_list()?;

    for comp in comp_name_list.iter() {
        let page_url = get_download_page(comp);
        // TODO: fill downloading logic
    }

    Ok(())
}
