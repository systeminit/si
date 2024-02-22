use std::fmt;

use base64::{engine::general_purpose, Engine};
use content_store::{Store, StoreError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_crypto::{SymmetricCryptoError, SymmetricCryptoService, SymmetricNonce};
use si_hash::Hash;
use sodiumoxide::crypto::{
    box_::{PublicKey, SecretKey},
    sealedbox,
};
use strum::{AsRefStr, Display, EnumDiscriminants, EnumString};
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;
use veritech_client::SensitiveContainer;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    history_event::HistoryEventMetadata,
    impl_standard_model,
    key_pair::KeyPairPk,
    pk,
    property_editor::schema::PropertyEditorPropWidgetKind,
    serde_impls::{base64_bytes_serde, nonce_serde},
    standard_model::{self, objects_from_rows, TypeHint},
    standard_model_accessor, standard_model_accessor_ro, ActorView, ChangeSetPk, DalContext,
    HistoryActor, HistoryEvent, HistoryEventError, KeyPair, KeyPairError, StandardModel,
    StandardModelError, Tenancy, Timestamp, TransactionsError, UserPk, Visibility, WsEvent,
    WsEventResult, WsPayload,
};

const LIST_SECRET_DEFINITIONS: &str = include_str!("queries/secrets/list_secret_definitions.sql");

/// Error type for Secrets.
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretError {
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error("error when decrypting crypted secret")]
    DecryptionFailed,
    #[error("error deserializing message: {0}")]
    DeserializeMessage(#[source] serde_json::Error),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("key pair not found for secret")]
    KeyPairNotFound,
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("content store error: {0}")]
    Store(#[from] StoreError),
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

/// Result type for Secrets.
pub type SecretResult<T> = Result<T, SecretError>;

pk!(SecretPk);
pk!(SecretId);

/// A reference to a database-persisted encrypted secret.
///
/// This type does not contain any encrypted information nor any encryption metadata and is
/// therefore safe to expose via external API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Secret {
    id: SecretId,

    // TODO(nick): evaluate how these three fields will work with the new engine.
    #[serde(flatten)]
    timestamp: Timestamp,
    created_by: Option<UserPk>,
    updated_by: Option<UserPk>,

    pk: SecretPk,
    key_pair_pk: KeyPairPk,
    name: String,
    definition: String,
    description: Option<String>,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SecretContent {
    V1(SecretContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SecretContentV1 {
    timestamp: Timestamp,
    created_by: Option<UserPk>,
    updated_by: Option<UserPk>,

    pk: SecretPk,
    key_pair_pk: KeyPairPk,
    name: String,
    definition: String,
    description: Option<String>,
}

impl From<Secret> for SecretContentV1 {
    fn from(value: Secret) -> Self {
        Self {
            timestamp: value.timestamp,
            created_by: value.created_by,
            updated_by: value.updated_by,
            pk: value.pk,
            key_pair_pk: value.key_pair_pk,
            name: value.name,
            definition: value.definition,
            description: value.description,
        }
    }
}

impl Secret {
    pub fn assemble(id: SecretId, inner: SecretContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            created_by: inner.created_by,
            updated_by: inner.updated_by,
            pk: inner.pk,
            key_pair_pk: inner.key_pair_pk,
            name: inner.name,
            definition: inner.definition,
            description: inner.description,
        }
    }

    // TODO(nick): to maintain API compatibility with main, we need "EncryptedSecret::new" to create
    // this. We may want the opposite to happen in the future. We should decide this after the
    // switchover. Let's consume the object for now to help ensure the underlying "encrypted secret"
    // is not used where it shouldn't be.
    pub async fn new(
        ctx: &DalContext,
        new_encrypted_secret: EncryptedSecret,
    ) -> SecretResult<Self> {
        let content = SecretContentV1 {
            timestamp: new_encrypted_secret.timestamp,
            created_by: new_encrypted_secret.created_by,
            updated_by: new_encrypted_secret.updated_by,
            pk: new_encrypted_secret.pk,
            key_pair_pk: new_encrypted_secret.key_pair_pk,
            name: new_encrypted_secret.name,
            definition: new_encrypted_secret.definition,
            description: new_encrypted_secret.description,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&SecretContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Secret(hash))?;

        // Attach secret to the category.
        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.write().await;
            workspace_snapshot.add_node(node_weight)?;

            // Root --> Secret Category --> Secret (this)
            let secret_category_id =
                workspace_snapshot.get_category_node(None, CategoryNodeKind::Secret)?;
            workspace_snapshot.add_edge(
                secret_category_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )?;
        }

        let secret = Self::assemble(id.into(), content);

        Ok(secret)
    }

    pub fn id(&self) -> SecretId {
        self.id
    }

    // Update the underlying `encrypted_secrets` table rather than attempting to update the
    // `secrets` view
    pub async fn set_name(
        &mut self,
        ctx: &DalContext,
        value: impl Into<String>,
    ) -> SecretResult<()> {
        let value = value.into();
        let _updated_at = standard_model::update(
            ctx,
            "encrypted_secrets",
            "name",
            &self.id(),
            &value,
            TypeHint::Text,
        )
        .await?;
        let _history_event = HistoryEvent::new(
            ctx,
            EncryptedSecret::history_event_label(vec!["updated"]),
            EncryptedSecret::history_event_message("updated"),
            &serde_json::json!({"pk": self.pk, "field": "name", "value": &value}),
        )
        .await?;
        self.name = value;

        Ok(())
    }

    pub async fn key_pair(&self, ctx: &DalContext) -> SecretResult<KeyPair> {
        Ok(KeyPair::get_by_pk(ctx, self.key_pair_pk).await?)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecretCreatedPayload {
    secret_id: SecretId,
    change_set_pk: ChangeSetPk,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SecretUpdatedPayload {
    secret_id: SecretId,
    change_set_pk: ChangeSetPk,
}

impl WsEvent {
    pub async fn secret_created(ctx: &DalContext, secret_id: SecretId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SecretCreated(SecretCreatedPayload {
                secret_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }

    pub async fn secret_updated(ctx: &DalContext, secret_id: SecretId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::SecretUpdated(SecretUpdatedPayload {
                secret_id,
                change_set_pk: ctx.visibility().change_set_pk,
            }),
        )
        .await
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretView {
    pub id: SecretId,
    pub name: String,
    pub definition: String,
    pub description: Option<String>,
    pub created_info: HistoryEventMetadata,
    pub updated_info: Option<HistoryEventMetadata>,
}

impl SecretView {
    pub async fn from_secret(ctx: &DalContext, secret: Secret) -> SecretResult<Self> {
        let created_info = {
            let actor = match secret.created_by {
                None => HistoryActor::SystemInit,
                Some(user_pk) => HistoryActor::from(user_pk),
            };

            let view = ActorView::from_history_actor(ctx, actor).await?;

            HistoryEventMetadata {
                actor: view,
                timestamp: secret.timestamp.created_at,
            }
        };

        let updated_info = {
            let actor = match secret.updated_by {
                None => HistoryActor::SystemInit,
                Some(user_pk) => HistoryActor::from(user_pk),
            };

            let view = ActorView::from_history_actor(ctx, actor).await?;

            if secret.timestamp.created_at == secret.timestamp.updated_at {
                None
            } else {
                Some(HistoryEventMetadata {
                    actor: view,
                    timestamp: secret.timestamp.updated_at,
                })
            }
        };

        Ok(Self {
            id: secret.id,
            name: secret.name,
            definition: secret.definition,
            description: secret.description,
            created_info,
            updated_info,
        })
    }
}

impl From<EncryptedSecret> for Secret {
    fn from(value: EncryptedSecret) -> Self {
        Self {
            pk: value.pk,
            id: value.id,
            name: value.name,
            key_pair_pk: value.key_pair_pk,
            definition: value.definition,
            description: value.description,
            timestamp: value.timestamp,
            created_by: value.created_by,
            updated_by: None,
        }
    }
}

/// A database-persisted encrypted secret.
///
/// This type contains the raw encrypted payload as well as the necessary encryption metadata and
/// should therefore should *only* be used internally when decrypting secrets for use by Cyclone.
///
/// NOTE: Other than creating a new encrypted secret, any external API will likely want to use
/// the [`Secret`] type which does not expose extra encryption information.
#[derive(Clone, Deserialize, Serialize)]
pub struct EncryptedSecret {
    pk: SecretPk,
    id: SecretId,
    name: String,
    definition: String,
    description: Option<String>,
    key_pair_pk: KeyPairPk,
    #[serde(with = "nonce_serde")]
    nonce: SymmetricNonce,
    key_hash: Hash,
    #[serde(with = "base64_bytes_serde")]
    crypted: Vec<u8>,
    version: SecretVersion,
    algorithm: SecretAlgorithm,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    created_by: Option<UserPk>,
    updated_by: Option<UserPk>,
    #[serde(flatten)]
    visibility: Visibility,
}

impl fmt::Debug for EncryptedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EncryptedSecret")
            .field("pk", &self.pk)
            .field("id", &self.id)
            .field("name", &self.name)
            .field("definition", &self.definition)
            .field("description", &self.description)
            .field("version", &self.version)
            .field("algorithm", &self.algorithm)
            .field("key_hash", &self.key_hash)
            .field("tenancy", &self.tenancy)
            .field("timestamp", &self.timestamp)
            .field("visibility", &self.visibility)
            .finish_non_exhaustive()
    }
}

impl_standard_model! {
    model: EncryptedSecret,
    pk: SecretPk,
    id: SecretId,
    table_name: "encrypted_secrets",
    history_event_label_base: "encrypted_secret",
    history_event_message_name: "Encrypted Secret"
}

impl EncryptedSecret {
    /// Creates a new encrypted secret and returns a corresponding [`Secret`] representation.
    #[allow(clippy::too_many_arguments, clippy::new_ret_no_self)]
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        definition: String,
        description: Option<String>,
        crypted: &[u8],
        key_pair_pk: KeyPairPk,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
    ) -> SecretResult<Secret> {
        let name = name.as_ref();

        let maybe_actor = match ctx.history_actor() {
            HistoryActor::SystemInit => None,
            HistoryActor::User(user_pk) => Some(user_pk),
        };

        let (double_crypted, nonce, key_hash) = ctx.symmetric_crypto_service().encrypt(crypted);

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM encrypted_secret_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
                &[
                    ctx.tenancy(),
                    ctx.visibility(),
                    &name,
                    &definition,
                    &description,
                    &base64_encode_bytes(double_crypted.as_slice()),
                    &version.as_ref(),
                    &algorithm.as_ref(),
                    &key_pair_pk,
                    &base64_encode_bytes(nonce.as_ref()),
                    &key_hash.to_string(),
                    &maybe_actor,
                ],
            )
            .await?;
        let object: Self = standard_model::finish_create_from_row(ctx, row).await?;

        let referential_secret = Secret::new(ctx, object).await?;

        Ok(referential_secret)
    }

    standard_model_accessor!(name, String, SecretResult);
    standard_model_accessor!(description, Option<String>, SecretResult);
    standard_model_accessor!(version, Enum(SecretVersion), SecretResult);
    standard_model_accessor!(algorithm, Enum(SecretAlgorithm), SecretResult);
    standard_model_accessor!(updated_by, Option<Pk(UserPk)>, SecretResult);
    standard_model_accessor!(key_pair_pk, Pk(KeyPairPk), SecretResult);

    // Once created, this object field is immutable
    standard_model_accessor_ro!(definition, String);

    pub async fn set_crypted(&mut self, ctx: &DalContext, value: Vec<u8>) -> SecretResult<()> {
        let (double_crypted, nonce, key_hash) = ctx.symmetric_crypto_service().encrypt(&value);
        let updated_at = standard_model::update(
            ctx,
            "encrypted_secrets",
            "crypted",
            self.id(),
            &base64_encode_bytes(double_crypted.as_slice()),
            TypeHint::Text,
        )
        .await?;
        standard_model::update(
            ctx,
            "encrypted_secrets",
            "nonce",
            self.id(),
            &base64_encode_bytes(nonce.as_ref()),
            TypeHint::Text,
        )
        .await?;
        standard_model::update(
            ctx,
            "encrypted_secrets",
            "key_hash",
            self.id(),
            &key_hash.to_string(),
            TypeHint::Text,
        )
        .await?;

        let _history_event = HistoryEvent::new(
            ctx,
            Self::history_event_label(vec!["updated"]),
            Self::history_event_message("updated"),
            &serde_json::json!({"pk": self.pk, "field": "crypted", "value": "encrypted"}),
        )
        .await?;
        self.timestamp.updated_at = updated_at;
        self.crypted = value;

        Ok(())
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

                Ok(DecryptedSecret {
                    name: self.name,
                    definition: self.definition,
                    message,
                })
            }
        }
    }

    pub async fn key_pair(&self, ctx: &DalContext) -> SecretResult<KeyPair> {
        Ok(KeyPair::get_by_pk(ctx, self.key_pair_pk).await?)
    }
}

/// A secret that has been decrypted.
///
/// This type is returned by calling `EncryptedSecret.decrypt(&txn).await?` which contains the raw
/// decrypted message, and without the encrypted payload and other metadata. It is not persistable
/// and is only intended to be used internally when passing secrets through to Cyclone.
//
// NOTE: We're being a bit careful here as to which traits are drrived in an effort to minimize
// leaking sensitive data.
#[derive(Serialize)]
pub struct DecryptedSecret {
    name: String,
    definition: String,
    message: Value,
}

impl DecryptedSecret {
    /// Gets a reference to the decrypted secret's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn message(&self) -> SensitiveContainer<Value> {
        self.message.clone().into()
    }

    /// Gets the decrypted secret's definition.
    pub fn definition(&self) -> &str {
        self.definition.as_ref()
    }
}

impl fmt::Debug for DecryptedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecryptedSecret")
            .field("name", &self.name)
            .field("definition", &self.definition)
            .finish_non_exhaustive()
    }
}

/// The version of encryption used to encrypt a secret.
#[remain::sorted]
#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SecretVersion {
    /// Version 1 of the encryption
    V1,
}

impl Default for SecretVersion {
    fn default() -> Self {
        Self::V1
    }
}

/// The algorithm used to encrypt a secret.
#[remain::sorted]
#[derive(
    AsRefStr, Clone, Copy, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SecretAlgorithm {
    /// The "sealedbox" encryption algorithm, provided by libsodium
    Sealedbox,
}

impl Default for SecretAlgorithm {
    fn default() -> Self {
        Self::Sealedbox
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SecretFormDataView {
    name: String,
    kind: String,
    widget_kind: PropertyEditorPropWidgetKind,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SecretDefinitionView {
    pub secret_definition: String,
    form_data: Vec<SecretFormDataView>,
}

impl SecretDefinitionView {
    pub async fn list(ctx: &DalContext) -> SecretResult<Vec<SecretDefinitionView>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(LIST_SECRET_DEFINITIONS, &[ctx.tenancy(), ctx.visibility()])
            .await?;

        Ok(objects_from_rows(rows)?)
    }
}

fn base64_encode_bytes(bytes: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod encrypted_secret {
        use sodiumoxide::crypto::box_;

        use crate::WorkspacePk;

        use super::*;

        fn encrypted_secret(
            name: impl Into<String>,
            definition: String,
            description: Option<String>,
            crypted: impl Into<Vec<u8>>,
            symmetric_crypto_service: &SymmetricCryptoService,
            wid: WorkspacePk,
        ) -> EncryptedSecret {
            let name = name.into();
            let crypted = crypted.into();

            let (double_crypted, nonce, key_hash) =
                symmetric_crypto_service.encrypt(crypted.as_ref());

            EncryptedSecret {
                pk: SecretPk::NONE,
                id: SecretId::NONE,
                name,
                definition,
                description,
                key_pair_pk: KeyPairPk::NONE,
                nonce,
                key_hash: *key_hash,
                crypted: double_crypted,
                version: Default::default(),
                algorithm: Default::default(),
                tenancy: Tenancy::new(wid),
                timestamp: Timestamp::now(),
                created_by: None,
                updated_by: None,
                visibility: Visibility::new_head(false),
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

            let service =
                SymmetricCryptoService::new(SymmetricCryptoService::generate_key(), vec![]);

            let encrypted = encrypted_secret(
                "the-cadillac-three",
                "dockerHub".to_owned(),
                None,
                crypted,
                &service,
                WorkspacePk::NONE,
            );
            let decrypted = encrypted
                .into_decrypted(&pkey, &skey, &service)
                .expect("could not decrypt secret");

            assert_eq!("the-cadillac-three", decrypted.name);
            assert_eq!("dockerHub", decrypted.definition);
            assert_eq!(message, decrypted.message);
        }
    }

    mod secret_version {
        use super::*;

        #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            version: SecretVersion,
        }

        fn str() -> &'static str {
            r#"{"version":"v1"}"#
        }

        fn invalid() -> &'static str {
            r#"{"version":"nope"}"#
        }

        fn object() -> Object {
            Object {
                version: SecretVersion::V1,
            }
        }

        #[test]
        fn serialize() {
            assert_eq!(
                str(),
                serde_json::to_string(&object()).expect("failed to serialize")
            );
        }

        #[test]
        fn deserialize() {
            assert_eq!(
                object(),
                serde_json::from_str(str()).expect("failed to deserialize")
            );
        }

        #[test]
        fn deserialize_invalid() {
            if serde_json::from_str::<Object>(invalid()).is_ok() {
                panic!("deserialize should not succeed")
            }
        }

        #[test]
        fn default() {
            // This test is intended to catch if and when we update the default variant for this
            // type
            assert_eq!(SecretAlgorithm::Sealedbox, SecretAlgorithm::default())
        }
    }

    mod secret_algorithm {
        use super::*;

        #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            algorithm: SecretAlgorithm,
        }

        fn str() -> &'static str {
            r#"{"algorithm":"sealedbox"}"#
        }

        fn invalid() -> &'static str {
            r#"{"algorithm":"nope"}"#
        }

        fn object() -> Object {
            Object {
                algorithm: SecretAlgorithm::Sealedbox,
            }
        }

        #[test]
        fn serialize() {
            assert_eq!(
                str(),
                serde_json::to_string(&object()).expect("failed to serialize")
            );
        }

        #[test]
        fn deserialize() {
            assert_eq!(
                object(),
                serde_json::from_str(str()).expect("failed to deserialize")
            );
        }

        #[test]
        fn deserialize_invalid() {
            if serde_json::from_str::<Object>(invalid()).is_ok() {
                panic!("deserialize should not succeed")
            }
        }

        #[test]
        fn default() {
            // This test is intended to catch if and when we update the default variant for this
            // type
            assert_eq!(SecretAlgorithm::Sealedbox, SecretAlgorithm::default())
        }
    }
}
