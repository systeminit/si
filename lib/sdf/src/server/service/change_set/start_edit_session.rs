use super::ChangeSetResult;
use crate::server::extract::{Authorization, HandlerContext, Tenancy};
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
    HandlerContext(builder, mut txns): HandlerContext,
    Authorization(claim): Authorization,
    Tenancy(_write_tenancy, read_tenancy): Tenancy,
    Json(request): Json<StartEditSessionRequest>,
) -> ChangeSetResult<Json<StartEditSessionResponse>> {
    dbg!("motherfucker");
    let txns = txns.start().await?;
    let ctx = builder.build(
        dal::context::AccessBuilder::new(
            read_tenancy,
            WriteTenancy::new_billing_account(claim.billing_account_id),
            HistoryActor::User(claim.user_id),
        )
        .build_head(),
        &txns,
    );

    let current_date_time = Utc::now();
    let edit_session_name = current_date_time.to_string();
    let edit_session = EditSession::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.history_actor(),
        &request.change_set_pk,
        &edit_session_name,
        None,
    )
    .await?;

    txns.commit().await?;

    Ok(Json(StartEditSessionResponse { edit_session }))
}
