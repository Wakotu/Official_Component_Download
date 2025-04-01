use std::str::FromStr;

use color_eyre::eyre::Result;
use eyre::bail;
use regex::Regex;

pub fn sanitize_ans(ans: &str) -> Result<String> {
    let re = Regex::new(r"```json([^`]+)```")?;
    if let Some(caps) = re.captures(ans) {
        let san_ans = String::from_str(&caps[1])?;
        Ok(san_ans)
    } else {
        bail!("Failed to match the answer: {}", ans);
    }
}
