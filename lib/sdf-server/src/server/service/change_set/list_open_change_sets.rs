use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use chrono::{DateTime, Utc};
use dal::{ChangeSet, ChangeSetPk, ChangeSetStatus, UserPk};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetView {
    pub pk: ChangeSetPk,
    pub name: String,
    pub status: ChangeSetStatus,
    pub merge_requested_at: Option<DateTime<Utc>>,
    pub merge_requested_by_user_id: Option<UserPk>,
    pub abandon_requested_at: Option<DateTime<Utc>>,
    pub abandon_requested_by_user_id: Option<UserPk>,
}

pub type ListOpenChangeSetsResponse = Vec<ChangeSetView>;

pub async fn list_open_change_sets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> ChangeSetResult<Json<ListOpenChangeSetsResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let list = ChangeSet::list_open(&ctx).await?;
    let mut view = Vec::with_capacity(list.len());
    for cs in list {
        view.push(ChangeSetView {
            pk: cs.pk,
            name: cs.name,
            status: cs.status,
            merge_requested_at: cs.merge_requested_at,
            merge_requested_by_user_id: cs.merge_requested_by_user_id,
            abandon_requested_at: cs.abandon_requested_at,
            abandon_requested_by_user_id: cs.abandon_requested_by_user_id,
        });
    }

    Ok(Json(view))
}
