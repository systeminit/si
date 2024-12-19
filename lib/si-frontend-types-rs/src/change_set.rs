use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{
    ulid::Ulid, ChangeSetApprovalKind, ChangeSetApprovalStatus, ChangeSetId, ChangeSetStatus,
    UserPk,
};

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSet {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub id: ChangeSetId,
    pub status: ChangeSetStatus,
    pub base_change_set_id: Option<ChangeSetId>,
    pub workspace_id: String,
    pub merge_requested_by_user_id: Option<String>,
    pub merge_requested_by_user: Option<String>,
    pub merge_requested_at: Option<DateTime<Utc>>,
    pub reviewed_by_user_id: Option<String>,
    pub reviewed_by_user: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeSetApprovals {
    pub required: Vec<ChangeSetRequiredApproval>,
    pub current: Vec<ChangeSetApproval>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeSetRequiredApproval {
    // What is the kind of the entity corresponding to the ID?
    kind: ChangeSetApprovalKind,
    // What is the ID of the entity that is requiring approvals?
    id: Ulid,
    // What is the minimum number needed?
    number: usize,
    // Is it satisfied?
    is_satisfied: bool,
    // Who can satisfy this?
    users: Vec<UserPk>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChangeSetApproval {
    // Who approved this?
    pub user_id: UserPk,
    // What kind of approval did they do (including negative)?
    pub status: ChangeSetApprovalStatus,
    // Is this still valid?
    pub is_valid: bool,
}
