use axum::extract::Query;
use axum::Json;
use dal::{qualification::QualificationView, Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQualificationsRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type QualificationResponse = Vec<QualificationView>;

pub async fn list_qualifications(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListQualificationsRequest>,
) -> ComponentResult<Json<QualificationResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let qualifications = Component::list_qualifications(&ctx, request.component_id).await?;

    Ok(Json(qualifications))
}
