use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::{edge, Edge, GetReply, ListRequest, PageToken, Query};
use crate::page_secret_key;

pub async fn get(
    edge_id: String,
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "edges", &edge_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, &"edges", "get").await?;

    let edge = Edge::get(&txn, &edge_id)
        .await
        .map_err(HandlerError::from)?;

    let item = serde_json::to_value(edge).map_err(HandlerError::from)?;
    let reply = GetReply { item };

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
    authorize(&txn, &claim.user_id, "edges", "list").await?;

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

    let reply = Edge::list(
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

pub async fn delete(
    edge_id: String,
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "edges", &edge_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, &"edges", "delete").await?;

    let mut edge = Edge::get(&txn, &edge_id)
        .await
        .map_err(HandlerError::from)?;
    edge.delete(&txn, &nats).await.map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = edge::DeleteReply { edge };

    Ok(warp::reply::json(&reply))
}

pub async fn all_predecessors(
    pg: PgPool,
    token: String,
    request: edge::AllPredecessorsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, &"edges", "allPredecessors").await?;

    let edges = if let Some(object_id) = request.object_id {
        if object_id.starts_with("system:") {
            validate_tenancy(&txn, "systems", &object_id, &claim.billing_account_id).await?;
        } else {
            validate_tenancy(&txn, "entities", &object_id, &claim.billing_account_id).await?;
        }
        Edge::all_predecessor_edges_by_object_id(&txn, &request.edge_kind, &object_id)
            .await
            .map_err(HandlerError::from)?
    } else if let Some(node_id) = request.node_id {
        validate_tenancy(&txn, "nodes", &node_id, &claim.billing_account_id).await?;
        Edge::all_predecessor_edges_by_node_id(&txn, &request.edge_kind, &node_id)
            .await
            .map_err(HandlerError::from)?
    } else {
        return Err(warp::reject::custom(HandlerError::InvalidRequest));
    };

    let reply = edge::AllPredecessorsReply { edges };

    Ok(warp::reply::json(&reply))
}

pub async fn all_successors(
    pg: PgPool,
    token: String,
    request: edge::AllSuccessorsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, &"edges", "allSuccessors").await?;

    let edges = if let Some(object_id) = request.object_id {
        if object_id.starts_with("system:") {
            validate_tenancy(&txn, "systems", &object_id, &claim.billing_account_id).await?;
        } else {
            validate_tenancy(&txn, "entities", &object_id, &claim.billing_account_id).await?;
        }
        Edge::all_successor_edges_by_object_id(&txn, &request.edge_kind, &object_id)
            .await
            .map_err(HandlerError::from)?
    } else if let Some(node_id) = request.node_id {
        validate_tenancy(&txn, "nodes", &node_id, &claim.billing_account_id).await?;
        Edge::all_successor_edges_by_node_id(&txn, &request.edge_kind, &node_id)
            .await
            .map_err(HandlerError::from)?
    } else {
        return Err(warp::reject::custom(HandlerError::InvalidRequest));
    };

    let reply = edge::AllSuccessorsReply { edges };

    Ok(warp::reply::json(&reply))
}
