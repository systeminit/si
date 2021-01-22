use crate::data::{EventLogFS, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::{EventLog, ListRequest, OutputLineStream, PageToken, Query};
use crate::page_secret_key;

pub async fn get(
    event_log_id: String,
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "event_logs", &event_log_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, "event_log", "get").await?;

    let object = EventLog::get(&txn, &event_log_id)
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
    authorize(&txn, &claim.user_id, "event_log", "list").await?;

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

    let reply = EventLog::list(
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

pub async fn get_output(
    _event_log_id: String,
    _stream: OutputLineStream,
    pg: PgPool,
    _event_log_fs: EventLogFS,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "event_log", "get_output").await?;

    Ok(warp::reply::json(&serde_json::json!({})))
}
