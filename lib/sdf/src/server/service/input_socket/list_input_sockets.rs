use axum::extract::Query;
use axum::Json;
use dal::socket::input::InputSocket;
use dal::{SchemaVariantId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::input_socket::InputSocketResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListInputSocketRequest {
    pub schema_variant_id: SchemaVariantId,
    pub workspace_id: Option<WorkspaceId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListInputSocketResponse {
    pub input_sockets: Vec<InputSocket>,
}

pub async fn list_input_sockets(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListInputSocketRequest>,
) -> InputSocketResult<Json<ListInputSocketResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let input_sockets =
        InputSocket::list_for_schema_variant(&ctx, request.schema_variant_id).await?;

    let response = ListInputSocketResponse { input_sockets };
    Ok(Json(response))
}
