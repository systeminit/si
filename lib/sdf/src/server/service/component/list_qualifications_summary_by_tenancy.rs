use axum::extract::Query;
use axum::Json;
use serde::{Deserialize, Serialize};

use dal::component::QualificationSummary;
use dal::{Component, Visibility, WorkspaceId};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQualificationsRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn list_qualifications_summary_by_tenancy(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListQualificationsRequest>,
) -> ComponentResult<Json<QualificationSummary>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let qual_summary = Component::list_qualifications_summary_by_tenancy(&ctx).await?;

    txns.commit().await?;

    Ok(Json(qual_summary))
}
