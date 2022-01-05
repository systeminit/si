use super::SessionResult;
use crate::server::extract::{Authorization, PgRoTxn};
use axum::Json;
use dal::billing_account::BillingAccountDefaults;
use dal::{BillingAccount, Organization, System, Tenancy, Visibility, Workspace};
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
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
) -> SessionResult<Json<GetDefaultsResponse>> {
    let txn = txn.start().await?;
    let tenancy = Tenancy::new_billing_account(vec![claim.billing_account_id]);
    let visibility = Visibility::new_head(false);
    let response =
        BillingAccount::get_defaults(&txn, &tenancy, &visibility, &claim.billing_account_id)
            .await?
            .into();
    Ok(Json(response))
}
