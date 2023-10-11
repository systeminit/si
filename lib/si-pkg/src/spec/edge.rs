use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use super::SpecError;

use strum::{AsRefStr, Display, EnumIter, EnumString};

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
pub enum EdgeSpecKind {
    Configuration,
    Symbolic,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct EdgeSpec {
    #[builder(setter(into))]
    pub edge_kind: EdgeSpecKind,
    #[builder(setter(into))]
    pub from_component_unique_id: String,
    #[builder(setter(into))]
    pub from_socket_name: String,
    #[builder(setter(into))]
    pub to_component_unique_id: String,
    #[builder(setter(into))]
    pub to_socket_name: String,

    #[builder(setter(into))]
    pub creation_user_pk: Option<String>,
    #[builder(setter(into))]
    pub deletion_user_pk: Option<String>,
    #[builder(setter(into))]
    pub deleted_implicitly: bool,

    #[builder(setter(into), default)]
    pub unique_id: String,
    #[builder(setter(into), default)]
    pub deleted: bool,
}

impl EdgeSpec {
    pub fn builder() -> EdgeSpecBuilder {
        EdgeSpecBuilder::default()
    }
}
