use si_data::Db;

use crate::handlers::{authorize, HandlerError};
use crate::models;

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
    authorize(&db, &user_id, &billing_account_id).await?;

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
