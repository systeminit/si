use super::ChangeSetResult;
use crate::server::extract::{Authorization, HandlerContext, Tenancy};
use axum::Json;
use chrono::Utc;
use dal::{ChangeSet, EditSession, HistoryActor, WriteTenancy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetRequest {
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetResponse {
    pub change_set: ChangeSet,
    pub edit_session: EditSession,
}

pub async fn create_change_set(
    HandlerContext(builder, mut txns): HandlerContext,
    Authorization(claim): Authorization,
    Tenancy(_write_tenancy, read_tenancy): Tenancy,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
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

    let change_set = ChangeSet::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.history_actor(),
        request.change_set_name,
        None,
    )
    .await?;
    let current_date_time = Utc::now();
    let edit_session_name = current_date_time.to_string();
    let edit_session = EditSession::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.history_actor(),
        &change_set.pk,
        &edit_session_name,
        None,
    )
    .await?;

    txns.commit().await?;

    Ok(Json(CreateChangeSetResponse {
        change_set,
        edit_session,
    }))
}
