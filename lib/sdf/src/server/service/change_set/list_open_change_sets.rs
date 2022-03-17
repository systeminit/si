use super::ChangeSetResult;
use crate::server::extract::{Authorization, PgRwTxn};
use axum::Json;
use dal::{ChangeSet, ChangeSetPk, LabelList, ReadTenancy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListOpenChangeSetsResponse {
    pub list: LabelList<ChangeSetPk>,
}

pub async fn list_open_change_sets(
    mut txn: PgRwTxn,
    Authorization(claim): Authorization,
) -> ChangeSetResult<Json<ListOpenChangeSetsResponse>> {
    let txn = txn.start().await?;
    let read_tenancy = ReadTenancy::new_billing_account(vec![claim.billing_account_id]);
    let list = ChangeSet::list_open(&txn, &read_tenancy).await?;
    Ok(Json(ListOpenChangeSetsResponse { list }))
}
