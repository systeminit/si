use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ChangeSetId,
    ChangeSetStatus,
};
use si_id::WorkspaceId;

use crate::{
    object::FrontendObject,
    reference::Reference,
};

// Data view for the frontend.
#[derive(
    Debug,
    Clone,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
    si_frontend_mv_types_macros::Refer,
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
    si_frontend_mv_types_macros::FrontendChecksum,
    si_frontend_mv_types_macros::FrontendObject,
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
