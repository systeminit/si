use sodiumoxide::crypto::secretbox;

use crate::data::{Connection, Db};
use crate::handlers::{authenticate, authorize, HandlerError};
use crate::models::api_client::{CreateReply, CreateRequest};
use crate::models::ApiClient;

pub async fn create(
    db: Db,
    nats: Connection,
    secret_key: secretbox::Key,
    token: String,
    request: CreateRequest,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let claim = authenticate(&db, &token).await?;
    authorize(
        &db,
        &claim.user_id,
        &claim.billing_account_id,
        "apiClient",
        "create",
    )
    .await?;

    let (api_client, jwt) = ApiClient::new(
        &db,
        &nats,
        secret_key,
        request.name,
        request.kind,
        &claim.billing_account_id,
    )
    .await
    .map_err(HandlerError::from)?;

    let reply = CreateReply {
        api_client,
        token: jwt,
    };

    Ok(warp::reply::json(&reply))
}
