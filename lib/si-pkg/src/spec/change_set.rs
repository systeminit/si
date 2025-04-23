use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use strum::{
    Display,
    EnumString,
};

use super::{
    ComponentSpec,
    EdgeSpec,
    FuncSpec,
    SchemaSpec,
    SpecError,
};

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Display, EnumString, PartialEq, Eq, Clone, Copy)]
pub enum ChangeSetSpecStatus {
    Abandoned,
    Applied,
    Closed,
    Failed,
    Open,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct ChangeSetSpec {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into, strip_option), default)]
    pub based_on_change_set: Option<String>,

    #[builder(setter(into), default = "ChangeSetSpecStatus::Open")]
    pub status: ChangeSetSpecStatus,

    #[builder(setter(each(name = "component", into)), default)]
    #[serde(default)]
    pub components: Vec<ComponentSpec>,

    #[builder(setter(each(name = "schema", into)), default)]
    #[serde(default)]
    pub schemas: Vec<SchemaSpec>,

    #[builder(setter(each(name = "func", into)), default)]
    #[serde(default)]
    pub funcs: Vec<FuncSpec>,

    #[builder(setter(each(name = "edge", into)), default)]
    #[serde(default)]
    pub edges: Vec<EdgeSpec>,
}

impl ChangeSetSpec {
    pub fn builder() -> ChangeSetSpecBuilder {
        ChangeSetSpecBuilder::default()
    }
}
