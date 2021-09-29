use crate::handlers::{authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{
    schematic::{self, LinkNodeItem},
    Edge, EdgeKind, Entity, Node, NodePosition, Schematic, SchematicKind, SchematicNode, SiClaims,
    Veritech, Vertex,
};

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
    pub include_root_node: bool,
    pub schematic_kind: SchematicKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationSystemSchematicReply {
    schematic: Schematic,
}

pub async fn get_application_system_schematic(
    claim: SiClaims,
    request: GetApplicationSystemSchematicRequest,
    pg: PgPool,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

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

    // We are just checking to see that the root node exists in our context.
    // This whole thing needs to be streamlined. But it works fine for now.
    match Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.root_object_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    {
        Ok(_) => {}
        Err(_e) => {
            return Err(HandlerError::InvalidContext.into());
        }
    }

    let schematic = Schematic::get_by_schematic_kind(
        &txn,
        &request.schematic_kind,
        &request.root_object_id,
        request.change_set_id.clone(),
        request.edit_session_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

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
    pub socket_name: String,
    pub node_kind: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
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
    pub root_object_id: String,
    pub schematic_kind: SchematicKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreateReply {
    edge: Edge,
    schematic: Schematic,
}

pub async fn connection_create(
    claim: SiClaims,
    request: ConnectionCreateRequest,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

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
        &request.root_object_id,
        &claim.billing_account_id,
    )
    .await?;

    let source_node = Node::get(&txn, &request.connection.source.node_id)
        .await
        .map_err(HandlerError::from)?;
    let tail_vertex = Vertex::new(
        &request.connection.source.node_id,
        &source_node.object_id,
        &request.connection.source.socket_name,
        &request.connection.source.node_kind,
    );

    let destination_node = Node::get(&txn, &request.connection.destination.node_id)
        .await
        .map_err(HandlerError::from)?;
    let head_vertex = Vertex::new(
        &request.connection.destination.node_id,
        &destination_node.object_id,
        &request.connection.destination.socket_name,
        &request.connection.destination.node_kind,
    );

    let edge_kind = match request.schematic_kind {
        SchematicKind::Deployment => EdgeKind::Deployment,
        SchematicKind::Component => EdgeKind::Configures,
        _ => EdgeKind::Configures,
    };

    let edge = Edge::new(
        &txn,
        &nats,
        tail_vertex.clone(),
        head_vertex.clone(),
        false,
        edge_kind,
        &request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    // A cheap and dirty way to re-trigger property calculations
    // TODO: This should probably all be cleaned up to be efficient!
    let mut tail_entity = Entity::for_edit_session(
        &txn,
        &tail_vertex.object_id,
        &request.change_set_id,
        &request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;
    tail_entity
        .update_entity_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let mut head_entity = Entity::for_edit_session(
        &txn,
        &head_vertex.object_id,
        &request.change_set_id,
        &request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;
    head_entity
        .update_entity_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let schematic = Schematic::get_by_schematic_kind(
        &txn,
        &request.schematic_kind,
        &request.root_object_id,
        Some(request.change_set_id.clone()),
        Some(request.edit_session_id.clone()),
    )
    .await
    .map_err(HandlerError::from)?;

    head_entity
        .check_qualifications_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            None,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;
    tail_entity
        .check_qualifications_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            None,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;

    let reply = ConnectionCreateReply { edge, schematic };
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
    pub deployment_selected_entity_id: Option<String>,
    pub schematic_kind: SchematicKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateReply {
    pub node: SchematicNode,
    pub schematic: Schematic,
}

pub async fn node_create_for_application(
    claim: SiClaims,
    request: NodeCreateForApplicationRequest,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

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

    let schematic_node = SchematicNode::new(
        &txn,
        node.clone(),
        entity.clone(),
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
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
        Vertex::from_node(&node, "includes"),
        false,
        si_model::EdgeKind::Includes,
        request.workspace_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

    let schematic = match request.schematic_kind {
        SchematicKind::Deployment => {
            let _edge = Edge::new(
                &txn,
                &nats,
                Vertex::from_entity(&application_entity, "output"),
                Vertex::from_node(&node, "application"),
                false,
                si_model::EdgeKind::Component,
                request.workspace_id.clone(),
            )
            .await
            .map_err(HandlerError::from)?;

            Schematic::get_by_schematic_kind(
                &txn,
                &request.schematic_kind,
                &application_entity.id,
                Some(request.change_set_id.clone()),
                Some(request.edit_session_id.clone()),
            )
            .await
            .map_err(HandlerError::from)?
        }
        SchematicKind::Component => {
            if let Some(deployment_selected_entity_id) = request.deployment_selected_entity_id {
                let deployment_entity = Entity::for_head_or_change_set_or_edit_session(
                    &txn,
                    &deployment_selected_entity_id,
                    Some(&request.change_set_id),
                    Some(&request.edit_session_id),
                )
                .await
                .map_err(HandlerError::from)?;

                let _edge = Edge::new(
                    &txn,
                    &nats,
                    Vertex::from_entity(&deployment_entity, "output"),
                    Vertex::from_node(&node, "deployment"),
                    false,
                    si_model::EdgeKind::Component,
                    request.workspace_id.clone(),
                )
                .await
                .map_err(HandlerError::from)?;
                Schematic::get_by_schematic_kind(
                    &txn,
                    &request.schematic_kind,
                    &deployment_entity.id,
                    Some(request.change_set_id.clone()),
                    Some(request.edit_session_id.clone()),
                )
                .await
                .map_err(HandlerError::from)?
            } else {
                return Err(HandlerError::MissingDeploymentSelectedEntityId.into());
            }
        }
        _ => unreachable!(),
    };

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = NodeCreateReply {
        node: schematic_node,
        schematic,
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
    claim: SiClaims,
    request: UpdateNodePositionRequest,
    pg: PgPool,
    nats_conn: NatsConn,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

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
    pub deleted: bool,
}

pub async fn delete_node(
    claim: SiClaims,
    request: DeleteNodeRequest,
    pg: PgPool,
    nats_conn: NatsConn,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

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

    entity.delete();

    entity
        .save_for_edit_session(&txn, &request.change_set_id, &request.edit_session_id)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = DeleteNodeReply { deleted: true };

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeLinkForApplicationRequest {
    pub name: Option<String>,
    pub node_id: String,
    pub entity_id: String,
    pub entity_type: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub application_id: String,
    pub deployment_selected_entity_id: Option<String>,
    pub schematic_kind: SchematicKind,
}

pub async fn node_link_for_application(
    claim: SiClaims,
    request: NodeLinkForApplicationRequest,
    pg: PgPool,
    nats_conn: NatsConn,
    _veritech: Veritech,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    authorize(&txn, &claim.user_id, "editorDal", "nodeLink").await?;
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
    validate_tenancy(
        &txn,
        "entities",
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(&txn, "nodes", &request.node_id, &claim.billing_account_id).await?;

    let node = Node::get(&txn, &request.node_id)
        .await
        .map_err(HandlerError::from)?;

    let entity = Entity::for_edit_session(
        &txn,
        &request.entity_id,
        &request.change_set_id,
        &request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let schematic_node = SchematicNode::new(
        &txn,
        node.clone(),
        entity.clone(),
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
    )
    .await
    .map_err(HandlerError::from)?;

    let application_entity = Entity::for_head(&txn, &request.application_id)
        .await
        .map_err(HandlerError::from)?;

    let _edge = Edge::new_if_not_exists(
        &txn,
        &nats,
        Vertex::from_entity(&application_entity, "output"),
        Vertex::from_node(&node, "includes"),
        false,
        si_model::EdgeKind::Includes,
        request.workspace_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

    let schematic = match request.schematic_kind {
        SchematicKind::Deployment => {
            let _edge = Edge::new_if_not_exists(
                &txn,
                &nats,
                Vertex::from_entity(&application_entity, "output"),
                Vertex::from_node(&node, "application"),
                false,
                si_model::EdgeKind::Component,
                request.workspace_id.clone(),
            )
            .await
            .map_err(HandlerError::from)?;

            Schematic::get_by_schematic_kind(
                &txn,
                &request.schematic_kind,
                &application_entity.id,
                Some(request.change_set_id.clone()),
                Some(request.edit_session_id.clone()),
            )
            .await
            .map_err(HandlerError::from)?
        }
        SchematicKind::Component => {
            if let Some(deployment_selected_entity_id) = request.deployment_selected_entity_id {
                let deployment_entity = Entity::for_head_or_change_set_or_edit_session(
                    &txn,
                    &deployment_selected_entity_id,
                    Some(&request.change_set_id),
                    Some(&request.edit_session_id),
                )
                .await
                .map_err(HandlerError::from)?;

                let _edge = Edge::new_if_not_exists(
                    &txn,
                    &nats,
                    Vertex::from_entity(&deployment_entity, "output"),
                    Vertex::from_node(&node, "deployment"),
                    false,
                    si_model::EdgeKind::Component,
                    request.workspace_id.clone(),
                )
                .await
                .map_err(HandlerError::from)?;
                Schematic::get_by_schematic_kind(
                    &txn,
                    &request.schematic_kind,
                    &deployment_entity.id,
                    Some(request.change_set_id.clone()),
                    Some(request.edit_session_id.clone()),
                )
                .await
                .map_err(HandlerError::from)?
            } else {
                return Err(HandlerError::MissingDeploymentSelectedEntityId.into());
            }
        }
        _ => unreachable!(),
    };

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = NodeCreateReply {
        node: schematic_node,
        schematic,
    };

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeLinkMenuRequest {
    pub entity_types: Vec<String>,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub component_entity_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNodeLinkMenuReply {
    pub link: Vec<LinkNodeItem>,
}

pub async fn get_node_link_menu(
    claim: SiClaims,
    request: GetNodeLinkMenuRequest,
    pg: PgPool,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    authorize(&txn, &claim.user_id, "attributeDal", "getNodeLinkMenu").await?;
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

    let link = schematic::get_link_menu(
        &txn,
        &request.workspace_id,
        &request.change_set_id,
        &request.edit_session_id,
        &request.component_entity_id,
        request.entity_types,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = GetNodeLinkMenuReply { link };

    txn.commit().await.map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}
