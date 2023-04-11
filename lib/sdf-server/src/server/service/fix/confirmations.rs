use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::component::confirmation::view::ConfirmationView as DalConfirmationView;
use dal::component::confirmation::view::RecommendationView as DalRecommendationView;
use dal::{Component, Visibility};
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationsResponse {
    pub confirmations: Vec<DalConfirmationView>,
    pub recommendations: Vec<DalRecommendationView>,
}

pub async fn confirmations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<ConfirmationsRequest>,
) -> FixResult<Json<ConfirmationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let (confirmation_views, recommendation_views) = Component::list_confirmations(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_confirmations",
        serde_json::json!({
            "number_of_suggested_recommendations": recommendation_views.len(),
            "recommendations_list": recommendation_views
        }),
    );

    Ok(Json(ConfirmationsResponse {
        confirmations: confirmation_views,
        recommendations: recommendation_views,
    }))
}
