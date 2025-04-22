use std::collections::HashMap;

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetApprovalStatus,
    ChangeSetId,
    ChangeSetStatus,
    UserPk,
    workspace_snapshot::{
        Checksum,
        ChecksumHasher,
        EntityKind,
    },
};
use si_id::{
    ChangeSetApprovalId,
    EntityId,
    WorkspaceId,
};

use crate::{
    checksum::FrontendChecksum,
    object::FrontendObject,
    reference::{
        Refer,
        Reference,
        ReferenceId,
        ReferenceKind,
    },
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
#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
    si_frontend_types_macros::Refer,
)]
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
#[derive(
    Debug,
    Clone,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_types_macros::FrontendChecksum,
    si_frontend_types_macros::FrontendObject,
)]
pub struct ChangeSetList {
    pub name: String,
    pub id: WorkspaceId,
    pub default_change_set_id: ChangeSetId,
    pub change_sets: Vec<Reference<ChangeSetId>>,
}

#[allow(dead_code)]
fn example() -> Result<FrontendObject, serde_json::Error> {
    let ulid = si_id::ulid::Ulid::new();

    // Pretend we retrieved the `ChangeSetRecord` materialized views
    // for the Change Sets we're interested in.
    let change_set_records = [
        ChangeSetRecord {
            name: "Base".to_string(),
            id: ulid.into(),
            status: ChangeSetStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            base_change_set_id: None,
            workspace_id: ulid.into(),
            merge_requested_by_user_id: None,
            merge_requested_by_user: None,
            merge_requested_at: None,
        },
        ChangeSetRecord {
            name: "Feature 1".to_string(),
            id: ulid.into(),
            status: ChangeSetStatus::Open,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            base_change_set_id: Some(ulid.into()),
            workspace_id: ulid.into(),
            merge_requested_by_user_id: None,
            merge_requested_by_user: None,
            merge_requested_at: None,
        },
    ];

    let change_set_list = ChangeSetList {
        name: "Workspace Name".to_string(),
        id: ulid.into(),
        default_change_set_id: ulid.into(),
        change_sets: change_set_records.iter().map(Into::into).collect(),
    };

    change_set_list.try_into()
}
