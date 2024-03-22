use crate::attribute::value::AttributeValueError;
use crate::{AttributeValue, AttributeValueId, DalContext};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CodeViewError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("no code language found for string: {0}")]
    NoCodeLanguageForString(String),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
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

/// A view on "OutputStream" from cyclone.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeViewOutputStreamView {
    pub stream: String,
    pub line: String,
    pub level: String,
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

#[derive(Deserialize, Debug)]
struct CodeGenerationEntry {
    pub code: Option<String>,
    pub format: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

impl CodeView {
    pub async fn new(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> Result<Option<Self>, CodeViewError> {
        let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let code_view_name = match attribute_value.key(ctx).await? {
            Some(key) => key,
            None => return Ok(None),
        };

        let func_execution = match attribute_value.materialized_view(ctx).await? {
            Some(func_execution) => func_execution,
            None => return Ok(None),
        };

        let code_gen_entry: CodeGenerationEntry = serde_json::from_value(func_execution)?;
        if code_gen_entry.code.is_none() || code_gen_entry.format.is_none() {
            return Ok(None);
        }

        // Safe unwraps because of the above check
        let format = code_gen_entry.format.as_ref().unwrap();
        let code = code_gen_entry.code.as_ref().unwrap();

        let language = if format.is_empty() {
            CodeLanguage::Unknown
        } else {
            CodeLanguage::try_from(format.to_owned())?
        };

        let code = if code.is_empty() {
            None
        } else {
            Some(code.clone())
        };

        let message = code_gen_entry.message.clone();

        Ok(Some(CodeView::assemble(
            language,
            code,
            message,
            Some(code_view_name),
        )))
    }

    pub fn assemble(
        language: CodeLanguage,
        code: Option<String>,
        message: Option<String>,
        func: Option<String>,
    ) -> CodeView {
        CodeView {
            language,
            code,
            message,
            func,
        }
    }
}
