use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::server::service::change_set::ChangeSetError;
use axum::Json;
use dal::{ChangeSet, ChangeSetPk};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.apply(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(ApplyChangeSetResponse { change_set }))
}
