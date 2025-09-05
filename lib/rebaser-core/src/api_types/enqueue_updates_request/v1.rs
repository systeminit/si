use acceptable::{
    RequestId,
    Versioned,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    WorkspacePk,
    rebase_batch_address::RebaseBatchAddress,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
#[serde(rename_all = "camelCase")]
#[acceptable(version = 1)]
pub struct EnqueueUpdatesRequestV1 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub updates_address: RebaseBatchAddress,
    pub from_change_set_id: Option<ChangeSetId>,
}
