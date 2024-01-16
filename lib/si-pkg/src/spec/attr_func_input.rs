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
    InputSocket {
        name: String,
        socket_name: String,
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    OutputSocket {
        name: String,
        socket_name: String,
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
    Prop {
        name: String,
        prop_path: String,
        #[serde(default)]
        unique_id: Option<String>,
        #[serde(default)]
        deleted: bool,
    },
}

#[derive(Clone, Debug, Default)]
pub struct AttrFuncInputSpecBuilder {
    kind: Option<AttrFuncInputSpecKind>,
    name: Option<String>,
    prop_path: Option<String>,
    socket_name: Option<String>,
    unique_id: Option<String>,
    deleted: bool,
}

impl AttrFuncInputSpec {
    pub fn builder() -> AttrFuncInputSpecBuilder {
        AttrFuncInputSpecBuilder::default()
    }

    pub fn name(&self) -> &str {
        match self {
            Self::InputSocket { name, .. } => name.as_str(),
            Self::OutputSocket { name, .. } => name.as_str(),
            Self::Prop { name, .. } => name.as_str(),
        }
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

    pub fn unique_id(&mut self, unique_id: impl Into<String>) -> &mut Self {
        self.unique_id = Some(unique_id.into());
        self
    }

    pub fn deleted(&mut self, deleted: impl Into<bool>) -> &mut Self {
        self.deleted = deleted.into();
        self
    }

    pub fn build(&self) -> Result<AttrFuncInputSpec, SpecError> {
        let self_clone = self.to_owned();
        Ok(match self_clone.kind {
            Some(kind) => match kind {
                AttrFuncInputSpecKind::Prop => AttrFuncInputSpec::Prop {
                    name: self_clone
                        .name
                        .ok_or(UninitializedFieldError::from("name"))?,
                    prop_path: self_clone
                        .prop_path
                        .ok_or(UninitializedFieldError::from("prop_path"))?,
                    deleted: self_clone.deleted,
                    unique_id: self_clone.unique_id,
                },
                AttrFuncInputSpecKind::InputSocket => AttrFuncInputSpec::InputSocket {
                    name: self_clone
                        .name
                        .ok_or(UninitializedFieldError::from("name"))?,
                    socket_name: self_clone
                        .socket_name
                        .ok_or(UninitializedFieldError::from("socket_name"))?,
                    deleted: self_clone.deleted,
                    unique_id: self_clone.unique_id,
                },
                AttrFuncInputSpecKind::OutputSocket => AttrFuncInputSpec::OutputSocket {
                    name: self_clone
                        .name
                        .ok_or(UninitializedFieldError::from("name"))?,
                    socket_name: self_clone
                        .socket_name
                        .ok_or(UninitializedFieldError::from("socket_name"))?,
                    deleted: self_clone.deleted,
                    unique_id: self_clone.unique_id,
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}
