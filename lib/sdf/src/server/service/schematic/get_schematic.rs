use axum::{extract::Query, Json};
use dal::{Schema, SchemaId, StandardModel, Tenancy, Visibility};
use serde::{Deserialize, Serialize};

use super::{SchematicResult};
use crate::server::extract::{Authorization, PgRoTxn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchematicRequest {
    pub context: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSchematicResponse = serde_json::Value;

pub async fn get_schematic(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
    Query(request): Query<GetSchematicRequest>,
) -> SchematicResult<Json<GetSchematicResponse>> {
    let response = serde_json::json![{ "poop": "canoe" }];
    Ok(Json(response))
}
