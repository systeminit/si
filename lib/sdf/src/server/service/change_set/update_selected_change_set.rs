use axum::Json;
use dal::{ChangeSet, ChangeSetPk};
use serde::{Deserialize, Serialize};

use super::{ChangeSetError, ChangeSetResult};
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSelectedChangeSetRequest {
    pub next_change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSelectedChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn update_selected_change_set(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(_claim): Authorization,
    Json(request): Json<UpdateSelectedChangeSetRequest>,
) -> ChangeSetResult<Json<UpdateSelectedChangeSetResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let change_set = ChangeSet::get_by_pk(&ctx, &request.next_change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    txns.commit().await?;

    Ok(Json(UpdateSelectedChangeSetResponse { change_set }))
}
