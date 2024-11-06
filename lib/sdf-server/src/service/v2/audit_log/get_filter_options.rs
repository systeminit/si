use axum::{extract::Path, Json};
use dal::{ChangeSet, User};
use serde::Serialize;
use si_events::{ChangeSetId, UserPk};

use super::AuditLogResult;
use crate::extract::{AccessBuilder, HandlerContext};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangeSetFilterOption {
    pub id: ChangeSetId,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserFilterOption {
    pub id: UserPk,
    pub name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetFilterOptionsResponse {
    change_sets: Vec<ChangeSetFilterOption>,
    users: Vec<UserFilterOption>,
}

pub async fn get_filter_options(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((workspace_pk, change_set_id)): Path<(dal::WorkspacePk, dal::ChangeSetId)>,
) -> AuditLogResult<Json<GetFilterOptionsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let change_sets = ChangeSet::list_all_for_workspace_audit_trail(&ctx, workspace_pk)
        .await?
        .iter()
        .map(|c| ChangeSetFilterOption {
            id: c.id.into(),
            name: c.name.to_owned(),
        })
        .collect();
    let users = User::list_members_for_workspace(&ctx, workspace_pk.to_string())
        .await?
        .iter()
        .map(|u| UserFilterOption {
            id: u.pk().into(),
            name: u.name().to_owned(),
        })
        .collect();

    Ok(Json(GetFilterOptionsResponse { change_sets, users }))
}
