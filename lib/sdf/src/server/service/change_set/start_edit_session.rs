use super::ChangeSetResult;
use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};
use axum::Json;
use chrono::Utc;
use dal::{ChangeSetPk, EditSession, HistoryActor, WriteTenancy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartEditSessionRequest {
    pub change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartEditSessionResponse {
    pub edit_session: EditSession,
}

pub async fn start_edit_session(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<StartEditSessionRequest>,
) -> ChangeSetResult<Json<StartEditSessionResponse>> {
    dbg!("motherfucker");
    let txn = txn.start().await?;
    let nats = nats.start().await?;
    let write_tenancy = WriteTenancy::new_billing_account(claim.billing_account_id);
    let history_actor: HistoryActor = HistoryActor::from(claim.user_id);

    let current_date_time = Utc::now();
    let edit_session_name = current_date_time.to_string();
    let edit_session = EditSession::new(
        &txn,
        &nats,
        &write_tenancy,
        &history_actor,
        &request.change_set_pk,
        &edit_session_name,
        None,
    )
    .await?;
    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(StartEditSessionResponse { edit_session }))
}
