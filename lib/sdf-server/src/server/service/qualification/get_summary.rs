use axum::extract::Query;
use axum::Json;
use serde::{Deserialize, Serialize};

use dal::qualification::{QualificationSummary, QualificationSummaryForComponent};
use dal::{ComponentId, Visibility};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::qualification::QualificationResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSummaryRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationSummaryForComponentResponse {
    component_id: ComponentId,
    component_name: String,
    total: i64,
    warned: i64,
    succeeded: i64,
    failed: i64,
}

impl From<QualificationSummaryForComponent> for QualificationSummaryForComponentResponse {
    fn from(q: QualificationSummaryForComponent) -> Self {
        QualificationSummaryForComponentResponse {
            component_id: q.component_id,
            component_name: q.component_name,
            total: q.total,
            warned: q.warned,
            succeeded: q.succeeded,
            failed: q.failed,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationSummaryResponse {
    total: i64,
    succeeded: i64,
    warned: i64,
    failed: i64,
    components: Vec<QualificationSummaryForComponentResponse>,
}

impl From<QualificationSummary> for QualificationSummaryResponse {
    fn from(s: QualificationSummary) -> Self {
        QualificationSummaryResponse {
            total: s.total,
            succeeded: s.succeeded,
            warned: s.warned,
            failed: s.failed,
            components: s.components.into_iter().map(|c| c.into()).collect(),
        }
    }
}

pub async fn get_summary(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetSummaryRequest>,
) -> QualificationResult<Json<QualificationSummaryResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let qual_summary = QualificationSummary::get_summary(&ctx).await?;

    Ok(Json(qual_summary.into()))
}
