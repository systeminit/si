use crate::data::{NatsConn, PgPool};
use crate::handlers::{authenticate, authorize, validate_tenancy, HandlerError};
use crate::models::edit_session::{
    CreateReply, CreateRequest, EditSession, PatchReply, PatchRequest,
};
use crate::veritech::Veritech;

pub async fn create(
    change_set_id: String,
    pg: PgPool,
    nats_conn: NatsConn,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.pool.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    let claim = authenticate(&txn, &token).await?;
    validate_tenancy(
        &txn,
        "change_sets",
        &change_set_id,
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

    authorize(&txn, &claim.user_id, "editSession", "create").await?;

    let edit_session = EditSession::new(
        &txn,
        &nats,
        request.name,
        change_set_id,
        request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = CreateReply { item: edit_session };
    Ok(warp::reply::json(&reply))
}

pub async fn patch(
    change_set_id: String,
    edit_session_id: String,
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
    validate_tenancy(
        &txn,
        "change_sets",
        &change_set_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "edit_sessions",
        &edit_session_id,
        &claim.billing_account_id,
    )
    .await?;
    authorize(&txn, &claim.user_id, "editSession", "patch").await?;

    let edit_session = EditSession::get(&txn, &edit_session_id)
        .await
        .map_err(HandlerError::from)?;
    match request {
        PatchRequest::Cancel(_) => edit_session
            .cancel(&pg, &txn, &nats_conn, &nats, &veritech, None)
            .await
            .map_err(HandlerError::from)?,
    }

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = PatchReply::Cancel(edit_session);
    Ok(warp::reply::json(&reply))
}
