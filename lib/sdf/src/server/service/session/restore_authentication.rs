use axum::Json;
use dal::{BillingAccount, StandardModel, Tenancy, User, Visibility};
use serde::{Deserialize, Serialize};

use super::SessionResult;
use crate::server::{
    extract::{Authorization, PgRoTxn},
    service::session::SessionError,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationResponse {
    pub user: User,
    pub billing_account: BillingAccount,
}

pub async fn restore_authentication(
    mut txn: PgRoTxn,
    Authorization(claim): Authorization,
) -> SessionResult<Json<RestoreAuthenticationResponse>> {
    let txn = txn.start().await?;
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);

    let billing_account =
        BillingAccount::get_by_id(&txn, &tenancy, &visibility, &claim.billing_account_id)
            .await?
            .ok_or(SessionError::LoginFailed)?;

    let billing_account_tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);

    let user = User::get_by_id(&txn, &billing_account_tenancy, &visibility, &claim.user_id)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    let reply = RestoreAuthenticationResponse {
        user,
        billing_account,
    };

    Ok(Json(reply))
}
