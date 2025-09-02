use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionState,
    workspace_snapshot::EntityKind,
};
use si_id::{
    ActionId,
    ComponentId,
    WorkspacePk,
};
use strum::{
    AsRefStr,
    Display,
    EnumString,
};

use crate::reference::ReferenceKind;

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
  trigger_entity = EntityKind::CategoryAction,
  reference_kind = ReferenceKind::ActionDiffList,
  build_priority = "List",
)]
pub struct ActionDiffList {
    pub id: WorkspacePk,
    pub action_diffs: HashMap<ActionId, ActionDiffView>,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::DefinitionChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ActionDiffView {
    pub id: ActionId,
    pub diff_status: ActionDiffStatus,
    pub component_id: ComponentId,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Deserialize,
    Serialize,
    Debug,
    Display,
    EnumString,
    PartialEq,
    Eq,
    Copy,
    Clone,
    si_frontend_mv_types_macros::DefinitionChecksum,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
pub enum ActionDiffStatus {
    Added {
        new_state: ActionState,
    },
    Modified {
        old_state: ActionState,
        new_state: ActionState,
    },
    None,
    Removed,
}
