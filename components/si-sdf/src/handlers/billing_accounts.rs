use nats::asynk::Connection;
use si_data::Db;

use crate::handlers::HandlerError;
use crate::models::billing_account::{BillingAccount, CreateReply, CreateRequest};

pub async fn create(
    db: Db,
    nats: Connection,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let (billing_account, user, group, organization, workspace) = BillingAccount::signup(
        &db,
        &nats,
        request.billing_account_name,
        request.billing_account_description,
        request.user_name,
        request.user_email,
        request.user_password,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply {
        billing_account,
        user,
        group,
        organization,
        workspace,
    };
    Ok(warp::reply::json(&reply))
}
