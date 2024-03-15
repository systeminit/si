use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::change_set_pointer::{
    ChangeSetId, ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerResult,
};
use crate::{ChangeSetStatus, DalContext, UserPk};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenChangeSetsView {
    pub head_change_set_id: ChangeSetId,
    pub change_sets: Vec<ChangeSetView>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetView {
    pub id: ChangeSetId,
    pub name: String,
    pub status: ChangeSetStatus,
    pub merge_requested_at: Option<DateTime<Utc>>,
    pub base_change_set_id: Option<ChangeSetId>,
    pub merge_requested_by_user_id: Option<UserPk>,
    pub abandon_requested_at: Option<DateTime<Utc>>,
    pub abandon_requested_by_user_id: Option<UserPk>,
}

impl OpenChangeSetsView {
    pub async fn assemble(ctx: &DalContext) -> ChangeSetPointerResult<Self> {
        // List all open change sets and assemble them into individual views.
        let open_change_sets = ChangeSetPointer::list_open(ctx).await?;
        let mut views = Vec::with_capacity(open_change_sets.len());
        for change_set in open_change_sets {
            views.push(ChangeSetView {
                id: change_set.id,
                name: change_set.name,
                status: change_set.status,
                base_change_set_id: change_set.base_change_set_id,
                merge_requested_at: None,           // cs.merge_requested_at,
                merge_requested_by_user_id: None,   // cs.merge_requested_by_user_id,
                abandon_requested_at: None,         // cs.abandon_requested_at,
                abandon_requested_by_user_id: None, // cs.abandon_requested_by_user_id,
            });
        }

        // Ensure that we find exactly one change set view that matches the open change sets found.
        let head_change_set_id = ctx.get_workspace_default_change_set_id().await?;
        let maybe_head_change_set_id: Vec<ChangeSetId> = views
            .iter()
            .filter_map(|v| {
                if v.id == head_change_set_id {
                    Some(v.id)
                } else {
                    None
                }
            })
            .collect();
        if maybe_head_change_set_id.len() != 1 {
            return Err(
                ChangeSetPointerError::UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(
                    maybe_head_change_set_id,
                ),
            );
        }

        Ok(Self {
            head_change_set_id,
            change_sets: views,
        })
    }
}
