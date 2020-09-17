use jwt_simple::algorithms::RSAKeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::coarsetime::Duration;
use serde::{Deserialize, Serialize};
use si_data::Db;
use sodiumoxide::crypto::secretbox;

use crate::handlers::HandlerError;
use crate::models::{BillingAccount, JwtKeyPrivate, LoginReply, LoginRequest, User};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SiClaims {
    pub user_id: String,
    pub billing_account_id: String,
}

// TODO: Your mission, if you choose to accept it: get a validation working for the JWT, then
//       use it to populate the right values in the requests like before.
//
//       Then put the sign up form on the web ui, and start refactoring the store!
pub async fn login(
    db: Db,
    secret_key: secretbox::Key,
    request: LoginRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let billing_account = BillingAccount::get_by_name(&db, &request.billing_account_name)
        .await
        .map_err(HandlerError::from)?;

    let user = User::get_by_email(&db, &request.email, &billing_account.id)
        .await
        .map_err(HandlerError::from)?;
    let verified = user
        .verify(&db, &request.password)
        .await
        .map_err(HandlerError::from)?;

    if !verified {
        return Err(warp::reject::Rejection::from(HandlerError::Unauthorized));
    }

    let signing_key = JwtKeyPrivate::get_jwt_signing_key(&db, &secret_key)
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
