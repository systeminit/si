use si_data::Db;

use crate::handlers::HandlerError;
use crate::models::user::{LoginReply, LoginRequest, User};

// TODO: Your mission, if you choose to accept it: implement JWT support, then make it so
//       all the warp endpoints that need authentication handle the token, the unwrap,
//       etc.
//
//       Then put the sign up form on the web ui, and start refactoring the store!
pub async fn login(
    db: Db,
    request: LoginRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let user = User::get_by_email(&db, &request.email, &request.billing_account_name)
        .await
        .map_err(HandlerError::from)?;
    let verified = user
        .verify(&db, &request.password)
        .await
        .map_err(HandlerError::from)?;

    if !verified {
        return Err(warp::reject::Rejection::from(HandlerError::Unauthorized));
    }

    let reply = LoginReply { user };
    Ok(warp::reply::json(&reply))
}
