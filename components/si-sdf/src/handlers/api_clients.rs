use sodiumoxide::crypto::secretbox;

use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::api_client::{CreateReply, CreateRequest};
use crate::models::{ApiClient, ListRequest, PageToken, Query};
use crate::page_secret_key;

pub async fn create(
    pg: PgPool,
    nats_conn: NatsConn,
    secret_key: secretbox::Key,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "api_clients", "create").await?;

    let (api_client, jwt) = ApiClient::new(
        &txn,
        &nats,
        &secret_key,
        request.name,
        request.kind,
        &claim.billing_account_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply {
        api_client,
        token: jwt,
    };

    Ok(warp::reply::json(&reply))
}

pub async fn get(
    api_client_id: String,
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(
        &txn,
        "api_clients",
        &api_client_id,
        &claim.billing_account_id,
    )
    .await?;
    authorize(&txn, &claim.user_id, "api_clients", "get").await?;

    let object = ApiClient::get(&txn, &api_client_id)
        .await
        .map_err(HandlerError::from)?;

    let item = serde_json::to_value(object).map_err(HandlerError::from)?;

    let reply = crate::models::GetReply { item };
    Ok(warp::reply::json(&reply))
}

pub async fn list(
    pg: PgPool,
    token: String,
    request: ListRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "api_clients", "list").await?;

    let query = if let Some(query) = request.query {
        Some(Query::from_url_string(query).map_err(HandlerError::from)?)
    } else {
        None
    };

    let page_token = if let Some(page_token) = request.page_token {
        Some(PageToken::unseal(&page_token, page_secret_key()).map_err(HandlerError::from)?)
    } else {
        None
    };

    let reply = ApiClient::list(
        &txn,
        &claim.billing_account_id,
        query,
        request.page_size,
        request.order_by,
        request.order_by_direction,
        page_token,
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}
