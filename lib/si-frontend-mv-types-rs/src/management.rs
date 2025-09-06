use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    FuncId,
    ManagementPrototypeId,
};
use strum::Display;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct MgmtPrototypeView {
    pub id: ManagementPrototypeId,
    pub func_id: FuncId,
    pub description: Option<String>,
    pub prototype_name: String,
    pub name: String,
    pub kind: ManagementFuncKind,
}

#[remain::sorted]
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Display,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub enum ManagementFuncKind {
    Discover,
    Import,
    Other,
    RunTemplate,
}
