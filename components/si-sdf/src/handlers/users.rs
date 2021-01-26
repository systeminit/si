use crate::data::PgPool;
use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::coarsetime::Duration;
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::secretbox;

use crate::handlers::HandlerError;
use crate::models::{get_jwt_signing_key, BillingAccount, LoginReply, LoginRequest, User};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiClaims {
    pub user_id: String,
    pub billing_account_id: String,
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

    let user = User::get_by_email(&txn, &request.email, &billing_account.id)
        .await
        .map_err(HandlerError::from)?;
    let verified = user
        .verify(&txn, &request.password)
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

    let reply = LoginReply { user, jwt };

    Ok(warp::reply::json(&reply))
}
