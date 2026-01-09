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
    EventSessionId,
    RebaseBatchAddressKind,
    WorkspacePk,
    WorkspaceSnapshotAddress,
};

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq, Versioned)]
#[serde(rename_all = "camelCase")]
#[acceptable(version = 4)]
pub enum EnqueueUpdatesRequestV4 {
    Rebase {
        id: RequestId,
        request: RebaseRequestV4,
    },
    BeginApplyToHead {
        id: RequestId,
        request: BeginApplyToHeadRequestV4,
    },
    ApplyToHead {
        id: RequestId,
        request: ApplyToHeadRequestV4,
    },
    CreateChangeSet {
        id: RequestId,
        request: CreateChangeSetRequestV4,
    },
    ChangeSetStatusUpdate {
        id: RequestId,
        request: ChangeSetStatusUpdateRequestV4,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RebaseRequestV4 {
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub updates_address: RebaseBatchAddressKind,
    pub from_change_set_id: Option<ChangeSetId>,
    pub event_session_id: Option<EventSessionId>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BeginApplyMode {
    LockSchemasAndFuncs,
    NoLock,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BeginApplyToHeadRequestV4 {
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub head_change_set_id: ChangeSetId,
    pub head_change_set_address: WorkspaceSnapshotAddress,
    pub event_session_id: EventSessionId,
    pub previous_change_set_status: Option<ChangeSetStatus>,
    pub mode: BeginApplyMode,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApplyToHeadRequestV4 {
    pub workspace_id: WorkspacePk,
    pub head_change_set_id: ChangeSetId,
    pub head_change_set_address: WorkspaceSnapshotAddress,
    pub change_set_to_apply_id: ChangeSetId,
    pub event_session_id: EventSessionId,
    pub previous_change_set_status: ChangeSetStatus,
    pub mode: Option<BeginApplyMode>,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetRequestV4 {
    pub workspace_id: WorkspacePk,
    pub head_change_set_id: ChangeSetId,
    pub name: String,
    pub event_session_id: EventSessionId,
}

#[derive(Clone, Debug, Deserialize, Eq, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetStatusUpdateRequestV4 {
    pub workspace_id: WorkspacePk,
    pub change_set_id: ChangeSetId,
    pub status: ChangeSetStatus,
    pub event_session_id: EventSessionId,
}

impl EnqueueUpdatesRequestV4 {
    pub fn request_id(&self) -> RequestId {
        match self {
            EnqueueUpdatesRequestV4::Rebase { id, .. } => *id,
            EnqueueUpdatesRequestV4::BeginApplyToHead { id, .. } => *id,
            EnqueueUpdatesRequestV4::ApplyToHead { id, .. } => *id,
            EnqueueUpdatesRequestV4::CreateChangeSet { id, .. } => *id,
            EnqueueUpdatesRequestV4::ChangeSetStatusUpdate { id, .. } => *id,
        }
    }

    /// This is the workspace id that will be used for the NATs subject
    pub fn workspace_id(&self) -> WorkspacePk {
        match self {
            EnqueueUpdatesRequestV4::Rebase { request, .. } => request.workspace_id,
            EnqueueUpdatesRequestV4::BeginApplyToHead { request, .. } => request.workspace_id,
            EnqueueUpdatesRequestV4::ApplyToHead { request, .. } => request.workspace_id,
            EnqueueUpdatesRequestV4::CreateChangeSet { request, .. } => request.workspace_id,
            EnqueueUpdatesRequestV4::ChangeSetStatusUpdate { request, .. } => request.workspace_id,
        }
    }

    /// The change set id that will be used for the NATs subject
    pub fn change_set_id(&self) -> ChangeSetId {
        match self {
            EnqueueUpdatesRequestV4::Rebase { request, .. } => request.change_set_id,
            EnqueueUpdatesRequestV4::BeginApplyToHead { request, .. } => request.change_set_id,
            EnqueueUpdatesRequestV4::ApplyToHead { request, .. } => request.head_change_set_id,
            EnqueueUpdatesRequestV4::CreateChangeSet { request, .. } => request.head_change_set_id,
            EnqueueUpdatesRequestV4::ChangeSetStatusUpdate { request, .. } => request.change_set_id,
        }
    }

    /// Set event session id if not set
    pub fn set_event_session_id(&mut self, event_session_id: EventSessionId) {
        if let EnqueueUpdatesRequestV4::Rebase { request, .. } = self {
            if request.event_session_id.is_none() {
                request.event_session_id = Some(event_session_id);
            }
        }
    }

    pub fn event_session_id(&self) -> Option<EventSessionId> {
        match self {
            EnqueueUpdatesRequestV4::Rebase { request, .. } => request.event_session_id,
            EnqueueUpdatesRequestV4::BeginApplyToHead { request, .. } => {
                Some(request.event_session_id)
            }
            EnqueueUpdatesRequestV4::ApplyToHead { request, .. } => Some(request.event_session_id),
            EnqueueUpdatesRequestV4::CreateChangeSet { request, .. } => {
                Some(request.event_session_id)
            }
            EnqueueUpdatesRequestV4::ChangeSetStatusUpdate { request, .. } => {
                Some(request.event_session_id)
            }
        }
    }
}
