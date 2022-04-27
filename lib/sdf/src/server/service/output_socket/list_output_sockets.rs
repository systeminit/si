use axum::extract::Query;
use axum::Json;
use dal::socket::output::OutputSocket;
use dal::{SchemaVariantId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::output_socket::OutputSocketResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListOutputSocketRequest {
    pub schema_variant_id: SchemaVariantId,
    pub workspace_id: Option<WorkspaceId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListOutputSocketResponse {
    pub output_sockets: Vec<OutputSocket>,
}

pub async fn list_output_sockets(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListOutputSocketRequest>,
) -> OutputSocketResult<Json<ListOutputSocketResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let output_sockets =
        OutputSocket::list_for_schema_variant(&ctx, request.schema_variant_id).await?;

    let response = ListOutputSocketResponse { output_sockets };
    Ok(Json(response))
}
