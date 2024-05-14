use base64::{engine::general_purpose, Engine};
use derive_builder::Builder;
use object_tree::Hash;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use url::Url;

use super::SpecError;

#[remain::sorted]
#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
#[serde(rename_all = "camelCase")]
pub enum FuncArgumentKind {
    Any,
    Array,
    Boolean,
    Integer,
    Json,
    Map,
    Object,
    String,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct FuncArgumentSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub kind: FuncArgumentKind,
    #[builder(setter(into), default)]
    pub element_kind: Option<FuncArgumentKind>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub unique_id: Option<String>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,
}

impl FuncArgumentSpec {
    pub fn builder() -> FuncArgumentSpecBuilder {
        FuncArgumentSpecBuilder::default()
    }
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, AsRefStr, Display, EnumIter, EnumString)]
#[serde(rename_all = "camelCase")]
pub enum FuncSpecBackendKind {
    Array,
    Boolean,
    Diff,
    Identity,
    Integer,
    JsAction,
    JsAttribute,
    JsAuthentication,
    Json,
    JsReconciliation,
    JsSchemaVariantDefinition,
    JsValidation,
    Map,
    Object,
    String,
    Unset,
    Validation,
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, AsRefStr, Display, EnumIter, EnumString)]
#[serde(rename_all = "camelCase")]
pub enum FuncSpecBackendResponseType {
    Action,
    Array,
    Boolean,
    CodeGeneration,
    Identity,
    Integer,
    Json,
    Map,
    Object,
    Qualification,
    Reconciliation,
    SchemaVariantDefinition,
    String,
    Unset,
    Validation,
    Void,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct FuncSpecData {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub display_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into))]
    pub handler: String,
    #[builder(setter(into))]
    pub code_base64: String,
    #[builder(setter(into))]
    pub backend_kind: FuncSpecBackendKind,
    #[builder(setter(into))]
    pub response_type: FuncSpecBackendResponseType,
    #[builder(setter(into), default)]
    pub hidden: bool,
    #[builder(setter(into, strip_option), default)]
    pub link: Option<Url>,
}

impl FuncSpecData {
    #[must_use]
    pub fn builder() -> FuncSpecDataBuilder {
        FuncSpecDataBuilder::default()
    }
}

impl FuncSpecDataBuilder {
    #[allow(unused_mut)]
    pub fn try_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.link(converted))
    }

    pub fn code_plaintext(&mut self, code: impl Into<String>) -> &mut Self {
        let code_plaintext = code.into();
        self.code_base64(general_purpose::STANDARD_NO_PAD.encode(code_plaintext))
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct FuncSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub unique_id: String,
    #[builder(setter(into, strip_option), default)]
    pub data: Option<FuncSpecData>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub is_from_builtin: Option<bool>,

    #[builder(setter(each(name = "argument"), into), default)]
    pub arguments: Vec<FuncArgumentSpec>,
}

impl FuncSpecBuilder {
    pub fn gen_unique_id(&self) -> Result<String, SpecError> {
        let mut bytes = vec![];

        bytes.extend_from_slice(self.name.as_deref().unwrap_or("").as_bytes());
        bytes.extend_from_slice(self.deleted.unwrap_or(false).to_string().as_bytes());
        if let Some(data) = &self.data {
            bytes.extend_from_slice(serde_json::to_string(data)?.as_bytes());
        }

        Ok(Hash::new(&bytes).to_string())
    }
}

impl FuncSpec {
    #[must_use]
    pub fn builder() -> FuncSpecBuilder {
        FuncSpecBuilder::default()
    }
}
