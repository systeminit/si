use std::collections::HashMap;

use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{
    application, discovery,
    entity::diff::{diff_for_props, Diffs},
    ApplicationEntities, Connection, Connections, DiscoveryListEntry, Edge, EdgeKind, Entity,
    LabelList, LabelListItem, NodePosition, Qualification, Schematic, SchematicKind, Veritech,
    Vertex,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiscoveryListRequest {
    pub workspace_id: String,
    pub entity_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDiscoveryListReply {
    list: Vec<DiscoveryListEntry>,
}

pub async fn get_discovery_list(
    pg: PgPool,
    token: String,
    request: GetDiscoveryListRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getDiscoveryList").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;

    let list = discovery::list(&txn, &request.workspace_id, &request.entity_type)
        .await
        .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&GetDiscoveryListReply { list }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetImplementationsListRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub implementation_entity_types: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetImplementationsListReply {
    list: HashMap<String, Vec<DiscoveryListEntry>>,
}

pub async fn get_implementations_list(
    pg: PgPool,
    token: String,
    request: GetImplementationsListRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(
        &txn,
        &claim.user_id,
        "attributeDal",
        "getImplementationsList",
    )
    .await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;

    let list = discovery::implementations_list(
        &txn,
        &request.workspace_id,
        &request.application_id,
        request.implementation_entity_types,
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&GetImplementationsListReply { list }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub entity_type: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverReply {
    success: bool,
}

pub async fn discover(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: DiscoverRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "discover").await?;
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
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;

    txn.commit().await.map_err(HandlerError::from)?;

    discovery::discover(
        &pg,
        &nats_conn,
        &veritech,
        &request.workspace_id,
        &request.entity_id,
        &request.entity_type,
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&DiscoverReply { success: true }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportImplementationRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub entity_id: String,
    pub implementation_entity_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportImplementationReply {
    success: bool,
}

pub async fn import_implementation(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: ImportImplementationRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "importImplementation").await?;
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
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "entities",
        &request.implementation_entity_id,
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
    txn.commit().await.map_err(HandlerError::from)?;

    discovery::import_implementation(
        &pg,
        &nats_conn,
        &veritech,
        &request.workspace_id,
        &request.application_id,
        &request.entity_id,
        &request.implementation_entity_id,
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&ImportImplementationReply {
        success: true,
    }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportConceptRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub implementation_entity_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImportConceptReply {
    success: bool,
}

pub async fn import_concept(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: ImportConceptRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "importConcept").await?;
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
        &request.implementation_entity_id,
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
    txn.commit().await.map_err(HandlerError::from)?;

    discovery::import_concept(
        &pg,
        &nats_conn,
        &veritech,
        &request.workspace_id,
        &request.application_id,
        &request.implementation_entity_id,
    )
    .await
    .map_err(HandlerError::from)?;

    Ok(warp::reply::json(&ImportConceptReply { success: true }))
}

// ----------------------------------------------------------------------------
// Entity
// ----------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityListRequest {
    pub workspace_id: String,
    pub application_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

pub type GetEntityListReply = ApplicationEntities;

pub async fn get_entity_list(
    pg: PgPool,
    token: String,
    request: GetEntityListRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntityList").await?;
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
        &request.application_id,
        &claim.billing_account_id,
    )
    .await?;
    if let Some(change_set_id) = request.change_set_id.as_ref() {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
    }
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let mut reply = application::all_entities(
        &txn,
        &request.application_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;

    reply.entity_list.insert(
        0,
        LabelListItem {
            value: "".to_string(),
            label: "".to_string(),
        },
    );

    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetEntityReply {
    pub entity: Entity,
    pub diff: Diffs,
    pub qualifications: Vec<Qualification>,
}

pub async fn get_entity(
    pg: PgPool,
    token: String,
    request: GetEntityRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntityList").await?;
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
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;
    if let Some(change_set_id) = request.change_set_id.as_ref() {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
    }
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let entity = Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(|_| HandlerError::InvalidContext)?;

    let diff = match Entity::for_diff(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    {
        Ok(lhs) => diff_for_props(&lhs, &entity).map_err(HandlerError::from)?,
        Err(_e) => diff_for_props(&entity, &entity).map_err(HandlerError::from)?,
    };

    let qualifications: Vec<Qualification> = Qualification::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = GetEntityReply {
        entity,
        diff,
        qualifications,
    };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntityRequest {
    pub workspace_id: String,
    pub entity: Entity,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEntityReply {
    pub entity: Entity,
    pub diff: Diffs,
    pub qualifications: Vec<Qualification>,
    pub label: LabelListItem,
}

pub async fn update_entity(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: UpdateEntityRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "saveEntity").await?;
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
        &request.entity.id,
        &claim.billing_account_id,
    )
    .await?;
    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    // These are going to get committed toot-suite. Probably dumb to have them
    // here, but.. in a hurry

    let mut entity = request.entity;
    entity
        .update_entity_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let diff = match Entity::for_diff(
        &txn,
        &entity.id,
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
    )
    .await
    {
        Ok(lhs) => diff_for_props(&lhs, &entity).map_err(HandlerError::from)?,
        Err(_e) => diff_for_props(&entity, &entity).map_err(HandlerError::from)?,
    };

    let label = LabelListItem {
        label: entity.name.clone(),
        value: entity.id.clone(),
    };

    let qualifications: Vec<Qualification> = Qualification::for_head_or_change_set_or_edit_session(
        &txn,
        &entity.id,
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    entity
        .check_qualifications_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            request.system_id,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let reply = UpdateEntityReply {
        entity,
        diff,
        qualifications,
        label,
    };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckQualificationsRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckQualificationsReply {
    pub success: bool,
}

pub async fn check_qualifications(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: CheckQualificationsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "checkQualifications").await?;
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
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;

    let entity = Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        Some(&request.change_set_id),
        Some(&request.edit_session_id),
    )
    .await
    .map_err(HandlerError::from)?;
    txn.commit().await.map_err(HandlerError::from)?;

    entity
        .check_qualifications_for_edit_session(
            &pg,
            &nats_conn,
            &veritech,
            request.system_id,
            &request.change_set_id,
            &request.edit_session_id,
        )
        .await
        .map_err(HandlerError::from)?;

    let reply = CheckQualificationsReply { success: true };
    Ok(warp::reply::json(&reply))
}

// ----------------------------------------------------------------------------
// Connections
// ----------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetConnectionsRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetConnectionsReply {
    pub connections: Connections,
}

pub async fn get_connections(
    pg: PgPool,
    token: String,
    request: GetConnectionsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntityList").await?;
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
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;
    if let Some(change_set_id) = request.change_set_id.as_ref() {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
    }
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let mut edge_kinds: Vec<EdgeKind> = Vec::new();
    edge_kinds.push(EdgeKind::Configures);
    edge_kinds.push(EdgeKind::Deployment);

    let entity = Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(|_| HandlerError::InvalidContext)?;

    let connections = Connection::connections_from_entity(
        entity.id,
        edge_kinds,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
        &txn,
    )
    .await
    .map_err(|_| HandlerError::InvalidContext)?;

    txn.commit().await.map_err(HandlerError::from)?;

    let reply = GetConnectionsReply { connections };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConnectionRequest {
    pub workspace_id: String,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
    pub edge_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteConnectionReply {
    pub deleted: bool,
}

pub async fn delete_connection(
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
    request: DeleteConnectionRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getEntityList").await?;
    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(&txn, "edges", &request.edge_id, &claim.billing_account_id).await?;
    if let Some(change_set_id) = request.change_set_id.as_ref() {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
    }
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    let mut edge = Edge::get(&txn, &request.edge_id)
        .await
        .map_err(HandlerError::from)?;

    let _result = edge.delete(&txn, &nats).await.map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = DeleteConnectionReply { deleted: true };
    Ok(warp::reply::json(&reply))
}

// ----------------------------------------------------------------------------
// Input labels
// ----------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetInputLabelsRequest {
    pub workspace_id: String,
    pub entity_id: String,
    pub input_name: String,
    pub schematic_kind: SchematicKind,
    pub change_set_id: Option<String>,
    pub edit_session_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetInputLabelsReply {
    pub items: LabelList,
}

pub async fn get_input_labels(
    pg: PgPool,
    token: String,
    request: GetInputLabelsRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "attributeDal", "getInputLabels").await?;
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
        &request.entity_id,
        &claim.billing_account_id,
    )
    .await?;
    if let Some(change_set_id) = request.change_set_id.as_ref() {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
    }
    if let Some(edit_session_id) = request.edit_session_id.as_ref() {
        validate_tenancy(
            &txn,
            "edit_sessions",
            &edit_session_id,
            &claim.billing_account_id,
        )
        .await?;
    }

    Entity::for_head_or_change_set_or_edit_session(
        &txn,
        &request.entity_id,
        request.change_set_id.as_ref(),
        request.edit_session_id.as_ref(),
    )
    .await
    .map_err(|_| HandlerError::InvalidContext)?;

    let schematic = Schematic::get_by_schematic_kind(
        &txn,
        &request.schematic_kind,
        &request.entity_id,
        request.change_set_id.clone(),
        request.edit_session_id.clone(),
    )
    .await
    .map_err(HandlerError::from)?;

    let mut items: LabelList = vec![];
    for edge in schematic.edges.values() {
        if edge.head_vertex.object_id == request.entity_id
            && edge.head_vertex.socket == request.input_name
        {
            let schematic_node = match schematic.nodes.get(&edge.tail_vertex.node_id) {
                Some(schematic_node) => schematic_node,
                None => continue,
            };
            items.push(LabelListItem {
                label: format!(
                    "{}: {}",
                    &schematic_node.object.entity_type, &schematic_node.object.name,
                ),
                value: schematic_node.object.id.clone(),
            });
        }
    }

    let reply = GetInputLabelsReply { items };
    Ok(warp::reply::json(&reply))
}
