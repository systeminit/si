use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionKind,
    ActionState,
    workspace_snapshot::EntityKind,
};
use si_id::{
    ActionId,
    ActionPrototypeId,
    ChangeSetId,
    ComponentId,
    FuncRunId,
    WorkspacePk,
};

use crate::reference::ReferenceKind;

pub mod prototype;

pub use prototype::{
    ActionPrototypeView,
    ActionPrototypeViewList,
};

#[derive(
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    si_frontend_mv_types_macros::FrontendChecksum,
)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub id: ActionId,
    pub prototype_id: ActionPrototypeId,
    pub component_id: Option<ComponentId>,
    pub component_schema_name: Option<String>,
    pub component_name: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub kind: ActionKind,
    pub state: ActionState,
    pub originating_change_set_id: ChangeSetId,
    pub func_run_id: Option<FuncRunId>,
    // Actions that will wait until I've successfully completed before running
    pub my_dependencies: Vec<ActionId>,
    // Things that need to finish before I can start
    pub dependent_on: Vec<ActionId>,
    // includes action ids that impact this status
    // this occurs when ancestors of this action are on hold or have failed
    pub hold_status_influenced_by: Vec<ActionId>,
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
    trigger_entity = EntityKind::CategoryAction,
    reference_kind = ReferenceKind::ActionViewList,
)]
pub struct ActionViewList {
    pub id: WorkspacePk,
    pub actions: Vec<ActionView>,
}
