use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::server::service::change_set::ChangeSetError;
use axum::Json;
use dal::{EditSession, EditSessionPk};
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
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveEditSessionRequest>,
) -> ChangeSetResult<Json<SaveEditSessionResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let mut edit_session = EditSession::get_by_pk(&ctx, &request.edit_session_pk)
        .await?
        .ok_or(ChangeSetError::EditSessionNotFound)?;
    edit_session.save(&ctx).await?;

    txns.commit().await?;

    Ok(Json(SaveEditSessionResponse { edit_session }))
}
