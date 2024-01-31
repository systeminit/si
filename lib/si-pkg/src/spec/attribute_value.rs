use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::{
    func::{FuncSpecBackendKind, FuncSpecBackendResponseType},
    AttrFuncInputSpec, SpecError,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AttributeValuePath {
    Prop {
        path: String,
        key: Option<String>,
        index: Option<i64>,
    },
    InputSocket(String),
    OutputSocket(String),
}

impl AttributeValuePath {
    pub fn path(&self) -> &str {
        match self {
            Self::Prop { path, .. } => path,
            Self::InputSocket(path) => path,
            Self::OutputSocket(path) => path,
        }
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
