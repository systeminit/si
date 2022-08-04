use axum::Json;
use chrono::Utc;
use dal::{ChangeSetPk, EditSession, EditSessionPk, WriteTenancy};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

use super::{ChangeSetError, ChangeSetResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CancelAndStartEditSessionRequest {
    pub change_set_pk: ChangeSetPk,
    pub edit_session_pk: EditSessionPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CancelAndStartEditSessionResponse {
    pub edit_session: EditSession,
}

pub async fn cancel_and_start_edit_session(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
    Json(request): Json<CancelAndStartEditSessionRequest>,
) -> ChangeSetResult<Json<CancelAndStartEditSessionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let mut edit_session = EditSession::get_by_pk(&ctx, &request.edit_session_pk)
        .await?
        .ok_or(ChangeSetError::EditSessionNotFound)?;
    edit_session.cancel(&ctx).await?;

    let ctx = ctx.clone_with_new_tenancies(
        ctx.read_tenancy().clone(),
        WriteTenancy::new_billing_account(claim.billing_account_id),
    );

    let current_date_time = Utc::now();
    let edit_session_name = current_date_time.to_string();
    let edit_session =
        EditSession::new(&ctx, &request.change_set_pk, &edit_session_name, None).await?;

    txns.commit().await?;

    Ok(Json(CancelAndStartEditSessionResponse { edit_session }))
}
