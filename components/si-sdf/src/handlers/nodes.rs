use crate::data::{Connection, Db};

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models;
use crate::models::change_set::ChangeSet;
use crate::models::edge::{Edge, EdgeKind};
use crate::models::node::{
    Node, ObjectPatchReply, ObjectPatchRequest, PatchConfiguredByReply, PatchIncludeSystemReply,
    PatchOp, PatchReply, PatchRequest, PatchSetPositionReply, SyncResourceReply,
};
use crate::models::ops::{
    OpEntityAction, OpEntityDelete, OpEntitySet, OpReply, OpRequest, OpSetName,
};

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn create(
    db: Db,
    nats: Connection,
    token: String,
    request: models::node::CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "node",
        "create",
    )
    .await?;

    let node = models::node::Node::new(
        db,
        nats,
        request.name,
        request.kind,
        request.object_type,
        claim.billing_account_id,
        request.organization_id,
        request.workspace_id,
        request.change_set_id,
        request.edit_session_id,
        Some(claim.user_id),
        request.system_ids,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = models::node::CreateReply { item: node };
    Ok(warp::reply::json(&reply))
}

#[tracing::instrument(level = "trace")]
pub async fn patch(
    node_id: String,
    db: Db,
    nats: Connection,
    token: String,
    request: PatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "node",
        "patch",
    )
    .await?;

    let mut node: Node = Node::get(&db, &node_id, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = match request.op {
        PatchOp::IncludeSystem(system_req) => {
            let edge = node
                .include_in_system(&db, &nats, &system_req.system_id)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::IncludeSystem(PatchIncludeSystemReply { edge })
        }
        PatchOp::ConfiguredBy(configured_by_req) => {
            let edge = node
                .configured_by(&db, &nats, configured_by_req.node_id)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::ConfiguredBy(PatchConfiguredByReply { edge })
        }
        PatchOp::SetPosition(set_position_req) => {
            node.set_position(
                set_position_req.context.clone(),
                set_position_req.position.clone(),
            );
            models::upsert_model(&db, &nats, &node.id, &node)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::SetPosition(PatchSetPositionReply {
                context: set_position_req.context,
                position: set_position_req.position,
            })
        }
        PatchOp::SyncResource(sync_resource_req) => {
            let resource = node
                .sync_resource(&db, &nats, sync_resource_req.system_id)
                .await
                .map_err(HandlerError::from)?;
            PatchReply::SyncResource(SyncResourceReply { resource })
        }
    };

    Ok(warp::reply::json(&reply))
}

#[tracing::instrument(level = "trace")]
pub async fn object_patch(
    node_id: String,
    db: Db,
    nats: Connection,
    token: String,
    request: ObjectPatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "node",
        "patch",
    )
    .await?;

    let node = Node::get(&db, &node_id, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let entity_id = node.get_object_id(&db).await.map_err(HandlerError::from)?;

    let mut change_set: ChangeSet =
        ChangeSet::get(&db, &request.change_set_id, &claim.billing_account_id)
            .await
            .map_err(HandlerError::from)?;

    match request.op {
        OpRequest::EntitySet(op_request) => {
            OpEntitySet::new(
                &db,
                &nats,
                &entity_id,
                op_request.path,
                op_request.value,
                op_request.override_system,
                claim.billing_account_id,
                request.organization_id,
                request.workspace_id,
                request.change_set_id,
                request.edit_session_id,
                claim.user_id,
            )
            .await
            .map_err(HandlerError::from)?;
        }
        OpRequest::NameSet(op_request) => {
            OpSetName::new(
                &db,
                &nats,
                &entity_id,
                op_request.value,
                claim.billing_account_id,
                request.organization_id,
                request.workspace_id,
                request.change_set_id,
                request.edit_session_id,
                claim.user_id,
            )
            .await
            .map_err(HandlerError::from)?;
        }
        OpRequest::EntityAction(op_request) => {
            OpEntityAction::new(
                &db,
                &nats,
                &entity_id,
                &op_request.action,
                &op_request.system_id,
                claim.billing_account_id,
                request.organization_id,
                request.workspace_id,
                request.change_set_id,
                request.edit_session_id,
                claim.user_id,
            )
            .await
            .map_err(HandlerError::from)?;
        }
        OpRequest::EntityDelete(op_request) => {
            OpEntityDelete::new(
                &db,
                &nats,
                &entity_id,
                claim.billing_account_id.clone(),
                request.organization_id.clone(),
                request.workspace_id.clone(),
                request.change_set_id.clone(),
                request.edit_session_id.clone(),
                claim.user_id.clone(),
            )
            .await
            .map_err(HandlerError::from)?;
            if op_request.cascade {
                let successors =
                    Edge::all_successor_edges_by_object_id(&db, EdgeKind::Configures, &entity_id)
                        .await
                        .map_err(HandlerError::from)?;
                // TODO: When we support changing completely the trees, or support more than one
                // configures, we will have to deal with this less heavy handidly (in particular, we're
                // going to have to look at each successor to make sure it doesn't have more than one
                // predecessor). But for now, we can just delete them all too.
                for successor in successors.iter() {
                    OpEntityDelete::new(
                        &db,
                        &nats,
                        &successor.head_vertex.object_id,
                        claim.billing_account_id.clone(),
                        request.organization_id.clone(),
                        request.workspace_id.clone(),
                        request.change_set_id.clone(),
                        request.edit_session_id.clone(),
                        claim.user_id.clone(),
                    )
                    .await
                    .map_err(HandlerError::from)?;
                }
            }
        }
    }

    let item_ids = change_set
        .execute(&db, &nats, true)
        .await
        .map_err(HandlerError::from)?;
    let reply = ObjectPatchReply::Op(OpReply { item_ids });
    Ok(warp::reply::json(&reply))
}

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn get_object(
    node_id: String,
    db: Db,
    token: String,
    request: models::GetRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    tracing::error!("you did call it, right?");
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "node",
        "objectGet",
    )
    .await?;

    let node = models::node::Node::get(&db, node_id, claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let object: serde_json::Value = if let Some(change_set_id) = request.change_set_id {
        node.get_object_projection(&db, change_set_id)
            .await
            .map_err(|_e| warp::reject::not_found())?
    } else {
        node.get_head_object(&db)
            .await
            .map_err(|_e| warp::reject::not_found())?
    };
    tracing::error!(?object, "got the obj");

    let reply = models::GetReply { item: object };
    Ok(warp::reply::json(&reply))
}
