use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum CodeLanguage {
    Yaml,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CodeView {
    language: CodeLanguage,
    code: String,
}

impl CodeView {
    pub fn new(language: CodeLanguage, code: impl Into<String>) -> Self {
        let code = code.into();
        CodeView { language, code }
    }
}
