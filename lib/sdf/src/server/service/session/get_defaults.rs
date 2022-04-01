use super::SessionResult;
use crate::server::extract::{AccessBuilder, Authorization, HandlerContext};
use axum::Json;
use dal::billing_account::BillingAccountDefaults;
use dal::{BillingAccount, Organization, System, Workspace};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDefaultsResponse {
    pub workspace: Workspace,
    pub organization: Organization,
    pub system: System,
}

impl From<BillingAccountDefaults> for GetDefaultsResponse {
    fn from(defaults: BillingAccountDefaults) -> Self {
        GetDefaultsResponse {
            workspace: defaults.workspace,
            organization: defaults.organization,
            system: defaults.system,
        }
    }
}

pub async fn get_defaults(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Authorization(claim): Authorization,
) -> SessionResult<Json<GetDefaultsResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build_head(), &txns);

    let response = BillingAccount::get_defaults(&ctx, &claim.billing_account_id)
        .await?
        .into();
    Ok(Json(response))
}
