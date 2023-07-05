use super::{ChangeSetError, ChangeSetResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::extract::Query;
use axum::Json;
use dal::{ChangeSet, ChangeSetPk};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetRequest {
    pub pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn get_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Query(request): Query<GetChangeSetRequest>,
) -> ChangeSetResult<Json<GetChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set = ChangeSet::get_by_pk(&ctx, &request.pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    ctx.commit().await?;

    Ok(Json(GetChangeSetResponse { change_set }))
}
