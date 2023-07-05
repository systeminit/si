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
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Authorization(_claim): Authorization,
    Json(request): Json<UpdateSelectedChangeSetRequest>,
) -> ChangeSetResult<Json<UpdateSelectedChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set = ChangeSet::get_by_pk(&ctx, &request.next_change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    Ok(Json(UpdateSelectedChangeSetResponse { change_set }))
}
