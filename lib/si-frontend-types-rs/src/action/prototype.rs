use serde::{Deserialize, Serialize};
use si_events::{
    workspace_snapshot::{Checksum, ChecksumHasher, EntityKind},
    ActionKind,
};
use si_id::{ActionPrototypeId, FuncId, SchemaVariantId};

use crate::checksum::FrontendChecksum;
use crate::{
    object::FrontendObject,
    reference::{Refer, Reference, ReferenceId, ReferenceKind},
    MaterializedView,
};

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Eq, Clone, si_frontend_types_macros::FrontendChecksum,
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
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
    si_frontend_types_macros::Refer,
    si_frontend_types_macros::MV,
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
