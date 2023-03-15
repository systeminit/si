use axum::Json;
use dal::edge::EdgeId;
use dal::{Connection, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConnectionRequest {
    pub edge_id: EdgeId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a [`Connection`](dal::Connection) via its EdgeId.
pub async fn delete_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<DeleteConnectionRequest>,
) -> DiagramResult<()> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    Connection::delete_for_edge(&ctx, request.edge_id).await?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}
