use color_eyre::Result;
use futures::future::join_all;
use official_download::{llm_api::get_llm_completion, utils::init_report_utils};

const NUM: u32 = 5;
fn get_prompt_list() -> Vec<String> {
    let mut res = vec![];
    for idx in 0..NUM {
        res.push(format!("give me the answer of {} + {}", idx + 1, idx * 2));
    }
    res
}

fn show_answer(ans: &str) {
    println!("{}", ans);
}

#[tokio::main]
async fn main() -> Result<()> {
    init_report_utils()?;
    let prompt_list = get_prompt_list();
    let completions = prompt_list.iter().map(|q| get_llm_completion(q));

    let res_list = join_all(completions).await;
    for res in res_list.iter() {
        match res {
            Ok(ans) => show_answer(ans),
            Err(e) => {
                eprintln!("{e}");
            }
        }
        println!();
    }
    Ok(())
}
