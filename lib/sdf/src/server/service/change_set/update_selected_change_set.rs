use axum::Json;
use chrono::Utc;
use dal::{ChangeSet, ChangeSetPk, EditSession, EditSessionPk, WriteTenancy};
use serde::{Deserialize, Serialize};

use super::{ChangeSetError, ChangeSetResult};
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSelectedChangeSetRequest {
    pub current_edit_session_pk: Option<EditSessionPk>,
    pub next_change_set_pk: ChangeSetPk,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSelectedChangeSetResponse {
    pub change_set: ChangeSet,
    pub edit_session: EditSession,
}

pub async fn update_selected_change_set(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
    Json(request): Json<UpdateSelectedChangeSetRequest>,
) -> ChangeSetResult<Json<UpdateSelectedChangeSetResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    if let Some(current_edit_session_pk) = request.current_edit_session_pk {
        let mut current_edit_session = EditSession::get_by_pk(&ctx, &current_edit_session_pk)
            .await?
            .ok_or(ChangeSetError::EditSessionNotFound)?;
        current_edit_session.save(&ctx).await?;
    }

    let change_set = ChangeSet::get_by_pk(&ctx, &request.next_change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    let ctx = ctx.clone_with_new_tenancies(
        ctx.read_tenancy().clone(),
        WriteTenancy::new_billing_account(claim.billing_account_id),
    );
    let current_date_time = Utc::now();
    let edit_session_name = current_date_time.to_string();
    let edit_session = EditSession::new(&ctx, &change_set.pk, &edit_session_name, None).await?;

    txns.commit().await?;

    Ok(Json(UpdateSelectedChangeSetResponse {
        change_set,
        edit_session,
    }))
}
