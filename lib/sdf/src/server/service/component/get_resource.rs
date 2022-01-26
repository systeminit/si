use axum::extract::Query;
use axum::Json;
use dal::{ComponentId, SystemId, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{Authorization, PgRoTxn};
use chrono::Utc;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetResourceResponse {
    pub resource: serde_json::Value,
}

pub async fn get_resource(
    mut txn: PgRoTxn,
    Query(_request): Query<GetResourceRequest>,
    Authorization(_claim): Authorization,
) -> ComponentResult<Json<GetResourceResponse>> {
    let txn = txn.start().await?;

    let resource = serde_json::json!({
        "id": "-1",
        "timestamp": u64::try_from(std::cmp::max(Utc::now().timestamp(), 0)).expect("Timestamp will never be negative").to_string(),
        "error": "Boto Cor de Rosa Spotted",
        "data": { "Saci-Pererê": { "Its just a prank bro": 3 } },
        "health": "warning",
        "entityType": "Eat Acarajé with Shrimps & Vatapa & Caruru & a lot of hot sauce",
    });
    txn.commit().await?;
    Ok(Json(GetResourceResponse { resource }))
}
