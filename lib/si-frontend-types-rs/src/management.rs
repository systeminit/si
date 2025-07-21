use serde::{
    Deserialize,
    Serialize,
};
use si_events::Timestamp;
use si_id::{
    ChangeSetId,
    ComponentId,
    FuncRunId,
    ManagementFuncJobStateId,
    ManagementPrototypeId,
    UserPk,
    WorkspacePk,
};

#[remain::sorted]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ManagementState {
    Executing,
    Failure,
    Operating,
    Pending,
    Success,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagementFuncJobState {
    pub id: ManagementFuncJobStateId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub component_id: ComponentId,
    pub prototype_id: ManagementPrototypeId,
    pub user_id: Option<UserPk>,
    pub func_run_id: Option<FuncRunId>,
    pub state: ManagementState,
    pub timestamp: Timestamp,
    pub message: Option<String>,
}
