use axum::extract::Query;
use axum::Json;
use dal::node::NodeTemplate;
use dal::{SchemaId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeTemplateRequest {
    pub schema_id: SchemaId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetNodeTemplateResponse = NodeTemplate;

pub async fn get_node_template(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetNodeTemplateRequest>,
) -> SchematicResult<Json<GetNodeTemplateResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let response = NodeTemplate::new_from_schema_id(&ctx, request.schema_id).await?;

    Ok(Json(response))
}
