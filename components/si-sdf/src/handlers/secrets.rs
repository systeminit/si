use crate::{
    data::{Connection, Db},
    handlers::{authenticate, authorize, HandlerError},
    models::secret::{CreateReply, CreateRequest, Secret},
};

#[tracing::instrument(level = "trace", target = "secrets::create")]
pub async fn create(
    db: Db,
    nats: Connection,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "secret",
        "create",
    )
    .await?;

    let secret = Secret::new(
        &db,
        &nats,
        request.name,
        request.object_type,
        request.kind,
        request.crypted,
        request.key_pair_id,
        request.version,
        request.algorithm,
        claim.billing_account_id,
        request.organization_id,
        request.workspace_id,
        claim.user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply { item: secret };

    Ok(warp::reply::json(&reply))
}
