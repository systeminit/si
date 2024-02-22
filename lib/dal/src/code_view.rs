use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CodeViewError {
    #[error("no code language found for string: {0}")]
    NoCodeLanguageForString(String),
}

pub type CodeViewResult<T> = Result<T, CodeViewError>;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, Display, AsRefStr, PartialEq, Eq, Copy)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum CodeLanguage {
    Diff,
    Json,
    String,
    Unknown,
    Yaml,
}

impl TryFrom<String> for CodeLanguage {
    type Error = CodeViewError;

    fn try_from(value: String) -> CodeViewResult<Self> {
        match value.to_lowercase().as_str() {
            "diff" => Ok(Self::Diff),
            "json" => Ok(Self::Json),
            "string" => Ok(Self::String),
            "yaml" => Ok(Self::Yaml),
            "unknown" => Ok(Self::Unknown),
            _ => Err(CodeViewError::NoCodeLanguageForString(value)),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeView {
    pub language: CodeLanguage,
    /// None means the code is still being generated
    /// Used to avoid showing stale data
    pub code: Option<String>,
    pub message: Option<String>,
    pub func: Option<String>,
}

impl CodeView {
    pub fn new(
        language: CodeLanguage,
        code: Option<String>,
        message: Option<String>,
        func: Option<String>,
    ) -> Self {
        CodeView {
            func,
            language,
            code,
            message,
        }
    }
}
