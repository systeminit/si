use crate::{
    data::{Connection, Db},
    handlers::{authenticate, authorize, HandlerError},
    models::{
        billing_account::{CreateReply, CreateRequest},
        BillingAccount, PublicKey,
    },
};

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

pub async fn get_public_key(
    billing_account_id: String,
    db: Db,
    token: String,
    type_name: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        &type_name,
        "get",
    )
    .await?;

    let public_key = PublicKey::get_current(&db, billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let item = serde_json::to_value(public_key).map_err(HandlerError::from)?;

    let reply = crate::models::GetReply { item };
    Ok(warp::reply::json(&reply))
}
