use crate::data::{Connection, Db};

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models;
use crate::models::change_set::ChangeSet;
use crate::models::entity::Entity;
use crate::models::node::{
    Node, ObjectPatchReply, ObjectPatchRequest, PatchConfiguredByReply, PatchConfiguredByRequest,
    PatchIncludeSystemReply, PatchOp, PatchReply, PatchRequest, PatchSetPositionReply,
    PatchSetPositionRequest,
};
use crate::models::ops::{OpEntitySet, OpReply, OpRequest, OpSetName};

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
            .map_err(|e| warp::reject::not_found())?
    } else {
        node.get_head_object(&db)
            .await
            .map_err(|e| warp::reject::not_found())?
    };
    tracing::error!(?object, "got the obj");

    let reply = models::GetReply { item: object };
    Ok(warp::reply::json(&reply))
}
