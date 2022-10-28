use axum::{extract::Query, Json};
use dal::fix::recommendation::Recommendation;
use dal::Visibility;
use serde::{Deserialize, Serialize};

use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type RecommendationsResponse = Vec<Recommendation>;

pub async fn recommendations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<RecommendationsRequest>,
) -> FixResult<Json<RecommendationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;
    let recommendations = Recommendation::list(&ctx).await?;
    Ok(Json(recommendations))
}
