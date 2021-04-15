use si_data::{NatsConn, PgPool};
use si_model::{
    Edge, EdgeKind, Entity, Node, NodePosition, Schematic, SchematicNode, Veritech, Vertex,
};

use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};

// ===============================================================
// Schematic (nodes and edges)
// ===============================================================

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationSystemSchematicRequest {
    pub workspace_id: String,
    pub root_object_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
    pub system_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationSystemSchematicReply {
    schematic: Schematic,
}

pub async fn get_application_system_schematic(
    pg: PgPool,
    token: String,
    request: GetApplicationSystemSchematicRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(
        &txn,
        &claim.user_id,
        "schematicDal",
        "getApplicationSystemSchematic",
    )
    .await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.system_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.root_object_id,
        &claim.billing_account_id,
    )
    .await?;

    let mut schematic = Schematic::get(
        &txn,
        &request.root_object_id,
        &request.workspace_id,
        request.change_set_id.clone(),
        request.edit_session_id.clone(),
        vec![EdgeKind::Configures],
    )
    .await
    .map_err(HandlerError::from)?;

    let root_node = Node::get_for_object_id(
        &txn,
        &request.root_object_id,
        request.change_set_id.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;
    schematic.prune_node(root_node.id);
    txn.commit().await.map_err(HandlerError::from)?;

    let reply = GetApplicationSystemSchematicReply { schematic };
    Ok(warp::reply::json(&reply))
}

// ===============================================================
// Connection (nodes connections)
// ===============================================================

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionNodeReference {
    pub node_id: String,
    pub socket_id: String,
    pub node_kind: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub kind: String,
    pub source: ConnectionNodeReference,
    pub destination: ConnectionNodeReference,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreateRequest {
    pub connection: Connection,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub application_id: String,
    pub return_schematic: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreateReply {
    edge: Edge,
    schematic: Option<Schematic>,
}

pub async fn connection_create(
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
    request: ConnectionCreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "editorDal", "edgeCreate").await?;
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
        "entities",
        &request.application_id,
        &claim.billing_account_id,
    )
    .await?;

    let source_node = Node::get(&txn, &request.connection.source.node_id)
        .await
        .map_err(HandlerError::from)?;
    let tail_vertex = Vertex::new(
        &request.connection.source.node_id,
        &source_node.object_id,
        &request.connection.source.socket_id,
        &request.connection.source.node_kind,
    );

    let destination_node = Node::get(&txn, &request.connection.destination.node_id)
        .await
        .map_err(HandlerError::from)?;
    let head_vertex = Vertex::new(
        &request.connection.destination.node_id,
        &destination_node.object_id,
        &request.connection.destination.socket_id,
        &request.connection.destination.node_kind,
    );

    let edge = Edge::new(
        &txn,
        &nats,
        tail_vertex.clone(),
        head_vertex.clone(),
        false,
        EdgeKind::Configures, //TODO pass this as an argument
        &request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let mut schematic = Schematic::get(
        &txn,
        &request.application_id,
        &request.workspace_id,
        Some(request.change_set_id.clone()),
        Some(request.edit_session_id.clone()),
        vec![EdgeKind::Configures],
    )
    .await
    .map_err(HandlerError::from)?;

    let root_node =
        Node::get_for_object_id(&txn, &request.application_id, Some(&request.change_set_id))
            .await
            .map_err(HandlerError::from)?;
    schematic.prune_node(root_node.id);

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply_edge: Edge = edge;
    let mut reply_schematic: Option<Schematic> = None;
    if request.return_schematic {
        reply_schematic = Some(schematic);
    }
    let reply = ConnectionCreateReply {
        edge: reply_edge,
        schematic: reply_schematic,
    };
    Ok(warp::reply::json(&reply))
}

// ===============================================================
// Node (schematic nodes)
// ===============================================================

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateForApplicationRequest {
    pub name: Option<String>,
    pub entity_type: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub application_id: String,
    pub return_schematic: bool,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateReply {
    pub node: SchematicNode,
    pub schematic: Option<Schematic>,
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

    let schematic_node = SchematicNode::new(&txn, node.clone(), serde_json::json![entity])
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
        si_model::EdgeKind::Configures,
        request.workspace_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

    let mut schematic = Schematic::get(
        &txn,
        &request.application_id,
        &request.workspace_id,
        Some(request.change_set_id.clone()),
        Some(request.edit_session_id.clone()),
        vec![EdgeKind::Configures],
    )
    .await
    .map_err(HandlerError::from)?;

    let root_node =
        Node::get_for_object_id(&txn, &request.application_id, Some(&request.change_set_id))
            .await
            .map_err(HandlerError::from)?;
    schematic.prune_node(root_node.id);

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply_node: SchematicNode = schematic_node;
    let mut reply_schematic: Option<Schematic> = None;
    if request.return_schematic {
        reply_schematic = Some(schematic);
    }
    let reply = NodeCreateReply {
        node: reply_node,
        schematic: reply_schematic,
    };

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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNodeRequest {
    pub node_id: String,
    pub application_id: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNodeReply {
    pub schematic: Schematic,
}
pub async fn delete_node(
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
    request: DeleteNodeRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(
        &txn,
        &claim.user_id,
        "attributeDal",
        "save_for_edit_sessionEntity",
    )
    .await?;
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
    validate_tenancy(&txn, "nodes", &request.node_id, &claim.billing_account_id).await?;

    let node = Node::get(&txn, &request.node_id)
        .await
        .map_err(HandlerError::from)?;

    let mut entity = Entity::for_edit_session(
        &txn,
        &node.object_id,
        &request.change_set_id,
        &request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;

    validate_tenancy(&txn, "entities", &entity.id, &claim.billing_account_id).await?;

    entity.delete().await.map_err(HandlerError::from)?;

    entity
        .save_for_edit_session(&txn, &request.change_set_id, &request.edit_session_id)
        .await
        .map_err(HandlerError::from)?;

    let mut schematic = Schematic::get(
        &txn,
        &request.application_id,
        &request.workspace_id,
        Some(request.change_set_id.clone()),
        Some(request.edit_session_id),
        vec![EdgeKind::Configures],
    )
    .await
    .map_err(HandlerError::from)?;

    let root_node =
        Node::get_for_object_id(&txn, &request.application_id, Some(&request.change_set_id))
            .await
            .map_err(HandlerError::from)?;
    schematic.prune_node(root_node.id);

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = DeleteNodeReply {
        schematic: schematic,
    };

    Ok(warp::reply::json(&reply))
}
