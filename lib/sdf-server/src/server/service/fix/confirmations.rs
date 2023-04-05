use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::component::confirmation::view::ConfirmationView as DalConfirmationView;
use dal::{Component, ComponentId, Visibility};
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

pub type ConfirmationsResponse = Vec<DalConfirmationView>;

pub async fn confirmations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<ConfirmationsRequest>,
) -> FixResult<Json<ConfirmationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let confirmation_views = Component::list_confirmations(&ctx).await?;

    let recommendations = confirmation_views
        .iter()
        .flat_map(|confirmation| {
            confirmation
                .recommendations
                .iter()
                .map(|rec| (rec.name.clone(), rec.component_id))
                .collect::<Vec<(String, ComponentId)>>()
        })
        .collect::<Vec<(String, ComponentId)>>();

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_confirmations",
        serde_json::json!({
            "number_of_suggested_recommendations": recommendations.len(),
            "recommendations_list": recommendations
        }),
    );

    Ok(Json(confirmation_views))
}
