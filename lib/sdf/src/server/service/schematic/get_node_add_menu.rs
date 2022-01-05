use axum::Json;
use dal::{MenuFilter, Visibility};
use serde::{Deserialize, Serialize};

use super::SchematicResult;
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeAddMenuRequest {
    pub menu_filter: MenuFilter,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetNodeAddMenuResponse = serde_json::Value;

pub async fn get_node_add_menu(
    _txn: PgRoTxn,
    Authorization(_claim): Authorization,
    Json(_request): Json<GetNodeAddMenuRequest>,
) -> SchematicResult<Json<GetNodeAddMenuResponse>> {
    let response = serde_json::json![
     [
       {
         "kind": "category",
         "name": "Snoopy",
         "items": [
           {
             "kind": "item",
             "name": "floopy",
             "entityType": "floopy",
           },
         ],
       },
     ]
    ];
    Ok(Json(response))
}
