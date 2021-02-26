use crate::{
    data::{NatsConn, PgPool},
    handlers::{authenticate, authorize, validate_tenancy, HandlerError},
    models::{ChangeSet, Entity, Node, NodeKind, OpEntitySet, OpSetName, System},
    veritech::Veritech,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateForApplicationRequest {
    pub name: Option<String>,
    pub kind: NodeKind,
    pub object_type: String,
    pub workspace_id: String,
    pub change_set_id: String,
    pub edit_session_id: String,
    pub system_id: String,
    pub application_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeObject {
    pub entity: Option<Entity>,
    pub system: Option<System>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NodeCreateReply {
    pub node: Node,
    pub object: NodeObject,
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

    let (node, object) = if request.kind == NodeKind::Entity {
        let node = Node::new(
            &pg,
            &txn,
            &nats_conn,
            &nats,
            &veritech,
            request.name,
            request.kind,
            request.object_type,
            request.workspace_id,
            request.change_set_id.clone(),
            request.edit_session_id,
            Some(vec![request.system_id.clone()]),
        )
        .await
        .map_err(HandlerError::from)?;
        let entity = node
            .get_projection_object_entity(&txn, &request.change_set_id)
            .await
            .map_err(HandlerError::from)?;
        let object = NodeObject {
            entity: Some(entity),
            system: None,
        };
        (node, object)
    } else {
        let node = Node::new(
            &pg,
            &txn,
            &nats_conn,
            &nats,
            &veritech,
            request.name,
            request.kind,
            request.object_type,
            request.workspace_id,
            request.change_set_id.clone(),
            request.edit_session_id,
            None,
        )
        .await
        .map_err(HandlerError::from)?;
        let system = node
            .get_projection_object_system(&txn, &request.change_set_id)
            .await
            .map_err(HandlerError::from)?;
        let object = NodeObject {
            entity: None,
            system: Some(system),
        };
        (node, object)
    };

    let application_entity = Entity::get_head(&txn, &request.application_id)
        .await
        .map_err(HandlerError::from)?;
    node.configured_by(&txn, &nats, &application_entity.node_id)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = NodeCreateReply { node, object };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetPropertyRequest {
    workspace_id: String,
    entity_id: String,
    change_set_id: String,
    edit_session_id: String,
    override_system: Option<String>,
    path: Vec<String>,
    value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetPropertyReply {
    object: Entity,
}

pub async fn entity_set_property(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: EntitySetPropertyRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "editorDal", "entitySetProperty").await?;
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

    let mut change_set: ChangeSet = ChangeSet::get(&txn, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;

    let _op = OpEntitySet::new(
        &txn,
        &nats,
        &request.entity_id,
        request.path,
        request.value,
        request.override_system,
        request.workspace_id,
        request.change_set_id.clone(),
        request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let _item_ids = change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let entity = Entity::get_projection_or_head(&txn, &request.entity_id, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;
    txn.commit().await.map_err(HandlerError::from)?;

    let reply = EntitySetPropertyReply { object: entity };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetNameRequest {
    workspace_id: String,
    entity_id: String,
    change_set_id: String,
    edit_session_id: String,
    override_system: Option<String>,
    name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetNameReply {
    object: Entity,
}

pub async fn entity_set_name(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: EntitySetNameRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "editorDal", "entitySetName").await?;
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

    let mut change_set: ChangeSet = ChangeSet::get(&txn, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;

    let _op = OpSetName::new(
        &txn,
        &nats,
        &request.entity_id,
        request.name,
        request.workspace_id,
        request.change_set_id.clone(),
        request.edit_session_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let _item_ids = change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let entity = Entity::get_projection_or_head(&txn, &request.entity_id, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;
    txn.commit().await.map_err(HandlerError::from)?;

    let reply = EntitySetPropertyReply { object: entity };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetPropertyBulkProperties {
    path: Vec<String>,
    value: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetPropertyBulkRequest {
    workspace_id: String,
    entity_id: String,
    change_set_id: String,
    edit_session_id: String,
    override_system: Option<String>,
    properties: Vec<EntitySetPropertyBulkProperties>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EntitySetPropertyBulkReply {
    object: Entity,
}

pub async fn entity_set_property_bulk(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: EntitySetPropertyBulkRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "editorDal", "entitySetProperty").await?;
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

    let mut change_set: ChangeSet = ChangeSet::get(&txn, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;

    for property in request.properties.into_iter() {
        let _op = OpEntitySet::new(
            &txn,
            &nats,
            &request.entity_id,
            property.path,
            property.value,
            request.override_system.clone(),
            request.workspace_id.clone(),
            request.change_set_id.clone(),
            request.edit_session_id.clone(),
        )
        .await
        .map_err(HandlerError::from)?;
    }

    let _item_ids = change_set
        .execute(&pg, &txn, &nats_conn, &nats, &veritech, true, None)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let entity = Entity::get_projection_or_head(&txn, &request.entity_id, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;
    txn.commit().await.map_err(HandlerError::from)?;

    let reply = EntitySetPropertyReply { object: entity };
    Ok(warp::reply::json(&reply))
}
