use crate::data::{NatsConn, PgPool};
use crate::handlers::HandlerError;
use crate::models::BillingAccount;
use crate::Veritech;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub billing_account_name: String,
    pub billing_account_description: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub billing_account: BillingAccount,
}

pub async fn create_billing_account(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let (billing_account, _user, _group, _organization, _workspace, _public_key, _system) =
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

    // The db part of the transaction is committed in the function itself
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = CreateReply { billing_account };
    Ok(warp::reply::json(&reply))
}
