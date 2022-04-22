use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Deserialize, Serialize, Debug, Clone, Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum CodeLanguage {
    Yaml,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeView {
    pub language: CodeLanguage,
    /// None means the code is still being generated
    /// Used to avoid showing stale data
    pub code: Option<String>,
}

impl CodeView {
    pub fn new(language: CodeLanguage, code: Option<String>) -> Self {
        let code = code.map(Into::into);
        CodeView { language, code }
    }
}
