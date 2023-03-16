use axum::Json;
use dal::ChangeSet;
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

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

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    let change_set = ChangeSet::new(&ctx, request.change_set_name, None).await?;

    ctx.commit().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}
