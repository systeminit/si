use super::SessionResult;
use crate::server::extract::{JwtSecretKey, NatsTxn, PgRwTxn};
use crate::server::service::session::SessionError;
use axum::Json;
use dal::{BillingAccount, StandardModel, Tenancy, User, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub billing_account_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user: User,
    pub billing_account: BillingAccount,
    pub jwt: String,
}

pub async fn login(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    JwtSecretKey(jwt_secret_key): JwtSecretKey,
    Json(request): Json<LoginRequest>,
) -> SessionResult<Json<LoginResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);

    let billing_account =
        BillingAccount::find_by_name(&txn, &tenancy, &visibility, &request.billing_account_name)
            .await?
            .ok_or(SessionError::LoginFailed)?;

    let ba_tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);
    let user = User::find_by_email(&txn, &ba_tenancy, &visibility, &request.user_email)
        .await?
        .ok_or(SessionError::LoginFailed)?;

    let jwt = user
        .login(
            &txn,
            &jwt_secret_key,
            billing_account.id(),
            &request.user_password,
        )
        .await
        .map_err(|_| SessionError::LoginFailed)?;

    txn.commit().await?;
    nats.commit().await?;
    Ok(Json(LoginResponse {
        jwt,
        user,
        billing_account,
    }))
}
