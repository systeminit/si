use super::SchemaResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use axum::Json;
use dal::{component::ComponentKind, HistoryActor, Schema, SchemaKind, Tenancy, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaRequest {
    pub name: String,
    pub kind: SchemaKind,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSchemaResponse {
    pub schema: Schema,
}

pub async fn create_schema(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<CreateSchemaRequest>,
) -> SchemaResult<Json<CreateSchemaResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let schema = Schema::new(
        &txn,
        &nats,
        &tenancy,
        &request.visibility,
        &history_actor,
        &request.name,
        &request.kind,
        &ComponentKind::Standard,
    )
    .await?;
    txn.commit().await?;
    nats.commit().await?;
    let response = CreateSchemaResponse { schema };
    Ok(Json(response))
}
