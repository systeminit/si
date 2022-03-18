use super::{ChangeSetError, ChangeSetResult};
use crate::server::extract::{Authorization, PgRoTxn};
use axum::extract::Query;
use axum::Json;
use dal::{ChangeSet, ChangeSetPk, ReadTenancy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetRequest {
    pub pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn get_change_set(
    mut txn: PgRoTxn,
    Query(request): Query<GetChangeSetRequest>,
    Authorization(claim): Authorization,
) -> ChangeSetResult<Json<GetChangeSetResponse>> {
    let txn = txn.start().await?;
    let read_tenancy = ReadTenancy::new_billing_account(vec![claim.billing_account_id]);
    let change_set = ChangeSet::get_by_pk(&txn, &read_tenancy, &request.pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    txn.commit().await?;
    Ok(Json(GetChangeSetResponse { change_set }))
}
