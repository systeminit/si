use axum::Json;
use dal::{ChangeSet, ChangeSetPk, EditSession, EditSessionPk};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext};

use super::{ChangeSetError, ChangeSetResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveEditSessionAndApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPk,
    pub edit_session_pk: EditSessionPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveEditSessionAndApplyChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn save_edit_session_and_apply_change_set(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveEditSessionAndApplyChangeSetRequest>,
) -> ChangeSetResult<Json<SaveEditSessionAndApplyChangeSetResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let mut edit_session = EditSession::get_by_pk(&ctx, &request.edit_session_pk)
        .await?
        .ok_or(ChangeSetError::EditSessionNotFound)?;
    edit_session.save(&ctx).await?;

    let mut change_set = ChangeSet::get_by_pk(&ctx, &request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.apply(&ctx).await?;

    txns.commit().await?;

    Ok(Json(SaveEditSessionAndApplyChangeSetResponse {
        change_set,
    }))
}
