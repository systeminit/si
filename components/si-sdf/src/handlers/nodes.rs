use crate::data::Db;

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models;
use crate::models::change_set::ChangeSet;
use crate::models::entity::Entity;
use crate::models::node::{Node, PatchReply, PatchRequest};
use crate::models::ops::{OpEntitySetString, OpReply, OpRequest};

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn create(
    db: Db,
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
        &db,
        request.name,
        request.kind,
        request.object_type,
        claim.billing_account_id,
        request.organization_id,
        request.workspace_id,
        request.change_set_id,
        request.edit_session_id,
        claim.user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = models::node::CreateReply { item: node };
    Ok(warp::reply::json(&reply))
}

#[tracing::instrument(level = "trace", target = "nodes::object::patch")]
pub async fn patch(
    node_id: String,
    db: Db,
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

    let node = Node::get(&db, &node_id, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let entity: Entity = node
        .get_head_object(&db)
        .await
        .map_err(HandlerError::from)?;

    let mut change_set: ChangeSet =
        ChangeSet::get(&db, &request.change_set_id, &claim.billing_account_id)
            .await
            .map_err(HandlerError::from)?;

    let _op = match request.op {
        OpRequest::EntitySetString(op_request) => OpEntitySetString::new(
            &db,
            &entity.id,
            &op_request.pointer,
            &op_request.value,
            op_request.override_system,
            claim.billing_account_id,
            request.organization_id,
            request.workspace_id,
            request.change_set_id,
            request.edit_session_id,
            claim.user_id,
        )
        .await
        .map_err(HandlerError::from)?,
    };

    let item_ids = change_set
        .execute(&db, true)
        .await
        .map_err(HandlerError::from)?;
    let reply = PatchReply::Op(OpReply { item_ids });
    Ok(warp::reply::json(&reply))
}
