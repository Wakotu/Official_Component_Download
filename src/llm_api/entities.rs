use color_eyre::eyre::Result;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// request json spec
#[derive(Serialize)]
pub struct ReqBody {
    pub model: String,
    pub temperature: f64,
    pub messages: Vec<LLMMsg>,
}

#[derive(Serialize)]
pub struct LLMMsg {
    pub role: AllowedRole,
    pub content: String,
}

#[derive(Serialize)]
pub enum AllowedRole {
    #[serde(rename = "user")]
    USER,
    #[serde(rename = "system")]
    SYSTEM,
}

/// response json spec
#[derive(Deserialize)]
pub struct RespBody {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChoiceObj>,
}

#[derive(Deserialize)]
pub struct ChoiceObj {
    index: usize,
    message: RespMsg,
    finish_reason: String,
}

#[derive(Deserialize)]
pub struct RespMsg {
    role: String,
    content: String,
}

impl RespBody {
    pub fn get_content(&self) -> Result<String> {
        assert!(!self.choices.is_empty(), "No available choices: {}", self);
        Ok(self.choices[0].message.content.clone())
    }
}

impl Display for RespBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "************* Response ***********")?;
        writeln!(f, "Response id: {}", self.id)?;
        for cho in self.choices.iter() {
            writeln!(f, "\nModel: {}", self.model)?;
            writeln!(f, "Answer {}", cho.index)?;
            writeln!(f, "role: {}", cho.message.role)?;
            writeln!(f, "finish reason: {}", cho.finish_reason)?;
            write!(f, "Answer Content:\n{}", cho.message.content)?;
        }
        Ok(())
    }
}
