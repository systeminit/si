use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{
    workspace_snapshot::{Checksum, ChecksumHasher, EntityKind},
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

// Data view for the frontend.
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

// Data view for the frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ChangeSetList {
    pub name: String,
    pub id: WorkspaceId,
    pub default_change_set_id: ChangeSetId,
    pub change_sets: Vec<Reference<ChangeSetId>>,
}

// Convenience for doing things like:
//  `let refs: Vec<Reference<ChangeSetId>> = change_set_records.iter().map(Into::into).collect();`
impl From<ChangeSetRecord> for Reference<ChangeSetId> {
    fn from(value: ChangeSetRecord) -> Self {
        let checksum = FrontendChecksum::checksum(&value).to_string();

        Reference {
            kind: ReferenceKind::ChangeSetRecord,
            id: ReferenceId(value.id),
            checksum,
        }
    }
}

pub trait FrontendChecksum {
    fn checksum(&self) -> Checksum;
}

// Should be very derivable for any of the frontend data view structs.
impl FrontendChecksum for ChangeSetList {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.default_change_set_id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.change_sets).as_bytes());

        hasher.finalize()
    }
}

// Should be very derivable for any of the frontend data view structs.
impl FrontendChecksum for ChangeSetRecord {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.name).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.status).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.created_at).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.updated_at).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.base_change_set_id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.workspace_id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.merge_requested_by_user_id).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.merge_requested_by_user).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.merge_requested_at).as_bytes());

        hasher.finalize()
    }
}

// Would be nice to do this automatically as part of the macros. As an impl for a trait
// seems difficult to work around "conflicting implementations for trait" errors with
// the other trait impls for the more basic types.
impl FrontendChecksum for ChangeSetId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for ChangeSetStatus {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

impl FrontendChecksum for WorkspaceId {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_string())
    }
}

// Generic impl for a basic type.
impl FrontendChecksum for String {
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        hasher.update(self.as_bytes());
        hasher.finalize()
    }
}

impl<T> FrontendChecksum for Option<T>
where
    T: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        if let Some(inner) = self {
            inner.checksum()
        } else {
            Checksum::default()
        }
    }
}

impl<T> FrontendChecksum for Vec<T>
where
    T: FrontendChecksum,
{
    fn checksum(&self) -> Checksum {
        let mut hasher = ChecksumHasher::new();
        for item in self {
            hasher.update(item.checksum().to_string().as_bytes());
        }
        hasher.finalize()
    }
}

impl FrontendChecksum for DateTime<Utc> {
    fn checksum(&self) -> Checksum {
        FrontendChecksum::checksum(&self.to_rfc3339())
    }
}

// Payload wrapper for sending data views to the frontend.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FrontendObject {
    pub kind: String,
    pub id: String,
    pub checksum: Checksum,
    pub data: serde_json::Value,
}

// Very derivable for any of the frontend data view structs.
impl TryFrom<ChangeSetRecord> for FrontendObject {
    type Error = serde_json::Error;

    fn try_from(value: ChangeSetRecord) -> Result<Self, Self::Error> {
        let kind = "ChangeSetRecord".to_string();
        let id = value.id.to_string();
        let checksum = FrontendChecksum::checksum(&value);
        let data = serde_json::to_value(value)?;

        Ok(FrontendObject {
            kind,
            id,
            checksum,
            data,
        })
    }
}
