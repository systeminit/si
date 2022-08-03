use axum::extract::Query;
use axum::Json;
use serde::{Deserialize, Serialize};

use dal::qualification::QualificationSummary;
use dal::Visibility;

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::qualification::QualificationResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSummaryRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSummaryResponse = QualificationSummary;

pub async fn get_summary(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetSummaryRequest>,
) -> QualificationResult<Json<GetSummaryResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let qual_summary = QualificationSummary::get_summary(&ctx).await?;

    txns.commit().await?;

    Ok(Json(qual_summary))
}
