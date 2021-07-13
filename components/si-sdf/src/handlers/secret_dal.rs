use crate::handlers::{authorize, validate_tenancy, HandlerError};
use serde::{Deserialize, Serialize};
use si_data::{NatsConn, PgPool};
use si_model::{
    PublicKey, Secret, SecretAlgorithm, SecretKind, SecretObjectType, SecretVersion, SiClaims,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetPublicKeyReply {
    pub public_key: PublicKey,
}

pub async fn get_public_key(
    claim: SiClaims,
    pg: PgPool,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    let public_key = PublicKey::get_current(&txn, &claim.billing_account_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = GetPublicKeyReply { public_key };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub crypted: Vec<u8>,
    pub key_pair_id: String,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretReply {
    pub secret: Secret,
}

pub async fn create_secret(
    claim: SiClaims,
    request: CreateSecretRequest,
    pg: PgPool,
    nats_conn: NatsConn,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;
    let nats = nats_conn.transaction();

    authorize(&txn, &claim.user_id, "secretDal", "createSecret").await?;

    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;
    validate_tenancy(
        &txn,
        "key_pairs",
        &request.key_pair_id,
        &claim.billing_account_id,
    )
    .await?;

    let secret = Secret::new(
        &txn,
        &nats,
        request.name,
        request.object_type,
        request.kind,
        request.crypted,
        request.key_pair_id,
        request.version,
        request.algorithm,
        request.workspace_id,
    )
    .await
    .map_err(HandlerError::from)?;

    txn.commit().await.map_err(HandlerError::from)?;
    nats.commit().await.map_err(HandlerError::from)?;

    let reply = CreateSecretReply { secret };
    Ok(warp::reply::json(&reply))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretsForWorkspaceRequest {
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListSecretsForWorkspaceReply {
    pub list: Vec<Secret>,
}

pub async fn list_secrets_for_workspace(
    claim: SiClaims,
    request: ListSecretsForWorkspaceRequest,
    pg: PgPool,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut conn = pg.get().await.map_err(HandlerError::from)?;
    let txn = conn.transaction().await.map_err(HandlerError::from)?;

    authorize(&txn, &claim.user_id, "secretDal", "listSecretsForWorkspace").await?;

    validate_tenancy(
        &txn,
        "workspaces",
        &request.workspace_id,
        &claim.billing_account_id,
    )
    .await?;

    let list = Secret::list_for_workspace(&txn, request.workspace_id)
        .await
        .map_err(HandlerError::from)?;

    let reply = ListSecretsForWorkspaceReply { list };
    Ok(warp::reply::json(&reply))
}
