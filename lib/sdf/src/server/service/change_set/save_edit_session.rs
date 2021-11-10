use super::ChangeSetResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use crate::server::service::change_set::ChangeSetError;
use axum::Json;
use dal::{EditSession, EditSessionPk, HistoryActor, Tenancy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveEditSessionRequest {
    pub edit_session_pk: EditSessionPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveEditSessionResponse {
    pub edit_session: EditSession,
}

pub async fn save_edit_session(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<SaveEditSessionRequest>,
) -> ChangeSetResult<Json<SaveEditSessionResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);

    let mut edit_session = EditSession::get_by_pk(&txn, &tenancy, &request.edit_session_pk)
        .await?
        .ok_or(ChangeSetError::EditSessionNotFound)?;
    edit_session.save(&txn, &nats, &history_actor).await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(SaveEditSessionResponse { edit_session }))
}
