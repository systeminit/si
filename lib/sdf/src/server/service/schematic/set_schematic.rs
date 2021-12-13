use super::SchematicResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use axum::Json;
use dal::{HistoryActor, Schema, SchemaKind, Tenancy, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetSchematicRequest {
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type SetSchematicResponse = serde_json::Value;

pub async fn set_schematic(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<SetSchematicRequest>,
) -> SchematicResult<Json<SetSchematicResponse>> {
    let response = serde_json::json!({ "poop": "is set"});
    Ok(Json(response))
}
