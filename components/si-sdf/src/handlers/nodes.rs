use si_data::Db;

use crate::handlers::{authorize, HandlerError};
use crate::models;
use crate::models::change_set::ChangeSet;
use crate::models::entity::ops::{OpReply, OpRequest, OpSetString};
use crate::models::entity::Entity;
use crate::models::node::{Node, PatchReply, PatchRequest};

#[tracing::instrument(level = "trace", target = "nodes::create")]
pub async fn create(
    db: Db,
    user_id: String,
    billing_account_id: String,
    organization_id: String,
    workspace_id: String,
    change_set_id: String,
    edit_session_id: String,
    request: models::node::CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, &user_id, &billing_account_id, "node", "create").await?;

    let node = models::node::Node::new(
        &db,
        request.name,
        request.kind,
        request.object_type,
        billing_account_id,
        organization_id,
        workspace_id,
        change_set_id,
        edit_session_id,
        user_id,
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
    user_id: String,
    billing_account_id: String,
    organization_id: String,
    workspace_id: String,
    change_set_id: String,
    edit_session_id: String,
    request: PatchRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, &user_id, &billing_account_id, "node", "patch").await?;

    let node = Node::get(&db, &node_id, &billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    let entity: Entity = node
        .get_head_object(&db)
        .await
        .map_err(HandlerError::from)?;
    let mut change_set: ChangeSet = ChangeSet::get(&db, &change_set_id, &billing_account_id)
        .await
        .map_err(HandlerError::from)?;

    let _op = match request {
        PatchRequest::Op(OpRequest::SetString(op_request)) => OpSetString::new(
            &db,
            &entity.id,
            &op_request.pointer,
            &op_request.value,
            op_request.override_system,
            billing_account_id,
            organization_id,
            workspace_id,
            change_set_id,
            edit_session_id,
            user_id,
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
