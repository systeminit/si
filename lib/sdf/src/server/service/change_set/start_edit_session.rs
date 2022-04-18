use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};
use axum::Json;
use chrono::Utc;
use dal::{ChangeSetPk, EditSession, WriteTenancy};
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
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
    Json(request): Json<StartEditSessionRequest>,
) -> ChangeSetResult<Json<StartEditSessionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);
    let ctx = ctx.clone_with_new_tenancies(
        ctx.read_tenancy().clone(),
        WriteTenancy::new_billing_account(claim.billing_account_id),
    );

    let current_date_time = Utc::now();
    let edit_session_name = current_date_time.to_string();
    let edit_session =
        EditSession::new(&ctx, &request.change_set_pk, &edit_session_name, None).await?;

    txns.commit().await?;

    Ok(Json(StartEditSessionResponse { edit_session }))
}
