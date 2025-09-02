use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionKind,
    workspace_snapshot::EntityKind,
};
use si_id::{
    ActionPrototypeId,
    FuncId,
    SchemaVariantId,
};

use crate::reference::ReferenceKind;

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
pub struct ActionPrototypeView {
    pub id: ActionPrototypeId,
    pub func_id: FuncId,
    pub kind: ActionKind,
    pub display_name: Option<String>,
    pub name: String,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
    si_frontend_mv_types_macros::MV,
)]
#[serde(rename_all = "camelCase")]
#[mv(
    trigger_entity = EntityKind::SchemaVariant,
    reference_kind = ReferenceKind::ActionPrototypeViewList,
)]
pub struct ActionPrototypeViewList {
    pub id: SchemaVariantId,
    pub action_prototypes: Vec<ActionPrototypeView>,
}
