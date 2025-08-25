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
    RebaseBatchAddressKind,
    WorkspacePk,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
#[serde(rename_all = "camelCase")]
#[acceptable(version = 1)]
pub struct EnqueueUpdatesResponseV1 {
    pub id: RequestId,
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub status: RebaseStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum RebaseStatus {
    Success {
        updates_performed: RebaseBatchAddressKind,
    },
    Error {
        message: String,
    },
}
