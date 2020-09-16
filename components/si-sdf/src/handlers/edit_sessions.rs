use si_data::Db;

use crate::handlers::{authorize, HandlerError};
use crate::models::edit_session::{CreateReply, CreateRequest, EditSession};

pub async fn create(
    change_set_id: String,
    db: Db,
    user_id: String,
    billing_account_id: String,
    organization_id: String,
    workspace_id: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    authorize(&db, &user_id, &billing_account_id, "editSession", "create").await?;

    let edit_session = EditSession::new(
        &db,
        request.name,
        change_set_id,
        billing_account_id,
        organization_id,
        workspace_id,
        user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply { item: edit_session };
    Ok(warp::reply::json(&reply))
}
