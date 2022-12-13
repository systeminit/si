use axum::{extract::Query, Json};
use dal::{
    status::{StatusUpdateData, StatusUpdatePk},
    ChangeSetPk, StatusUpdate, Visibility,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::StatusResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListActiveStatusesRequest {
    pub change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ActiveStatus {
    pub pk: StatusUpdatePk,
    pub data: StatusUpdateData,
}
pub type ListActiveStatusesResponse = Vec<ActiveStatus>;

pub async fn list_active_statuses(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListActiveStatusesRequest>,
) -> StatusResult<Json<ListActiveStatusesResponse>> {
    let visibility = Visibility::new_change_set(request.change_set_pk, false);
    let ctx = builder.build(request_ctx.build(visibility)).await?;

    let list = StatusUpdate::list_active(&ctx)
        .await?
        .into_iter()
        .map(|status_update| ActiveStatus {
            pk: status_update.pk,
            data: status_update.data,
        })
        .collect();

    Ok(Json(list))
}
