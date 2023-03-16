use axum::{extract::Query, Json};
use dal::component::confirmation::view::ConfirmationView as DalConfirmationView;
use dal::{Component, Visibility};
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type ConfirmationsResponse = Vec<DalConfirmationView>;

pub async fn confirmations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ConfirmationsRequest>,
) -> FixResult<Json<ConfirmationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let confirmation_views = Component::list_confirmations(&ctx).await?;
    Ok(Json(confirmation_views))
}
