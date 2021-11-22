use super::SchemaResult;
use crate::server::extract::{Authorization, PgRoTxn, QueryVisibility};
use axum::Json;
use dal::{Schema, StandardModel, Tenancy, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ListSchemaRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSchemaResponse {
    pub list: Vec<Schema>,
}

pub async fn list_schemas(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
    QueryVisibility(visibility): QueryVisibility,
) -> SchemaResult<Json<ListSchemaResponse>> {
    let txn = txn.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let list = Schema::list(&txn, &tenancy, &visibility).await?;
    let response = ListSchemaResponse { list };
    Ok(Json(response))
}
