use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{
    workspace_snapshot::{ChecksumHasher, EntityKind},
    ChangeSetApprovalStatus, ChangeSetId, ChangeSetStatus, UserPk,
};
use si_id::{ChangeSetApprovalId, EntityId, WorkspaceId};

use crate::reference::{Reference, ReferenceId, ReferenceKind};

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetRequest {
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetResponse {
    pub change_set: ChangeSet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetApprovals {
    pub requirements: Vec<ChangeSetApprovalRequirement>,
    pub latest_approvals: Vec<ChangeSetApproval>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetApprovalRequirement {
    // What is the ID of the entity that is requiring approvals?
    pub entity_id: EntityId,
    // What is the kind of the entity corresponding to the ID?
    pub entity_kind: EntityKind,
    // What is the minimum number needed?
    pub required_count: usize,
    // Is it satisfied?
    pub is_satisfied: bool,
    // Which approvals are for this requirement?
    pub applicable_approval_ids: Vec<ChangeSetApprovalId>,
    // What groups can approve this?
    pub approver_groups: HashMap<String, Vec<UserPk>>,
    // What individuals can approve this?
    pub approver_individuals: Vec<UserPk>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetApproval {
    // What approval is this?
    pub id: ChangeSetApprovalId,
    // Who approved this?
    pub user_id: UserPk,
    // What kind of approval did they do (including negative)?
    pub status: ChangeSetApprovalStatus,
    // Is this still valid?
    pub is_valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetRecord {
    pub name: String,
    pub id: ChangeSetId,
    pub status: ChangeSetStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub base_change_set_id: Option<ChangeSetId>,
    pub workspace_id: WorkspaceId,
    pub merge_requested_by_user_id: Option<String>,
    pub merge_requested_by_user: Option<String>,
    pub merge_requested_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChangeSetList {
    pub name: String,
    pub id: WorkspaceId,
    pub default_change_set_id: ChangeSetId,
    pub change_sets: Vec<Reference<ChangeSetId>>,
}

// XXX: Should probably be `impl From<ComponentNodeWeight> for Reference<ComponentId> {...}`
//      whenever possible, since we can use the merkle tree hash as the checksum.
impl From<ChangeSetRecord> for Reference<ChangeSetId> {
    fn from(value: ChangeSetRecord) -> Self {
        let mut hasher = ChecksumHasher::new();
        hasher.update(value.name.as_bytes());
        hasher.update(value.id.to_string().as_bytes());
        hasher.update(value.status.to_string().as_bytes());
        hasher.update(value.created_at.to_rfc3339().as_bytes());
        hasher.update(value.updated_at.to_rfc3339().as_bytes());
        hasher.update(
            &value
                .base_change_set_id
                .map(|id| id.to_string().as_bytes().to_owned())
                .unwrap_or_else(|| vec![0]),
        );
        hasher.update(value.workspace_id.to_string().as_bytes());
        hasher.update(
            &value
                .merge_requested_by_user_id
                .map(|id| id.as_bytes().to_owned())
                .unwrap_or_else(|| vec![0]),
        );
        hasher.update(
            &value
                .merge_requested_by_user
                .map(|u| u.as_bytes().to_owned())
                .unwrap_or_else(|| vec![0]),
        );
        hasher.update(
            &value
                .merge_requested_at
                .map(|dt| dt.to_rfc3339().as_bytes().to_owned())
                .unwrap_or_else(|| vec![0]),
        );

        let checksum = hasher.finalize().to_string();

        Reference {
            kind: ReferenceKind::ChangeSetRecord,
            id: ReferenceId(value.id),
            checksum,
        }
    }
}
