use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use sodiumoxide::crypto::box_::{self, PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    pk, standard_model_accessor_ro, DalContext, HistoryEvent, HistoryEventError, Timestamp,
    TransactionsError, Workspace, WorkspaceError, WorkspacePk,
};

mod key_pair_box_public_key_serde;
mod key_pair_box_secret_key_serde;

const PUBLIC_KEY_GET_CURRENT: &str = include_str!("./queries/public_key_get_current.sql");
const KEY_PAIR_GET_BY_PK: &str = include_str!("queries/key_pair_get_by_pk.sql");

#[derive(Error, Debug)]
pub enum KeyPairError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("no current key pair found when one was expected")]
    NoCurrentKeyPair,
    #[error("Invalid workspace: {0}")]
    InvalidWorkspace(WorkspacePk),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type KeyPairResult<T> = Result<T, KeyPairError>;

pk!(KeyPairPk);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct KeyPair {
    pk: KeyPairPk,
    name: String,
    workspace_pk: WorkspacePk,
    #[serde(with = "key_pair_box_public_key_serde")]
    public_key: BoxPublicKey,
    #[serde(with = "key_pair_box_secret_key_serde")]
    secret_key: BoxSecretKey,
    created_lamport_clock: u64,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl KeyPair {
    pub fn pk(&self) -> KeyPairPk {
        self.pk
    }

    pub async fn new(ctx: &DalContext, name: impl AsRef<str>) -> KeyPairResult<Self> {
        let name = name.as_ref();
        let (public_key, secret_key) = box_::gen_keypair();

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM key_pair_create_v1($1, $2, $3, $4)",
                &[
                    &name,
                    &ctx.tenancy().workspace_pk(),
                    &encode_public_key(&public_key),
                    &encode_secret_key(&secret_key),
                ],
            )
            .await?;

        // Inlined `finish_create_from_row`

        let json: serde_json::Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;

        // HistoryEvent won't be accessible by any tenancy (null tenancy_workspace_pk)
        let _history_event = HistoryEvent::new(
            ctx,
            "key_pair.create".to_owned(),
            "Key Pair created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;

        Ok(object)
    }

    pub async fn get_by_pk(ctx: &DalContext, pk: KeyPairPk) -> KeyPairResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(KEY_PAIR_GET_BY_PK, &[&pk])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        Ok(serde_json::from_value(json)?)
    }

    pub async fn get_current(ctx: &DalContext) -> KeyPairResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(PUBLIC_KEY_GET_CURRENT, &[&ctx.tenancy().workspace_pk()])
            .await?;

        let json: serde_json::Value = row.try_get("object")?;
        Ok(serde_json::from_value(json)?)
    }

    standard_model_accessor_ro!(name, String);
    standard_model_accessor_ro!(workspace_pk, WorkspacePk);
    standard_model_accessor_ro!(public_key, BoxPublicKey);
    standard_model_accessor_ro!(secret_key, BoxSecretKey);
    standard_model_accessor_ro!(created_lamport_clock, u64);

    pub async fn workspace(&self, ctx: &DalContext) -> KeyPairResult<Workspace> {
        Workspace::get_by_pk(ctx, &self.workspace_pk)
            .await
            .map_err(Box::new)?
            .ok_or(KeyPairError::InvalidWorkspace(self.workspace_pk))
    }
}

fn encode_public_key(key: &BoxPublicKey) -> String {
    general_purpose::STANDARD_NO_PAD.encode(key.as_ref())
}

fn encode_secret_key(key: &BoxSecretKey) -> String {
    general_purpose::STANDARD_NO_PAD.encode(key.as_ref())
}

/// A database-persisted libsodium box public key.
///
/// This type only contains the public half of the underlying key pair and is therefore safe to
/// expose via external API.
///
/// The field "public_key" is base64 encoded into a string when this struct is serialized, and
/// decoded when deserialized. Thus, the DAL "PublicKey" (this struct) must be used for transport
/// between SI components rather than the nested "BoxPublicKey".
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicKey {
    pk: KeyPairPk,
    name: String,
    /// This field is base64 encoded into a string. Consumers will have to base64 decode it.
    #[serde(with = "key_pair_box_public_key_serde")]
    public_key: BoxPublicKey,
    created_lamport_clock: u64,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl PublicKey {
    pub async fn get_current(ctx: &DalContext) -> KeyPairResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(PUBLIC_KEY_GET_CURRENT, &[&ctx.tenancy().workspace_pk()])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        Ok(serde_json::from_value(json)?)
    }

    pub fn pk(&self) -> &KeyPairPk {
        &self.pk
    }

    pub fn public_key(&self) -> &BoxPublicKey {
        &self.public_key
    }
}
