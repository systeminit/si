use axum::Json;
use dal::node_menu::{get_node_menu_items, GenerateMenuItem};
use dal::{MenuFilter, ReadTenancy, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::{SchematicError, SchematicResult};
use crate::server::extract::{Authorization, PgRwTxn};

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
    mut txn: PgRwTxn,
    Authorization(claim): Authorization,
    Json(request): Json<GetNodeAddMenuRequest>,
) -> SchematicResult<Json<GetNodeAddMenuResponse>> {
    let txn = txn.start().await?;
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![request.workspace_id]).await?;
    if !read_tenancy
        .billing_accounts()
        .contains(&claim.billing_account_id)
    {
        return Err(SchematicError::NotAuthorized);
    }
    let items = get_node_menu_items(
        &txn,
        &read_tenancy,
        &request.visibility,
        &request.menu_filter,
    )
    .await?;
    let response = {
        let gmi = GenerateMenuItem::new();
        gmi.create_menu_json(items)?
    };
    Ok(Json(response))
}
