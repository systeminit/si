use derive_builder::UninitializedFieldError;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use super::SpecError;

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
pub enum AttrFuncInputSpecKind {
    InputSocket,
    OutputSocket,
    Prop,
}

#[remain::sorted]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AttrFuncInputSpec {
    InputSocket { name: String, socket_name: String },
    OutputSocket { name: String, socket_name: String },
    Prop { name: String, prop_path: String },
}

#[derive(Clone, Debug, Default)]
pub struct AttrFuncInputSpecBuilder {
    kind: Option<AttrFuncInputSpecKind>,
    name: Option<String>,
    prop_path: Option<String>,
    socket_name: Option<String>,
}

impl AttrFuncInputSpec {
    pub fn builder() -> AttrFuncInputSpecBuilder {
        AttrFuncInputSpecBuilder::default()
    }
}

impl AttrFuncInputSpecBuilder {
    pub fn kind(&mut self, kind: AttrFuncInputSpecKind) -> &mut Self {
        self.kind = Some(kind);
        self
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn prop_path(&mut self, prop_path: impl Into<String>) -> &mut Self {
        self.prop_path = Some(prop_path.into());
        self
    }

    pub fn socket_name(&mut self, socket_name: impl Into<String>) -> &mut Self {
        self.socket_name = Some(socket_name.into());
        self
    }

    pub fn build(&self) -> Result<AttrFuncInputSpec, SpecError> {
        Ok(match self.kind {
            Some(kind) => match kind {
                AttrFuncInputSpecKind::Prop => AttrFuncInputSpec::Prop {
                    name: self
                        .name
                        .clone()
                        .ok_or(UninitializedFieldError::from("name"))?,
                    prop_path: self
                        .clone()
                        .prop_path
                        .ok_or(UninitializedFieldError::from("prop_path"))?,
                },
                AttrFuncInputSpecKind::InputSocket => AttrFuncInputSpec::InputSocket {
                    name: self
                        .name
                        .clone()
                        .ok_or(UninitializedFieldError::from("name"))?,
                    socket_name: self
                        .socket_name
                        .clone()
                        .ok_or(UninitializedFieldError::from("socket_name"))?,
                },
                AttrFuncInputSpecKind::OutputSocket => AttrFuncInputSpec::OutputSocket {
                    name: self
                        .name
                        .clone()
                        .ok_or(UninitializedFieldError::from("name"))?,
                    socket_name: self
                        .socket_name
                        .clone()
                        .ok_or(UninitializedFieldError::from("socket_name"))?,
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}
