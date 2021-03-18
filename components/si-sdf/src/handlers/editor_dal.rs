use si_data::{NatsConn, PgPool};

use si_model::{Edge, Entity, Node, NodePosition, Veritech, Vertex};

use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateForApplicationRequest {
    pub name: Option<String>,
    pub entity_type: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_id: String,
    pub application_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateReply {
    pub node: Node,
    pub entity: Entity,
}

pub async fn node_create_for_application(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: NodeCreateForApplicationRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "editorDal", "nodeCreate").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "change_sets",
        &request.change_set_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "edit_sessions",
        &request.edit_session_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "systems",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.application_id,
        &claim.billing_account_id,
    )
    .await?;

    let node = Node::new(
        &pg,
        &txn,
        &nats_conn,
        &nats,
        &veritech,
        request.name,
        request.entity_type,
        request.workspace_id.clone(),
        request.change_set_id.clone(),
        request.edit_session_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

    let entity = Entity::for_edit_session(
        &txn,
        &node.object_id,
        &request.change_set_id,
        &request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let application_entity = Entity::for_head(&txn, &request.application_id)
        .await
        .map_err(HandlerError::from)?;

    let _edge = Edge::new(
        &txn,
        &nats,
        Vertex::from_entity(&application_entity, "output"),
        Vertex::from_node(&node, "input"),
        false,
        si_model::EdgeKind::Includes,
        request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = NodeCreateReply { node, entity };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodePositionRequest {
    pub node_id: String,
    pub context_id: String,
    pub x: String,
    pub y: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNodePositionReply {
    pub node_position: NodePosition,
}

pub async fn update_node_position(
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
    request: UpdateNodePositionRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "editorDal", "updateNodePosition").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(&txn, "nodes", &request.node_id, &claim.billing_account_id).await?;

    let node_position = NodePosition::create_or_update(
        &txn,
        &nats,
        &request.node_id,
        &request.context_id,
        &request.x,
        &request.y,
        &request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = UpdateNodePositionReply { node_position };
    Ok(warp::reply::json(&reply))
}
