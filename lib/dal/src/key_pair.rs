use base64::{
    Engine,
    engine::general_purpose,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_crypto::{
    SymmetricCryptoError,
    SymmetricCryptoService,
    SymmetricNonce,
};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use si_db::{
    HistoryEvent,
    key_pair::{
        GET_BY_PK,
        PUBLIC_KEY_GET_CURRENT,
    },
};
use si_events::Timestamp;
use si_hash::Hash;
use sodiumoxide::crypto::box_::{
    self,
    PublicKey as BoxPublicKey,
    SecretKey as BoxSecretKey,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    DalContext,
    TransactionsError,
    Workspace,
    WorkspaceError,
    WorkspacePk,
    getter,
    serde_impls::{
        base64_bytes_serde,
        nonce_serde,
    },
};
mod key_pair_box_public_key_serde;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum KeyPairError {
    #[error("invalid secret key bytes")]
    InvalidSecretKeyBytes,
    #[error("Invalid workspace: {0}")]
    InvalidWorkspace(WorkspacePk),
    #[error("key pair not found: {0}")]
    KeyPairNotFound(KeyPairPk),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("no current key pair found when one was expected")]
    NoCurrentKeyPair,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::SiDbError),
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("cannot get key for different workspace")]
    UnauthorizedKeyAccess,
    #[error("workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
}

pub type KeyPairResult<T> = Result<T, KeyPairError>;

pub use si_id::KeyPairPk;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct KeyPair {
    pk: KeyPairPk,
    name: String,
    workspace_pk: WorkspacePk,
    public_key: BoxPublicKey,
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
        let (public_key, secret_key_crypted, secret_key_nonce, secret_key_key_hash) =
            Self::gen_keys(ctx.symmetric_crypto_service());

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM key_pair_create_v1($1, $2, $3, $4, $5, $6)",
                &[
                    &name,
                    &ctx.tenancy().workspace_pk_opt(),
                    &base64_encode_bytes(public_key.as_ref()),
                    &base64_encode_bytes(secret_key_crypted.as_slice()),
                    &base64_encode_bytes(secret_key_nonce.as_ref()),
                    &secret_key_key_hash.to_string(),
                ],
            )
            .await?;

        // Inlined `finish_create_from_row`
        let json: serde_json::Value = row.try_get("object")?;
        let object_row: KeyPairRow = serde_json::from_value(json)?;

        let object = object_row.decrypt_into(ctx.symmetric_crypto_service())?;

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
        let Some(row) = ctx.txns().await?.pg().query_opt(GET_BY_PK, &[&pk]).await? else {
            return Err(KeyPairError::KeyPairNotFound(pk));
        };
        let json: serde_json::Value = row.try_get("object")?;
        let key_pair_row: KeyPairRow = serde_json::from_value(json)?;

        if key_pair_row.workspace_pk != ctx.tenancy().workspace_pk()? {
            return Err(KeyPairError::UnauthorizedKeyAccess);
        }

        let key_pair = key_pair_row.decrypt_into(ctx.symmetric_crypto_service())?;
        Ok(key_pair)
    }

    getter!(name, String);
    getter!(workspace_pk, WorkspacePk);
    getter!(public_key, BoxPublicKey);
    getter!(secret_key, BoxSecretKey);
    getter!(created_lamport_clock, u64);

    pub async fn workspace(&self, ctx: &DalContext) -> KeyPairResult<Workspace> {
        Workspace::get_by_pk_opt(ctx, self.workspace_pk)
            .await
            .map_err(Box::new)?
            .ok_or(KeyPairError::InvalidWorkspace(self.workspace_pk))
    }

    fn gen_keys(
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> (BoxPublicKey, Vec<u8>, SymmetricNonce, &Hash) {
        let (public_key, secret_key) = box_::gen_keypair();

        let (secret_key_crypted, secret_key_nonce, secret_key_key_hash) =
            symmetric_crypto_service.encrypt(secret_key.as_ref());

        (
            public_key,
            secret_key_crypted,
            secret_key_nonce,
            secret_key_key_hash,
        )
    }
}

fn base64_encode_bytes(bytes: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(bytes)
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
            .query_one(PUBLIC_KEY_GET_CURRENT, &[&ctx.tenancy().workspace_pk_opt()])
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
struct KeyPairRow {
    pk: KeyPairPk,
    name: String,
    workspace_pk: WorkspacePk,
    #[serde(with = "key_pair_box_public_key_serde")]
    public_key: BoxPublicKey,
    #[serde(with = "nonce_serde")]
    secret_key_nonce: SymmetricNonce,
    secret_key_key_hash: Hash,
    #[serde(with = "base64_bytes_serde")]
    secret_key_crypted: Vec<u8>,
    created_lamport_clock: u64,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl KeyPairRow {
    fn decrypt_into(
        self,
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> KeyPairResult<KeyPair> {
        let secret_key_bytes = symmetric_crypto_service.decrypt(
            &self.secret_key_crypted,
            &self.secret_key_nonce,
            &self.secret_key_key_hash,
        )?;
        let secret_key = BoxSecretKey::from_slice(secret_key_bytes.as_slice())
            .ok_or(KeyPairError::InvalidSecretKeyBytes)?;

        Ok(KeyPair {
            pk: self.pk,
            name: self.name,
            workspace_pk: self.workspace_pk,
            public_key: self.public_key,
            secret_key,
            created_lamport_clock: self.created_lamport_clock,
            timestamp: self.timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use sodiumoxide::crypto::sealedbox;

    use super::*;

    fn key_pair_row(
        name: impl Into<String>,
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> KeyPairRow {
        let name = name.into();
        let (public_key, secret_key_crypted, secret_key_nonce, secret_key_key_hash) =
            KeyPair::gen_keys(symmetric_crypto_service);

        KeyPairRow {
            pk: KeyPairPk::NONE,
            name,
            workspace_pk: WorkspacePk::NONE,
            public_key,
            secret_key_nonce,
            secret_key_key_hash: *secret_key_key_hash,
            secret_key_crypted,
            created_lamport_clock: 0,
            timestamp: Timestamp::now(),
        }
    }

    fn symmetric_crypto_service() -> SymmetricCryptoService {
        SymmetricCryptoService::new(SymmetricCryptoService::generate_key(), vec![])
    }

    #[test]
    fn key_pair_row_decrypt_into() {
        sodiumoxide::init().expect("crypto failed to init");
        let symmetric_crypto_service = symmetric_crypto_service();

        let key_pair_row = key_pair_row("the-temperance-movement", &symmetric_crypto_service);

        let key_pair = key_pair_row
            .decrypt_into(&symmetric_crypto_service)
            .expect("failed to decrypt into key_kair");

        assert_eq!("the-temperance-movement", key_pair.name);

        // Use the fully decrypted key pair to make sure we can round trip a message
        let crypted = sealedbox::seal(b"it's a secret", key_pair.public_key());
        let message = sealedbox::open(&crypted, key_pair.public_key(), key_pair.secret_key())
            .expect("failed to decrypt test message with keys");
        assert_eq!("it's a secret".as_bytes(), message.as_slice());
    }
}
