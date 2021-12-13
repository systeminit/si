use super::SchematicResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use axum::Json;
use dal::Visibility;
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
    mut _txn: PgRwTxn,
    mut _nats: NatsTxn,
    Authorization(_claim): Authorization,
    Json(_request): Json<SetSchematicRequest>,
) -> SchematicResult<Json<SetSchematicResponse>> {
    let response = serde_json::json!({ "poop": "is set"});
    Ok(Json(response))
}
