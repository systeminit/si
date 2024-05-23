use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use super::{AttrFuncInputSpec, SpecError};

#[remain::sorted]
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
)]
#[serde(rename_all = "camelCase")]
pub enum SocketSpecKind {
    Input,
    Output,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
    Copy,
    Default,
)]
#[serde(rename_all = "camelCase")]
pub enum SocketSpecArity {
    #[default]
    Many,
    One,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SocketSpecData {
    #[builder(setter(into, strip_option), default)]
    pub func_unique_id: Option<String>,

    #[builder(setter(into))]
    pub kind: SocketSpecKind,

    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into), default)]
    pub connection_annotations: String,

    #[builder(setter(into), default)]
    pub arity: SocketSpecArity,

    #[builder(setter(into), default)]
    pub ui_hidden: bool,
}

impl SocketSpecData {
    pub fn builder() -> SocketSpecDataBuilder {
        SocketSpecDataBuilder::default()
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SocketSpec {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into, strip_option), default)]
    #[serde(default)]
    pub data: Option<SocketSpecData>,

    #[builder(setter(each(name = "input"), into), default)]
    pub inputs: Vec<AttrFuncInputSpec>,

    #[builder(setter(into), default)]
    #[serde(default)]
    pub unique_id: Option<String>,
}

impl SocketSpec {
    pub fn builder() -> SocketSpecBuilder {
        SocketSpecBuilder::default()
    }

    pub fn kind(&self) -> Option<SocketSpecKind> {
        self.data.as_ref().map(|data| data.kind)
    }

    pub fn merge_socket_spec(&self, old_socket: &SocketSpec) -> SocketSpecBuilder {
        let mut builder = SocketSpec::builder();
        builder.name(self.clone().name);

        if let Some(new_data) = self.clone().data {
            let mut spec_data = new_data.clone();
            if new_data.func_unique_id.is_none() {
                if let Some(old_data) = old_socket.clone().data {
                    if old_data.func_unique_id.is_some() {
                        spec_data.func_unique_id = old_data.func_unique_id;
                    }
                }
            }

            builder.data(spec_data);
        }

        if self.clone().inputs.is_empty() && !old_socket.inputs.is_empty() {
            builder.inputs(old_socket.clone().inputs);
        }

        if old_socket.unique_id.is_some() && self.clone().unique_id.is_none() {
            builder.unique_id(old_socket.clone().unique_id);
        }

        builder
    }
}
