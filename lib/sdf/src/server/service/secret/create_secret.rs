use axum::Json;
use dal::{
    key_pair::KeyPairId, EncryptedSecret, HistoryActor, Secret, SecretAlgorithm, SecretKind,
    SecretObjectType, SecretVersion, Visibility, WorkspaceId, WriteTenancy,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{Authorization, NatsTxn, PgRwTxn};

use super::SecretResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretRequest {
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub crypted: Vec<u8>,
    pub key_pair_id: KeyPairId,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateSecretResponse {
    pub secret: Secret,
}

pub async fn create_secret(
    mut txn: PgRwTxn,
    mut nats: NatsTxn,
    Authorization(claim): Authorization,
    Json(request): Json<CreateSecretRequest>,
) -> SecretResult<Json<CreateSecretResponse>> {
    let txn = txn.start().await?;
    let nats = nats.start().await?;

    let history_actor = HistoryActor::from(claim.user_id);
    let write_tenancy = WriteTenancy::new_workspace(request.workspace_id);

    let secret = EncryptedSecret::new(
        &txn,
        &nats,
        &write_tenancy,
        &request.visibility,
        &history_actor,
        request.name,
        request.object_type,
        request.kind,
        &request.crypted,
        request.key_pair_id,
        request.version,
        request.algorithm,
        claim.billing_account_id,
    )
    .await?;

    txn.commit().await?;
    nats.commit().await?;

    Ok(Json(CreateSecretResponse { secret }))
}
