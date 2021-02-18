use crate::{
    data::{NatsConn, PgPool},
    handlers::{authenticate, authorize, validate_tenancy, HandlerError},
    models::{Entity, Node, NodeKind, System},
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
