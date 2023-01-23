use super::SessionResult;
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};
use axum::Json;
use dal::{billing_account::BillingAccountDefaults, BillingAccount, Organization, Workspace};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDefaultsResponse {
    pub workspace: Workspace,
    pub organization: Organization,
}

impl From<BillingAccountDefaults> for GetDefaultsResponse {
    fn from(defaults: BillingAccountDefaults) -> Self {
        GetDefaultsResponse {
            workspace: defaults.workspace,
            organization: defaults.organization,
        }
    }
}

pub async fn get_defaults(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
) -> SessionResult<Json<GetDefaultsResponse>> {
    let ctx = builder.build(request_ctx.build_head()).await?;

    let response = BillingAccount::get_defaults(&ctx, &claim.billing_account_pk)
        .await?
        .into();

    Ok(Json(response))
}
