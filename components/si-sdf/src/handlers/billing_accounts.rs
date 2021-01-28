use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::{
    billing_account::{CreateReply, CreateRequest},
    BillingAccount, PublicKey,
};
use crate::Veritech;

pub async fn create(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let (billing_account, user, group, organization, workspace, _public_key, system) =
        BillingAccount::signup(
            &pg,
            txn,
            &nats,
            &nats_conn,
            &veritech,
            request.billing_account_name,
            request.billing_account_description,
            request.user_name,
            request.user_email,
            request.user_password,
        )
        .await
        .map_err(HandlerError::from)?;

    // The transaction is committed in the function itself
    //txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = CreateReply {
        billing_account,
        user,
        group,
        organization,
        workspace,
        system,
    };
    Ok(warp::reply::json(&reply))
}

pub async fn get_public_key(
    billing_account_id: String,
    pg: PgPool,
    token: String,
    type_name: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, &type_name, "get").await?;

    let public_key = PublicKey::get_current(&txn, billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let item = serde_json::to_value(public_key).map_err(HandlerError::from)?;

    let reply = crate::models::GetReply { item };
    Ok(warp::reply::json(&reply))
}

pub async fn get(
    billing_account_id: String,
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, &"billingAccount", "get").await?;
    if &claim.billing_account_id != &billing_account_id {
        return Err(warp::Rejection::from(HandlerError::Unauthorized));
    }

    let object = BillingAccount::get(&txn, &billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let item = serde_json::to_value(object).map_err(HandlerError::from)?;

    let reply = crate::models::GetReply { item };
    Ok(warp::reply::json(&reply))
}
