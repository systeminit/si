use crate::data::PgPool;
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::{entity, Entity, ListRequest, PageToken, Query};
use crate::page_secret_key;

pub async fn get(
    entity_id: String,
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "entities", &entity_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, &"entities", "get").await?;

    let entities = Entity::get_all(&txn, &entity_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = entity::GetReply { items: entities };

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
    authorize(&txn, &claim.user_id, "entities", "list").await?;

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

    let reply = Entity::list(
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
