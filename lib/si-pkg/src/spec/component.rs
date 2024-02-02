use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::SpecError;
use super::{attribute_value::AttributeValueSpec, position::PositionSpec};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ComponentSpecVariant {
    BuiltinVariant {
        schema_name: String,
        variant_name: String,
        //        hash: String,
    },
    UpdateVariant {
        schema_name: String,
        variant_name: String,
        //        hash: String,
    },
    WorkspaceVariant {
        variant_unique_id: String,
    },
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct ComponentSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub position: PositionSpec,
    #[builder(setter(into))]
    pub variant: ComponentSpecVariant,
    #[builder(setter(into))]
    pub needs_destroy: bool,
    #[builder(setter(into))]
    pub deletion_user_pk: Option<String>,
    #[builder(setter(into))]
    pub unique_id: String,
    #[builder(setter(into))]
    pub deleted: bool,
    #[builder(setter(into))]
    pub hidden: bool,

    #[builder(setter(each(name = "attribute"), into), default)]
    pub attributes: Vec<AttributeValueSpec>,

    #[builder(setter(each(name = "input_socket"), into), default)]
    pub input_sockets: Vec<AttributeValueSpec>,

    #[builder(setter(each(name = "output_socket"), into), default)]
    pub output_sockets: Vec<AttributeValueSpec>,
}

impl ComponentSpec {
    pub fn builder() -> ComponentSpecBuilder {
        ComponentSpecBuilder::default()
    }
}
