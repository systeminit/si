use axum::Json;
use dal::node_menu::GenerateMenuItem;
use dal::{DiagramKind, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeAddMenuRequest {
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetNodeAddMenuResponse = serde_json::Value;

pub async fn get_node_add_menu(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<GetNodeAddMenuRequest>,
) -> DiagramResult<Json<GetNodeAddMenuResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // NOTE(nick): return only components at the moment.
    let gmi = GenerateMenuItem::new(&ctx, DiagramKind::Configuration).await?;
    let response = gmi.create_menu_json()?;

    Ok(Json(response))
}
