use std::fmt;

use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sodiumoxide::crypto::secretbox::Nonce;
use sodiumoxide::crypto::{
    box_::{PublicKey, SecretKey},
    sealedbox,
};
use strum::{AsRefStr, Display, EnumString};
use thiserror::Error;

use si_data_pg::PgError;
use telemetry::prelude::*;
use veritech_client::SensitiveContainer;

use crate::crypto::{SymmetricCryptoError, SymmetricCryptoService};
use crate::diagram::node::HistoryEventMetadata;
use crate::standard_model::objects_from_rows;
use crate::{
    impl_standard_model,
    key_pair::KeyPairPk,
    pk,
    standard_model::{self, TypeHint},
    standard_model_accessor, standard_model_accessor_ro, DalContext, HistoryEvent,
    HistoryEventError, KeyPair, KeyPairError, StandardModel, StandardModelError, Timestamp,
    Visibility,
};
use crate::{ActorView, HistoryActor, Tenancy, TransactionsError, UserPk};

const LIST_SECRET_DEFINITIONS: &str = include_str!("queries/secrets/list_secret_definitions.sql");

/// Error type for Secrets.
#[remain::sorted]
#[derive(Error, Debug)]
pub enum SecretError {
    #[error("error when decrypting crypted secret")]
    DecryptionFailed,
    #[error("error deserializing message: {0}")]
    DeserializeMessage(#[source] serde_json::Error),
    #[error("donkey not found for secret")]
    DonkeyNotFound,
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("key pair not found for secret")]
    KeyPairNotFound,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("symmetric crypto error: {0}")]
    SymmetricCrypto(#[from] SymmetricCryptoError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
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
    pk: SecretPk,
    id: SecretId,
    name: String,
    key_pair_pk: KeyPairPk,
    definition: String,
    description: Option<String>,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    created_by: Option<UserPk>,
    updated_by: Option<UserPk>,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Secret,
    pk: SecretPk,
    id: SecretId,
    table_name: "secrets",
    history_event_label_base: "secret",
    history_event_message_name: "Secret"
}

impl Secret {
    standard_model_accessor_ro!(name, str);

    // Update the underlying `encrypted_secrets` table rather than attempting to update the
    // `secrets` view
    pub async fn set_name(
        &mut self,
        ctx: &DalContext,
        value: impl Into<String>,
    ) -> SecretResult<()> {
        let value = value.into();
        let updated_at = standard_model::update(
            ctx,
            "encrypted_secrets",
            "name",
            self.id(),
            &value,
            TypeHint::Text,
        )
        .await?;
        let _history_event = HistoryEvent::new(
            ctx,
            Self::history_event_label(vec!["updated"]),
            Self::history_event_message("updated"),
            &serde_json::json!({"pk": self.pk, "field": "name", "value": &value}),
        )
        .await?;
        self.timestamp.updated_at = updated_at;
        self.name = value;

        Ok(())
    }

    // Once created, these object fields are to be considered immutable
    standard_model_accessor_ro!(definition, String);
    standard_model_accessor_ro!(description, Option<String>);

    pub async fn key_pair(&self, ctx: &DalContext) -> SecretResult<KeyPair> {
        Ok(KeyPair::get_by_pk(ctx, self.key_pair_pk).await?)
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
    nonce: Nonce,
    donkey_hash: [u8; 32],
    #[serde(with = "crypted_serde")]
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
                    &encode_crypted(double_crypted.as_slice()),
                    &version.as_ref(),
                    &algorithm.as_ref(),
                    &key_pair_pk,
                    &nonce.as_ref(),
                    &key_hash.as_ref(),
                    &maybe_actor,
                ],
            )
            .await?;
        let object: Secret = standard_model::finish_create_from_row(ctx, row).await?;

        Ok(object)
    }

    standard_model_accessor!(name, String, SecretResult);
    standard_model_accessor_ro!(description, Option<String>);

    // Once created, these object fields are to be considered immutable
    standard_model_accessor_ro!(definition, String);
    standard_model_accessor_ro!(version, SecretVersion);
    standard_model_accessor_ro!(algorithm, SecretAlgorithm);

    /// Decrypts the encrypted secret with its associated [`KeyPair`] and returns a
    /// [`DecryptedSecret`].
    pub async fn decrypt(self, ctx: &DalContext) -> SecretResult<DecryptedSecret> {
        let key_pair = self.key_pair(ctx).await?;

        self.into_decrypted(
            key_pair.public_key(),
            key_pair.secret_key(),
            ctx.services_context().symmetric_crypto_service(),
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
                let donkey_decrypted = symmetric_crypto_service.decrypt(
                    &self.crypted,
                    &self.nonce,
                    &self.donkey_hash,
                )?;

                let message = serde_json::from_slice(
                    &sealedbox::open(&donkey_decrypted, pkey, skey)
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
    widget_kind: String,
    widget_options: Option<String>,
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

fn encode_crypted(crypted: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(crypted)
}

mod crypted_serde {
    use base64::{engine::general_purpose, Engine};
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::encode_crypted;

    pub fn serialize<S>(crypted: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = encode_crypted(crypted);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let buffer = general_purpose::STANDARD_NO_PAD
            .decode(s)
            .map_err(serde::de::Error::custom)?;
        Ok(buffer)
    }
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

            let (double_crypted, nonce, donkey_hash) =
                symmetric_crypto_service.encrypt(crypted.as_ref());

            EncryptedSecret {
                pk: SecretPk::NONE,
                id: SecretId::NONE,
                name,
                definition,
                description,
                key_pair_pk: KeyPairPk::NONE,
                nonce,
                donkey_hash: *donkey_hash,
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
