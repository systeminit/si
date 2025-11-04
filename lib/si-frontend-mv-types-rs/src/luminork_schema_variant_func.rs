use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionKind,
    FuncKind,
};
use si_id::FuncId;

use crate::management::ManagementFuncKind;

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::DefinitionChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct LuminorkSchemaVariantFunc {
    pub id: FuncId,
    pub func_kind: FuncKindVariant,
    pub is_overlay: bool,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::DefinitionChecksum,
)]
#[serde(rename_all = "camelCase")]
pub enum FuncKindVariant {
    Action(ActionKind),
    Management(ManagementFuncKind),
    Other(FuncKind), // fallback for other variants you might have
}
