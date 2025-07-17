use core::fmt;
use std::hash::Hash;

use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    AttrFuncInputSpec,
    SpecError,
    func::{
        FuncSpecBackendKind,
        FuncSpecBackendResponseType,
    },
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AttributeValuePath {
    Prop {
        path: String,
        key_or_index: Option<KeyOrIndex>,
    },
    InputSocket(String),
    OutputSocket(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum KeyOrIndex {
    Key(String),
    Index(i64),
}
impl fmt::Display for KeyOrIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = match self {
            KeyOrIndex::Key(key) => write!(f, "[{key}]"),
            KeyOrIndex::Index(index) => write!(f, "[{index}]"),
        };
        Ok(())
    }
}
impl fmt::Display for AttributeValuePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributeValuePath::Prop { path, key_or_index } => {
                if let Some(attribute_value_index_or_key) = key_or_index {
                    write!(f, "{path}{attribute_value_index_or_key}")?
                } else {
                    write!(f, "{path}")?
                }
            }
            AttributeValuePath::InputSocket(path) => write!(f, "{path}")?,
            AttributeValuePath::OutputSocket(path) => write!(f, "{path}")?,
        };
        Ok(())
    }
}

/// This is the separator used for the "path" column. It is a vertical tab character, which should
/// not (we'll see) be able to be provided by our users in [`Prop`] names.
pub const ATTRIBUTE_VALUE_PATH_SEPARATOR: &str = "\x0B";

impl AttributeValuePath {
    pub fn path(&self) -> &str {
        match self {
            Self::Prop { path, .. } => path,
            Self::InputSocket(path) => path,
            Self::OutputSocket(path) => path,
        }
    }

    pub fn assemble_from_parts_with_separator(
        parts: impl IntoIterator<Item = AttributeValuePath>,
        separator: Option<&str>,
    ) -> String {
        // use default separator unless one is passed in
        let separator = match separator {
            Some(sep) => sep,
            None => ATTRIBUTE_VALUE_PATH_SEPARATOR,
        };
        parts
            .into_iter()
            .map(|part| part.to_string())
            .collect::<Vec<String>>()
            .join(separator)
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct AttributeValueSpec {
    #[builder(setter(into, strip_option), default)]
    pub parent_path: Option<AttributeValuePath>,
    #[builder(setter(into))]
    pub path: AttributeValuePath,
    #[builder(setter(into))]
    pub func_unique_id: String,
    #[builder(setter(into))]
    pub func_binding_args: serde_json::Value,
    #[builder(setter(into, strip_option), default)]
    pub handler: Option<String>,
    #[builder(setter(into))]
    pub backend_kind: FuncSpecBackendKind,
    #[builder(setter(into))]
    pub response_type: FuncSpecBackendResponseType,
    #[builder(setter(into, strip_option), default)]
    pub code_base64: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub unprocessed_value: Option<serde_json::Value>,
    #[builder(setter(into, strip_option), default)]
    pub value: Option<serde_json::Value>,
    #[builder(setter(into, strip_option), default)]
    pub output_stream: Option<serde_json::Value>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub is_proxy: bool,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub sealed_proxy: bool,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub component_specific: bool,
    #[builder(setter(each = "input"), default)]
    pub inputs: Vec<AttrFuncInputSpec>,
    #[builder(setter(into, strip_option), default)]
    pub implicit_value: Option<serde_json::Value>,
}

impl AttributeValueSpec {
    pub fn builder() -> AttributeValueSpecBuilder {
        AttributeValueSpecBuilder::default()
    }
}
