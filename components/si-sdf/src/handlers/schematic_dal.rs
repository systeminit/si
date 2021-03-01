use crate::{
    data::{NatsConn, PgPool},
    handlers::{authenticate, authorize, validate_tenancy, HandlerError},
    models::{Edge, EdgeKind, Node, Schematic, Vertex},
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationSystemSchematicRequest {
    pub workspace_id: String,
    pub root_object_id: String,
    pub change_set_id: Option<String>,
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
        "systems",
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
        &request.system_id,
        request.change_set_id.clone(),
        vec![EdgeKind::Configures],
    )
    .await
    .map_err(HandlerError::from)?;

    let root_node = Node::get_for_object_id(&txn, &request.root_object_id, request.change_set_id)
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
    pub system_id: String,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreateRequest {
    pub connection: Connection,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub application_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreateReply {
    edge: Edge,
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
        "systems",
        &request.connection.system_id,
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
        &source_node.get_object_id(),
        &request.connection.source.socket_id,
        &request.connection.source.node_kind,
    );

    let destination_node = Node::get(&txn, &request.connection.destination.node_id)
        .await
        .map_err(HandlerError::from)?;
    let head_vertex = Vertex::new(
        &request.connection.destination.node_id,
        &destination_node.get_object_id(),
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

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = ConnectionCreateReply { edge: edge };
    Ok(warp::reply::json(&reply))
}
