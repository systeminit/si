use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::node::{
    CreateReply, CreateRequest, Node, NodeKind, ObjectPatchReply, ObjectPatchRequest,
    PatchConfiguredByReply, PatchIncludeSystemReply, PatchOp, PatchReply, PatchRequest,
    PatchSetPositionReply, SyncResourceReply,
};
use crate::models::ops::{
    OpEntityAction, OpEntityDelete, OpEntitySet, OpReply, OpRequest, OpSetName,
};
use crate::models::{ChangeSet, GetReply, GetRequest};
use crate::veritech::Veritech;

pub async fn create(
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    authorize(&txn, &claim.user_id, "node", "create").await?;

    let node = Node::new(
        &pg,
        &txn,
        &nats,
        &veritech,
        request.name,
        request.kind,
        request.object_type,
        request.workspace_id,
        request.change_set_id,
        request.edit_session_id,
        request.system_ids,
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = CreateReply { item: node };
    Ok(warp::reply::json(&reply))
}

pub async fn patch(
    node_id: String,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: PatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "nodes", &node_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, "node", "patch").await?;

    let mut node: Node = Node::get(&txn, &node_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = match request.op {
        PatchOp::IncludeSystem(system_req) => {
            let edge = node
                .include_in_system(&txn, &nats, &system_req.system_id)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::IncludeSystem(PatchIncludeSystemReply { edge })
        }
        PatchOp::ConfiguredBy(configured_by_req) => {
            let edge = node
                .configured_by(&txn, &nats, configured_by_req.node_id)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::ConfiguredBy(PatchConfiguredByReply { edge })
        }
        PatchOp::SetPosition(set_position_req) => {
            node.set_position(
                set_position_req.context.clone(),
                set_position_req.position.clone(),
            );
            node.save(&txn, &nats).await.map_err(HandlerError::from)?;
            PatchReply::SetPosition(PatchSetPositionReply {
                context: set_position_req.context,
                position: set_position_req.position,
            })
        }
        PatchOp::SyncResource(sync_resource_req) => {
            node.sync_resource(
                &pg,
                &txn,
                &nats,
                &veritech,
                sync_resource_req.system_id,
                sync_resource_req.change_set_id,
            )
            .await
            .map_err(HandlerError::from)?;
            PatchReply::SyncResource(SyncResourceReply {})
        }
    };

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    Ok(warp::reply::json(&reply))
}

pub async fn object_patch(
    node_id: String,
    pg: PgPool,
    nats_conn: NatsConn,
    veritech: Veritech,
    token: String,
    request: ObjectPatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "nodes", &node_id, &claim.billing_account_id).await?;
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
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "organizations",
        &request.organization_id,
        &claim.billing_account_id,
    )
    .await?;
    authorize(&txn, &claim.user_id, "object", "patch").await?;

    let node = Node::get(&txn, &node_id)
        .await
        .map_err(HandlerError::from)?;
    let entity_id = node.get_object_id();

    let mut change_set: ChangeSet = ChangeSet::get(&txn, &request.change_set_id)
        .await
        .map_err(HandlerError::from)?;

    match request.op {
        OpRequest::EntitySet(op_request) => {
            OpEntitySet::new(
                &txn,
                &nats,
                &entity_id,
                op_request.path,
                op_request.value,
                op_request.override_system,
                request.workspace_id,
                request.change_set_id,
                request.edit_session_id,
            )
            .await
            .map_err(HandlerError::from)?;
        }
        OpRequest::NameSet(op_request) => {
            OpSetName::new(
                &txn,
                &nats,
                &entity_id,
                op_request.value,
                request.workspace_id,
                request.change_set_id,
                request.edit_session_id,
            )
            .await
            .map_err(HandlerError::from)?;
        }
        OpRequest::EntityAction(op_request) => {
            OpEntityAction::new(
                &txn,
                &nats,
                entity_id,
                op_request.action,
                op_request.system_id,
                request.workspace_id,
                request.change_set_id,
                request.edit_session_id,
            )
            .await
            .map_err(HandlerError::from)?;
        }
        OpRequest::EntityDelete(_op_request) => {
            OpEntityDelete::new(
                &txn,
                &nats,
                &entity_id,
                request.workspace_id.clone(),
                request.change_set_id.clone(),
                request.edit_session_id.clone(),
            )
            .await
            .map_err(HandlerError::from)?;
        }
    }

    let item_ids = change_set
        .execute(&txn, &nats, &veritech, true, None)
        .await
        .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = ObjectPatchReply::Op(OpReply { item_ids });
    Ok(warp::reply::json(&reply))
}

pub async fn get_object(
    node_id: String,
    pg: PgPool,
    token: String,
    request: GetRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "nodes", &node_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, "object", "get").await?;

    let node = Node::get(&txn, node_id).await.map_err(HandlerError::from)?;
    let object: serde_json::Value = if let Some(change_set_id) = request.change_set_id {
        validate_tenancy(
            &txn,
            "change_sets",
            &change_set_id,
            &claim.billing_account_id,
        )
        .await?;
        match node.kind {
            NodeKind::System => {
                let obj = node
                    .get_projection_object_system(&txn, &change_set_id)
                    .await
                    .map_err(HandlerError::from)?;
                serde_json::to_value(obj).map_err(HandlerError::from)?
            }
            NodeKind::Entity => {
                let obj = node
                    .get_projection_object_entity(&txn, &change_set_id)
                    .await
                    .map_err(HandlerError::from)?;
                serde_json::to_value(obj).map_err(HandlerError::from)?
            }
        }
    } else {
        match node.kind {
            NodeKind::System => {
                let obj = node
                    .get_head_object_system(&txn)
                    .await
                    .map_err(|_| HandlerError::NotFound)?;
                serde_json::to_value(obj).map_err(HandlerError::from)?
            }
            NodeKind::Entity => {
                let obj = node
                    .get_head_object_entity(&txn)
                    .await
                    .map_err(|_| HandlerError::NotFound)?;
                serde_json::to_value(obj).map_err(HandlerError::from)?
            }
        }
    };

    let reply = GetReply { item: object };
    Ok(warp::reply::json(&reply))
}

pub async fn get(
    node_id: String,
    pg: PgPool,
    token: String,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(&txn, "nodes", &node_id, &claim.billing_account_id).await?;
    authorize(&txn, &claim.user_id, &"nodes", "get").await?;

    let object = Node::get(&txn, &node_id)
        .await
        .map_err(HandlerError::from)?;

    let item = serde_json::to_value(object).map_err(HandlerError::from)?;

    let reply = GetReply { item };
    Ok(warp::reply::json(&reply))
}
