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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetChangeSetRequest>,
) -> ChangeSetResult<Json<GetChangeSetResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let change_set = ChangeSet::get_by_pk(&ctx, &request.pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    txns.commit().await?;

    Ok(Json(GetChangeSetResponse { change_set }))
}
