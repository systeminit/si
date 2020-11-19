use crate::data::{Connection, Db};

use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::edit_session::{
    CreateReply, CreateRequest, EditSession, PatchReply, PatchRequest,
};
use crate::models::get_model;

pub async fn create(
    change_set_id: String,
    db: Db,
    nats: Connection,
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
        &nats,
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

pub async fn patch(
    _change_set_id: String,
    edit_session_id: String,
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
        "editSession",
        "patch",
    )
    .await?;

    let edit_session: EditSession = get_model(&db, &edit_session_id, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;
    match request {
        PatchRequest::Cancel(_) => edit_session
            .cancel(&db, &nats, None)
            .await
            .map_err(HandlerError::from)?,
    }

    let reply = PatchReply::Cancel(edit_session);
    Ok(warp::reply::json(&reply))
}
