use super::ChangeSetResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use crate::server::service::change_set::ChangeSetError;
use axum::Json;
use dal::{ChangeSet, ChangeSetPk, HistoryActor, Tenancy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn apply_change_set(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);
    let mut change_set = ChangeSet::get_by_pk(&txn, &tenancy, &request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.apply(&txn, &nats, &history_actor).await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(ApplyChangeSetResponse { change_set }))
}
