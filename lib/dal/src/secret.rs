//! This module contains [`Secret`], which is a reference to an underlying [`EncryptedSecret`].

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_crypto::{SymmetricCryptoError, SymmetricCryptoService, SymmetricNonce};
use si_data_pg::PgError;
use si_events::ContentHash;
use si_events::EncryptedSecretKey;
use si_hash::Hash;
use si_layer_cache::LayerDbError;
use sodiumoxide::crypto::box_::{PublicKey, SecretKey};
use sodiumoxide::crypto::sealedbox;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use thiserror::Error;
use ulid::Ulid;
use veritech_client::SensitiveContainer;

use crate::key_pair::KeyPairPk;
use crate::layer_db_types::{SecretContent, SecretContentV1};
use crate::prop::PropError;
use crate::serde_impls::base64_bytes_serde;
use crate::serde_impls::nonce_serde;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    id, ChangeSetError, DalContext, HistoryActor, HistoryEventError, KeyPair, KeyPairError,
    SchemaVariantError, StandardModelError, Timestamp, TransactionsError, UserPk,
};

mod algorithm;
mod definition_view;
mod event;
mod view;

pub use algorithm::SecretAlgorithm;
pub use algorithm::SecretVersion;
pub use definition_view::SecretDefinitionView;
pub use definition_view::SecretDefinitionViewError;
pub use event::SecretCreatedPayload;
pub use event::SecretUpdatedPayload;
pub use view::SecretView;
pub use view::SecretViewError;
pub use view::SecretViewResult;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("error when decrypting crypted secret")]
    DecryptionFailed,
    #[error("error deserializing message: {0}")]
    DeserializeMessage(#[source] serde_json::Error),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("encrypted secret not found for corresponding secret: {0}")]
    EncryptedSecretNotFound(SecretId),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("key pair not found for secret")]
    KeyPairNotFound,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

#[allow(missing_docs)]
pub type SecretResult<T> = Result<T, SecretError>;

id!(SecretId);

/// A reference to an [`EncryptedSecret`] with metadata.
///
/// This type does not contain any encrypted information nor any encryption metadata and is
/// therefore safe to expose via external API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Secret {
    id: SecretId,
    key: EncryptedSecretKey,
    #[serde(flatten)]
    timestamp: Timestamp,
    created_by: Option<UserPk>,
    updated_by: Option<UserPk>,
    name: String,
    definition: String,
    description: Option<String>,
}

impl From<Secret> for SecretContentV1 {
    fn from(value: Secret) -> Self {
        Self {
            key: value.key,
            timestamp: value.timestamp,
            created_by: value.created_by,
            updated_by: value.updated_by,
            name: value.name,
            definition: value.definition,
            description: value.description,
        }
    }
}

impl Secret {
    #[allow(missing_docs)]
    pub fn assemble(id: SecretId, inner: SecretContentV1) -> Self {
        Self {
            id,
            key: inner.key,
            timestamp: inner.timestamp,
            created_by: inner.created_by,
            updated_by: inner.updated_by,
            name: inner.name,
            definition: inner.definition,
            description: inner.description,
        }
    }

    /// Creates a new [`Secret`] with a corresponding [`EncryptedSecret`].
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        definition: impl Into<String>,
        description: Option<String>,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<Self> {
        let user = match ctx.history_actor() {
            HistoryActor::SystemInit => None,
            HistoryActor::User(user_pk) => Some(*user_pk),
        };

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let secret_id = id.into();

        // Generate a key for the underlying encrypted secret.
        let key = Self::generate_key(ctx, secret_id)?;

        let content = SecretContentV1 {
            key,
            timestamp: Timestamp::now(),
            created_by: user,
            updated_by: user,
            name: name.into(),
            definition: definition.into(),
            description,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(SecretContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Secret(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        // Root --> Secret Category --> Secret (this)
        let secret_category_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Secret)
            .await?;
        workspace_snapshot
            .add_edge(
                secret_category_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                id,
            )
            .await?;

        let secret = Self::assemble(secret_id, content);

        // After creating the secret on the graph, create an underlying encrypted secret and use
        // the key we assembled.
        EncryptedSecret::insert(ctx, key, crypted, key_pair_pk, version, algorithm).await?;

        Ok(secret)
    }

    /// Generates a key based on the [`Tenancy`](crate::Tenancy), [`SecretId`] and a newly generated
    /// [`Ulid`].
    ///
    /// A new key should be assembled anytime an [`EncryptedSecret`] is created or mutated. This
    /// method is purposefully owned by [`Secret`] to help ensure that we don't generate a key based
    /// on any encrypted contents or parameters to insert encrypted contents.
    fn generate_key(ctx: &DalContext, secret_id: SecretId) -> SecretResult<EncryptedSecretKey> {
        let new_ulid = ctx.change_set()?.generate_ulid()?;

        let mut hasher = blake3::Hasher::new();
        hasher.update(&ctx.tenancy().to_bytes());
        hasher.update(secret_id.to_string().as_bytes());
        hasher.update(new_ulid.to_string().as_bytes());

        Ok(hasher.finalize().into())
    }

    /// Returns the [`id`](SecretId).
    pub fn id(&self) -> SecretId {
        self.id
    }

    /// Returns a reference to the name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns a reference to the definition.
    pub fn definition(&self) -> &str {
        self.definition.as_ref()
    }

    /// Returns a reference to the description.
    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    /// Returns the key corresponding to the underlying [`EncryptedSecret`].
    pub fn key(&self) -> EncryptedSecretKey {
        self.key
    }

    /// Gets the [`Secret`] with a given [`SecretId`]. If a [`Secret`] is not found, then return an
    /// [`error`](SecretError).
    ///
    /// _Note:_ this does not contain the encrypted or sensitive bits and is safe for external use.
    pub async fn get_by_id_or_error(ctx: &DalContext, id: SecretId) -> SecretResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let ulid: Ulid = id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(ulid).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: SecretContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(ulid))?;

        // NOTE(nick): if we had a v2, then there would be migration logic here.
        let SecretContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    /// Lists all [`Secrets`](Secret) in the current [`snapshot`](crate::WorkspaceSnapshot).
    pub async fn list(ctx: &DalContext) -> SecretResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut secrets = vec![];
        let secret_category_node_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Secret)
            .await?;

        let secret_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                secret_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut hashes = vec![];
        for index in secret_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Secret)?;
            hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let contents: HashMap<ContentHash, SecretContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match contents.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick): if we had a v2, then there would be migration logic here.
                    let SecretContent::V1(inner) = content;

                    secrets.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(secrets)
    }

    /// Updates the metadata for the [`Secret`], but not the encrypted contents.
    pub async fn update_metadata(
        self,
        ctx: &DalContext,
        name: impl Into<String>,
        description: Option<String>,
    ) -> SecretResult<Self> {
        self.modify(ctx, |s| {
            s.name = name.into();
            s.description = description;
            match ctx.history_actor() {
                HistoryActor::SystemInit => {}
                HistoryActor::User(id) => {
                    s.updated_by = Some(*id);
                }
            }
            Ok(())
        })
        .await
    }

    /// Updates the underlying encrypted contents by generating a new key and inserting a new
    /// [`EncryptedSecret`].
    pub async fn update_encrypted_contents(
        self,
        ctx: &DalContext,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<Self> {
        // Generate a new key and insert a new encrypted secret.
        let new_key = Self::generate_key(ctx, self.id)?;
        EncryptedSecret::insert(ctx, new_key, crypted, key_pair_pk, version, algorithm).await?;

        // Now, update the key on the secret on the graph.
        // TODO(nick): ensure that the old encrypted secret gets garbage collected.
        self.modify(ctx, |s| {
            s.key = new_key;
            Ok(())
        })
        .await
    }

    /// Modifies the [`Secret`] and persists modifications as applicable.
    async fn modify<L>(self, ctx: &DalContext, lambda: L) -> SecretResult<Self>
    where
        L: FnOnce(&mut Self) -> SecretResult<()>,
    {
        let mut secret = self;

        let before = SecretContentV1::from(secret.clone());
        lambda(&mut secret)?;
        let updated = SecretContentV1::from(secret.clone());

        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(SecretContent::V1(updated.clone()).into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;

            ctx.workspace_snapshot()?
                .update_content(ctx.change_set()?, secret.id.into(), hash)
                .await?;
        }

        Ok(secret)
    }
}

/// The [`EncryptedSecret`] corresponding to an individual [`Secret`]. It contains sensitive, but
/// encrypted, information.
#[derive(Clone, Deserialize, Serialize)]
pub struct EncryptedSecret {
    user: Option<UserPk>,
    version: SecretVersion,
    algorithm: SecretAlgorithm,
    key_hash: Hash,
    key_pair_pk: KeyPairPk,
    // TODO(nick): confirm that base64 de/ser works as intended.
    #[serde(with = "nonce_serde")]
    nonce: SymmetricNonce,
    // TODO(nick): confirm that base64 de/ser works as intended.
    #[serde(with = "base64_bytes_serde")]
    crypted: Vec<u8>,
}

impl fmt::Debug for EncryptedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedSecret")
            .field("user", &self.user)
            .field("version", &self.version)
            .field("algorithm", &self.algorithm)
            .field("key_hash", &self.key_hash)
            .finish_non_exhaustive()
    }
}

impl EncryptedSecret {
    /// This _private_ method is used by [`Secret`] during its creation or when the user wishes
    /// to "change" its corresponding encrypted contents.
    async fn insert(
        ctx: &DalContext,
        key: EncryptedSecretKey,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<()> {
        let user = match ctx.history_actor() {
            HistoryActor::SystemInit => None,
            HistoryActor::User(user_pk) => Some(*user_pk),
        };

        let (double_crypted, nonce, key_hash) = ctx.symmetric_crypto_service().encrypt(crypted);

        let value = Self {
            user,
            key_pair_pk,
            nonce,
            key_hash: key_hash.to_owned(),
            crypted: double_crypted,
            version,
            algorithm,
        };

        ctx.layer_db()
            .encrypted_secret()
            .write(
                key,
                Arc::new(value),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        Ok(())
    }

    /// Gets the [`EncryptedSecret`] by a given [`SecretId`].
    ///
    /// _Warning:_ sensitive, but encrypted, contents will be returned.
    pub async fn get_by_key(
        ctx: &DalContext,
        key: EncryptedSecretKey,
    ) -> SecretResult<Option<Self>> {
        Ok(ctx.layer_db().encrypted_secret().try_read_as(&key).await?)
    }

    /// Decrypts the encrypted secret with its associated [`KeyPair`] and returns a
    /// [`DecryptedSecret`].
    pub async fn decrypt(self, ctx: &DalContext) -> SecretResult<DecryptedSecret> {
        let key_pair = self.key_pair(ctx).await?;

        self.into_decrypted(
            key_pair.public_key(),
            key_pair.secret_key(),
            ctx.symmetric_crypto_service(),
        )
    }

    fn into_decrypted(
        self,
        pkey: &PublicKey,
        skey: &SecretKey,
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> SecretResult<DecryptedSecret> {
        // Explicitly match on (version, algorithm) tuple to ensure that any new
        // versions/algorithms will trigger a compilation failure
        match (self.version, self.algorithm) {
            (SecretVersion::V1, SecretAlgorithm::Sealedbox) => {
                let symmetric_decrypted =
                    symmetric_crypto_service.decrypt(&self.crypted, &self.nonce, &self.key_hash)?;

                let message = serde_json::from_slice(
                    &sealedbox::open(&symmetric_decrypted, pkey, skey)
                        .map_err(|_| SecretError::DecryptionFailed)?,
                )
                .map_err(SecretError::DeserializeMessage)?;

                Ok(DecryptedSecret { message })
            }
        }
    }

    /// Gets the [`KeyPair`] corresponding to the [`KeyPairPk`] on the [`EncryptedSecret`].
    pub async fn key_pair(&self, ctx: &DalContext) -> SecretResult<KeyPair> {
        Ok(KeyPair::get_by_pk(ctx, self.key_pair_pk).await?)
    }
}

/// This type corresponds to a secret that has been decrypted. It is returned by calling
/// [`EncryptedSecret::decrypt`], which contains the raw decrypted message, and without the
/// encrypted payload and other metadata. It is not persist-able and is only intended to be used
/// internally when passing secrets through to "cyclone".
///
/// _Note:_ we're being a bit careful here as to which traits are derived in an effort to minimize
/// leaking sensitive data.
#[derive(Serialize)]
pub struct DecryptedSecret {
    message: Value,
}

impl DecryptedSecret {
    pub(crate) fn message(&self) -> SensitiveContainer<Value> {
        self.message.clone().into()
    }
}

impl fmt::Debug for DecryptedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecryptedSecret").finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use sodiumoxide::crypto::box_;

    use super::*;

    fn encrypted_secret(
        crypted: impl Into<Vec<u8>>,
        symmetric_crypto_service: &SymmetricCryptoService,
    ) -> EncryptedSecret {
        let crypted = crypted.into();
        let (double_crypted, nonce, key_hash) = symmetric_crypto_service.encrypt(crypted.as_ref());

        EncryptedSecret {
            user: None,
            key_pair_pk: KeyPairPk::NONE,
            nonce,
            key_hash: *key_hash,
            crypted: double_crypted,
            version: Default::default(),
            algorithm: Default::default(),
        }
    }

    fn crypt<T>(value: &T, pkey: &PublicKey) -> Vec<u8>
    where
        T: ?Sized + Serialize,
    {
        sealedbox::seal(
            &serde_json::to_vec(value).expect("failed to serialize value"),
            pkey,
        )
    }

    #[test]
    fn into_decrypted() {
        sodiumoxide::init().expect("crypto failed to init");
        let (pkey, skey) = box_::gen_keypair();

        let message =
            serde_json::json!({"username": "The Cadillac Three", "password": "Slow Rollin"});
        let crypted = crypt(&message, &pkey);

        let service = SymmetricCryptoService::new(SymmetricCryptoService::generate_key(), vec![]);

        let encrypted = encrypted_secret(crypted, &service);
        let decrypted = encrypted
            .into_decrypted(&pkey, &skey, &service)
            .expect("could not decrypt secret");

        assert_eq!(message, decrypted.message);
    }
}
