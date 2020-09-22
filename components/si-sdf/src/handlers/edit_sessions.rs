use crate::data::Db;

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::edit_session::{CreateReply, CreateRequest, EditSession};

pub async fn create(
    change_set_id: String,
    db: Db,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "editSession",
        "create",
    )
    .await?;

    let edit_session = EditSession::new(
        &db,
        request.name,
        change_set_id,
        claim.billing_account_id,
        request.organization_id,
        request.workspace_id,
        claim.user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply { item: edit_session };
    Ok(warp::reply::json(&reply))
}
