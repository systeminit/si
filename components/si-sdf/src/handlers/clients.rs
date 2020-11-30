use crate::{
    data::{Connection, Db},
    handlers::{authenticate, authorize, HandlerError},
    models::client::{CreateReply, CreateRequest, Client},
};

#[tracing::instrument(level = "trace", target = "clients::create")]

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
        "client",
        "create",
    )
    .await?;

    let client = Client::new(
        &db,
        &nats,
        request.name,
        request.object_type,
        request.kind,
        request.version,
        claim.billing_account_id,
        request.organization_id,
        request.workspace_id,
        claim.user_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply { item: client };

    Ok(warp::reply::json(&reply))
}



    // let signing_key = JwtKeyPrivate::get_jwt_signing_key(&db, &secret_key)
    //     .await
    //     .map_err(HandlerError::from)?;
    // let si_claims = SiClaims {
    //     user_id: user.id.clone(),
    //     billing_account_id: user.si_storable.billing_account_id.clone(),
    // };
    // let claims = Claims::with_custom_claims(si_claims, Duration::from_days(1)) // make it longer
    //     .with_audience("https://app.systeminit.com")
    //     .with_issuer("https://app.systeminit.com")
    //     .with_subject(user.id.clone());
    // let jwt = signing_key
    //     .sign(claims)
    //     .map_err(|err| HandlerError::JwtClaim(format!("{}", err)))?;