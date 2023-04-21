use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

use super::{FuncUniqueId, SpecError};

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
pub enum LeafKind {
    CodeGeneration,
    Qualification,
    Confirmation,
}

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
pub enum LeafInputLocation {
    Code,
    DeletedAt,
    Domain,
    Resource,
}

impl LeafInputLocation {
    pub fn try_from_arg_name(arg_name: &str) -> Result<Self, SpecError> {
        Ok(match arg_name {
            "domain" => LeafInputLocation::Domain,
            "code" => LeafInputLocation::Code,
            "resource" => LeafInputLocation::Resource,
            "deleted_at" => LeafInputLocation::DeletedAt,
            _ => {
                return Err(SpecError::LeafInputLocationConversionError(
                    arg_name.to_string(),
                ))
            }
        })
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct LeafFunctionSpec {
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,

    #[builder(setter(into))]
    pub leaf_kind: LeafKind,

    #[builder(setter(into))]
    pub inputs: Vec<LeafInputLocation>,
}

impl LeafFunctionSpec {
    pub fn builder() -> LeafFunctionSpecBuilder {
        LeafFunctionSpecBuilder::default()
    }
}
