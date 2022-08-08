use axum::Json;
use dal::{ChangeSet, WriteTenancy};
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetRequest {
    pub change_set_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn create_change_set(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
    Json(request): Json<CreateChangeSetRequest>,
) -> ChangeSetResult<Json<CreateChangeSetResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);
    let ctx = ctx.clone_with_new_tenancies(
        ctx.read_tenancy().clone(),
        WriteTenancy::new_billing_account(claim.billing_account_id),
    );

    let change_set = ChangeSet::new(&ctx, request.change_set_name, None).await?;

    txns.commit().await?;

    Ok(Json(CreateChangeSetResponse { change_set }))
}
