use axum::extract::Query;
use axum::Json;
use dal::{
    qualification::QualificationView, qualification_resolver::UNSET_ID_VALUE, Component,
    ComponentId, Visibility, WorkspaceId,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQualificationsRequest {
    pub component_id: ComponentId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type QualificationResponse = Vec<QualificationView>;

pub async fn list_qualifications(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListQualificationsRequest>,
) -> ComponentResult<Json<QualificationResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let qualifications = Component::list_qualifications_by_component_id(
        &ctx,
        request.component_id,
        UNSET_ID_VALUE.into(),
    )
    .await?;

    txns.commit().await?;

    Ok(Json(qualifications))
}
