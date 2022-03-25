use axum::Json;
use dal::node_menu::{get_node_menu_items, GenerateMenuItem};
use dal::{MenuFilter, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeAddMenuRequest {
    pub menu_filter: MenuFilter,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetNodeAddMenuResponse = serde_json::Value;

pub async fn get_node_add_menu(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<GetNodeAddMenuRequest>,
) -> SchematicResult<Json<GetNodeAddMenuResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let items = get_node_menu_items(
        ctx.pg_txn(),
        ctx.read_tenancy(),
        ctx.visibility(),
        &request.menu_filter,
    )
    .await?;
    let response = {
        let gmi = GenerateMenuItem::new();
        gmi.create_menu_json(items)?
    };
    Ok(Json(response))
}
