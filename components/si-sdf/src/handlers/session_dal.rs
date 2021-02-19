use crate::data::PgPool;
use crate::handlers::{authenticate, HandlerError};
use crate::models::{get_jwt_signing_key, BillingAccount, Organization, System, User, Workspace};
use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::coarsetime::Duration;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;

use crate::models::user::SiClaims;

const GET_DEFAULTS: &str = include_str!("../data/queries/session_dal_get_defaults.sql");

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub billing_account_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LoginReply {
    pub user: User,
    pub billing_account: BillingAccount,
    pub jwt: String,
}

pub async fn login(
    pg: PgPool,
    secret_key: secretbox::Key,
    request: LoginRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let billing_account = BillingAccount::get_by_name(&txn, &request.billing_account_name)
        .await
        .map_err(HandlerError::from)?;

    let user = User::get_by_email(&txn, &request.user_email, &billing_account.id)
        .await
        .map_err(HandlerError::from)?;
    let verified = user
        .verify(&txn, &request.user_password)
        .await
        .map_err(HandlerError::from)?;

    if !verified {
        return Err(warp::reject::Rejection::from(HandlerError::Unauthorized));
    }

    let signing_key = get_jwt_signing_key(&txn, &secret_key)
        .await
        .map_err(HandlerError::from)?;

    let si_claims = SiClaims {
        user_id: user.id.clone(),
        billing_account_id: user.si_storable.billing_account_id.clone(),
    };
    let claims = Claims::with_custom_claims(si_claims, Duration::from_days(1))
        .with_audience("https://app.systeminit.com")
        .with_issuer("https://app.systeminit.com")
        .with_subject(user.id.clone());
    let jwt = signing_key
        .sign(claims)
        .map_err(|err| HandlerError::JwtClaim(format!("{}", err)))?;

    let reply = LoginReply {
        user,
        billing_account,
        jwt,
    };

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreAuthenticationReply {
    pub user: User,
    pub billing_account: BillingAccount,
}

pub async fn restore_authentication(
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;

    let billing_account = BillingAccount::get(&txn, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;

    let user = User::get(&txn, &claim.user_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = RestoreAuthenticationReply {
        user,
        billing_account,
    };

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDefaultsReply {
    pub organization: Organization,
    pub workspace: Workspace,
    pub system: System,
}

pub async fn get_defaults(
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;

    let row = txn
        .query_one(GET_DEFAULTS, &[&claim.billing_account_id])
        .await
        .map_err(HandlerError::from)?;

    let org_json: serde_json::Value = row.try_get("organization").map_err(HandlerError::from)?;
    let organization: Organization =
        serde_json::from_value(org_json).map_err(HandlerError::from)?;
    let w_json: serde_json::Value = row.try_get("workspace").map_err(HandlerError::from)?;
    let workspace: Workspace = serde_json::from_value(w_json).map_err(HandlerError::from)?;
    let s_json: serde_json::Value = row.try_get("system").map_err(HandlerError::from)?;
    let system: System = serde_json::from_value(s_json).map_err(HandlerError::from)?;

    let reply = GetDefaultsReply {
        organization,
        workspace,
        system,
    };

    Ok(warp::reply::json(&reply))
}
