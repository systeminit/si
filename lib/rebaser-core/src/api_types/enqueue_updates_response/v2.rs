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
    ChangeSetStatus,
    WorkspacePk,
};

use super::v1::RebaseStatus;

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
#[serde(rename_all = "camelCase")]
#[acceptable(version = 2)]
pub enum EnqueueUpdatesResponseV2 {
    Rebase {
        id: RequestId,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        status: RebaseStatus,
    },
    BeginApplyToHead {
        id: RequestId,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        status: BeginApplyStatus,
    },
    ApplyToHead {
        id: RequestId,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        status: ApplyToHeadStatus,
    },
    CreateChangeSet {
        id: RequestId,
        workspace_id: WorkspacePk,
        status: CreateChangeSetStatus,
    },
    UpdateChangeSetStatus {
        id: RequestId,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        status: UpdateChangeSetStatusStatus,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BeginApplyStatus {
    Success {
        previous_change_set_status: ChangeSetStatus,
    },
    Error {
        message: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CreateChangeSetStatus {
    Success { new_change_set_id: ChangeSetId },
    Error { message: String },
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum UpdateChangeSetStatusStatus {
    Retry {
        previous_change_set_status: ChangeSetStatus,
    },
    Success {
        previous_change_set_status: ChangeSetStatus,
    },
    Error {
        message: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ApplyToHeadStatus {
    Success,
    Retrying,
    Error { message: String },
}
