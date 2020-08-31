use crate::data::Db;
use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::{edge, Edge};

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn all_predecessors(
    db: Db,
    token: String,
    request: edge::AllPredecessorsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "edges",
        "allPredecessors",
    )
    .await?;

    let edges = if let Some(object_id) = request.object_id {
        Edge::all_predecessor_edges_by_object_id(&db, request.edge_kind, &object_id)
            .await
            .map_err(HandlerError::from)?
    } else if let Some(node_id) = request.node_id {
        Edge::all_predecessor_edges_by_node_id(&db, request.edge_kind, &node_id)
            .await
            .map_err(HandlerError::from)?
    } else {
        return Err(warp::reject::custom(HandlerError::InvalidRequest));
    };

    let reply = edge::AllPredecessorsReply { edges };

    Ok(warp::reply::json(&reply))
}

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn all_successors(
    db: Db,
    token: String,
    request: edge::AllSuccessorsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    tracing::error!("time for successors");
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "edges",
        "allSuccessors",
    )
    .await?;
    tracing::error!("time for edges");

    let edges = if let Some(object_id) = request.object_id {
        tracing::error!("time for object id");
        Edge::all_successor_edges_by_object_id(&db, request.edge_kind, &object_id)
            .await
            .map_err(HandlerError::from)?
    } else if let Some(node_id) = request.node_id {
        tracing::error!("time for node id");
        Edge::all_successor_edges_by_node_id(&db, request.edge_kind, &node_id)
            .await
            .map_err(HandlerError::from)?
    } else {
        return Err(warp::reject::custom(HandlerError::InvalidRequest));
    };

    tracing::error!("time for reply");

    let reply = edge::AllSuccessorsReply { edges };

    Ok(warp::reply::json(&reply))
}
