use axum::Json;
use dal::node_menu::{get_node_menu_items, GenerateMenuItem};
use dal::{MenuFilter, Tenancy, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
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
    let mut tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    tenancy.workspace_ids = vec![request.workspace_id];
    tenancy.universal = true;
    let items =
        get_node_menu_items(&txn, &tenancy, &request.visibility, &request.menu_filter).await?;
    let response = {
        let gmi = GenerateMenuItem::new();
        gmi.create_menu_json(items)?
    };
    Ok(Json(response))
}
