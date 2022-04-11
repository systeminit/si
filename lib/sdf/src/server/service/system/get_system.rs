use axum::{extract::Query, Json};
use dal::{StandardModel, System, SystemId, Visibility};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{SystemError, SystemResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSystemRequest {
    pub system_id: SystemId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSystemResponse {
    pub system: System,
}

pub async fn get_system(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetSystemRequest>,
) -> SystemResult<Json<GetSystemResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let system = System::get_by_id(&ctx, &request.system_id)
        .await?
        .ok_or(SystemError::SystemNotFound)?;

    Ok(Json(GetSystemResponse { system }))
}
