use axum::extract::{OriginalUri, Query};
use axum::Json;
use dal::{qualification::QualificationView, Component, ComponentId, Visibility};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQualificationsRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type QualificationResponse = Vec<QualificationView>;

pub async fn list_qualifications(
    OriginalUri(original_uri): OriginalUri,
    PosthogClient(posthog_client): PosthogClient,
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListQualificationsRequest>,
) -> ComponentResult<Json<QualificationResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let qualifications = Component::list_qualifications(&ctx, request.component_id).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_qualifications",
        serde_json::json!({
            "how": "/component/list_qualifications",
            "component_id": request.component_id.clone(),
            "qualifications": qualifications.clone().len(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(Json(qualifications))
}
