use super::{SchemaResult, SchemaError};
use crate::server::extract::{Authorization, NatsTxn, PgRoTxn, QueryVisibility};
use axum::extract::Query;
use axum::Json;
use dal::{HistoryActor, Schema, SchemaId, SchemaKind, StandardModel, Tenancy, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaRequest {
    pub schema_id: SchemaId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type GetSchemaResponse = Schema;

pub async fn get_schema(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
    Query(request): Query<GetSchemaRequest>,
) -> SchemaResult<Json<GetSchemaResponse>> {
    let txn = txn.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let response = Schema::get_by_id(&txn, &tenancy, &request.visibility, &request.schema_id)
        .await?
        .ok_or(SchemaError::SchemaNotFound)?;
    Ok(Json(response))
}
