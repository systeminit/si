use serde::{Deserialize, Serialize};

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
)]
pub enum SchematicKind {
    Deployment,
    Component,
}
