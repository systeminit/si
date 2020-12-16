use crate::{
    data::{NatsTxn, NatsTxnError, PgTxn},
    models::{
        list_model, next_update_clock, KeyPair, KeyPairError, ListReply, ModelError,
        OrderByDirection, PageToken, Query, SiStorable, UpdateClockError,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sodiumoxide::crypto::{
    box_::{PublicKey, SecretKey},
    sealedbox,
};
use strum_macros::{Display, EnumString};
use thiserror::Error;
use tracing::error;

macro_rules! enum_impls {
    ($ty:ty) => {
        impl From<$ty> for String {
            fn from(value: $ty) -> Self {
                value.to_string()
            }
        }

        impl std::convert::TryFrom<&str> for $ty {
            type Error = strum::ParseError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                <Self as std::str::FromStr>::from_str(value)
            }
        }

        impl std::convert::TryFrom<String> for $ty {
            type Error = strum::ParseError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                <Self as std::str::FromStr>::from_str(value.as_str())
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum SecretError {
    #[error("error when decrypting crypted secret")]
    DecryptionFailed,
    #[error("error in key pair: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("secret is not found")]
    NotFound,
    #[error("pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("update clock: {0}")]
    UpdateClock(#[from] UpdateClockError),
}

pub type SecretResult<T> = Result<T, SecretError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub crypted: Vec<u8>,
    pub key_pair_id: String,
    pub version: SecretVersion,
    pub algorithm: SecretAlgorithm,
    pub organization_id: String,
    pub workspace_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub item: Secret,
}

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum SecretObjectType {
    Credential,
}

enum_impls!(SecretObjectType);

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum SecretKind {
    DockerHub,
    AwsAccessKey,
    HelmRepo,
}

enum_impls!(SecretKind);

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum SecretVersion {
    V1,
}

enum_impls!(SecretVersion);

impl Default for SecretVersion {
    fn default() -> Self {
        Self::V1
    }
}

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", try_from = "String", into = "String")]
#[strum(serialize_all = "camelCase")]
pub enum SecretAlgorithm {
    Sealedbox,
}

enum_impls!(SecretAlgorithm);

impl Default for SecretAlgorithm {
    fn default() -> Self {
        Self::Sealedbox
    }
}

/// A reference to a database-persisted encrypted secret.
///
/// This type does not contain any encypted information nor any encryption metadata and is
/// therefore safe to expose via external API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    pub id: String,
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub si_storable: SiStorable,
}

impl Secret {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        object_type: SecretObjectType,
        kind: SecretKind,
        crypted: impl Into<Vec<u8>>,
        key_pair_id: impl Into<String>,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
        workspace_id: String,
    ) -> SecretResult<Self> {
        Ok(EncryptedSecret::new(
            txn,
            nats,
            name,
            object_type,
            kind,
            crypted,
            key_pair_id,
            version,
            algorithm,
            workspace_id,
        )
        .await?
        .into())
    }

    pub async fn get(txn: &PgTxn<'_>, id: impl AsRef<str> + std::fmt::Debug) -> SecretResult<Self> {
        let id = id.as_ref();
        let row = txn
            .query_one("SELECT object FROM secret_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn list(
        txn: &PgTxn<'_>,
        tenant_id: impl Into<String>,
        query: Option<Query>,
        page_size: Option<u32>,
        order_by: Option<String>,
        order_by_direction: Option<OrderByDirection>,
        page_token: Option<PageToken>,
    ) -> SecretResult<ListReply> {
        let tenant_id = tenant_id.into();
        let reply = list_model(
            txn,
            "secrets",
            tenant_id,
            query,
            page_size,
            order_by,
            order_by_direction,
            page_token,
        )
        .await?;
        Ok(reply)
    }
}

/// A database-persisted encrypted secret.
///
/// This type contains the raw encrypted payload as well as the necessary encryption metadata and
/// therefore should *only* be used internally when decrypting secrets for use by `veritech`.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EncryptedSecret {
    pub id: String,
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    #[serde(with = "crypted_serde")]
    crypted: Vec<u8>,
    key_pair_id: String,
    version: SecretVersion,
    algorithm: SecretAlgorithm,
    pub si_storable: SiStorable,
}

impl EncryptedSecret {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        object_type: SecretObjectType,
        kind: SecretKind,
        crypted: impl Into<Vec<u8>>,
        key_pair_id: impl Into<String>,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
        workspace_id: String,
    ) -> SecretResult<Self> {
        let name = name.into();
        let crypted = crypted.into();
        let key_pair_id = key_pair_id.into();

        let update_clock = next_update_clock(&workspace_id).await?;

        let row = txn
            .query_one(
                "SELECT object FROM secret_create_v1($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
                &[
                    &name,
                    &object_type.to_string(),
                    &kind.to_string(),
                    &encode_crypted(&crypted),
                    &key_pair_id,
                    &version.to_string(),
                    &algorithm.to_string(),
                    &workspace_id,
                    &update_clock.epoch,
                    &update_clock.update_count,
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: Self = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn get(txn: &PgTxn<'_>, id: impl AsRef<str> + std::fmt::Debug) -> SecretResult<Self> {
        let id = id.as_ref();
        let row = txn
            .query_one("SELECT object FROM secret_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn decrypt(self, txn: &PgTxn<'_>) -> SecretResult<DecryptedSecret> {
        let key_pair =
            KeyPair::get(txn, &self.key_pair_id, &self.si_storable.billing_account_id).await?;

        self.into_decrypted(&key_pair.public_key, &key_pair.secret_key)
    }

    fn into_decrypted(self, pk: &PublicKey, sk: &SecretKey) -> SecretResult<DecryptedSecret> {
        // Explicitly match on (version, algorithm) tuple to ensure that any new
        // versions/algorithms will trigger a compilation failure
        match (self.version, self.algorithm) {
            (SecretVersion::V1, SecretAlgorithm::Sealedbox) => Ok(DecryptedSecret {
                id: self.id,
                name: self.name,
                object_type: self.object_type,
                kind: self.kind,
                message: serde_json::from_slice(
                    &sealedbox::open(&self.crypted, pk, sk)
                        .map_err(|_| SecretError::DecryptionFailed)?,
                )?,
            }),
        }
    }
}

impl From<EncryptedSecret> for Secret {
    fn from(value: EncryptedSecret) -> Self {
        Self {
            id: value.id,
            name: value.name,
            object_type: value.object_type,
            kind: value.kind,
            si_storable: value.si_storable,
        }
    }
}

/// A secret that has been decrypted.
///
/// This type is returned by calling `EncyptedSecret.decrypt(&db).await?` which contains the raw
/// decrypted message, and without the encrypted payload and other encyption metadata. It is not
/// persistable and is only intended to be used internally when passing secrets into `veritech`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecryptedSecret {
    pub id: String,
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    pub message: Value,
}

fn encode_crypted(crypted: &[u8]) -> String {
    let s = base64::encode_config(crypted.as_ref(), base64::STANDARD_NO_PAD);
    s
}

mod crypted_serde {
    use super::encode_crypted;
    use serde::{self, Deserialize, Deserializer, Serializer};

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
        let buffer =
            base64::decode_config(s, base64::STANDARD_NO_PAD).map_err(serde::de::Error::custom)?;
        Ok(buffer.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UpdateClock;
    use sodiumoxide::crypto::{box_, sealedbox};

    mod encrypted_secret {
        use super::*;

        fn encrypted_secret(
            name: impl Into<String>,
            object_type: SecretObjectType,
            kind: SecretKind,
            crypted: impl Into<Vec<u8>>,
        ) -> EncryptedSecret {
            let name = name.into();
            let crypted = crypted.into();
            let id = "secret:abc123".to_string();
            let billing_account_id = "billingAccount:789".to_string();
            let organization_id = "organization:0123".to_string();
            let workspace_id = "workspace:456".to_string();
            let tenant_ids = vec![
                billing_account_id.clone(),
                organization_id.clone(),
                workspace_id.clone(),
            ];

            EncryptedSecret {
                id: id.clone(),
                name,
                object_type,
                kind,
                crypted,
                key_pair_id: "keyPair:def456".to_string(),
                version: Default::default(),
                algorithm: Default::default(),
                si_storable: SiStorable {
                    type_name: "secret".to_string(),
                    object_id: id,
                    billing_account_id,
                    organization_id,
                    workspace_id,
                    tenant_ids,
                    created_by_user_id: None,
                    update_clock: UpdateClock {
                        epoch: 1,
                        update_count: 0,
                    },
                    deleted: false,
                },
            }
        }

        fn crypt<T>(value: &T, pk: &PublicKey) -> Vec<u8>
        where
            T: ?Sized + Serialize,
        {
            sealedbox::seal(&serde_json::to_vec(value).expect("failed to serialize"), pk)
        }

        #[test]
        fn into_decrypted() {
            sodiumoxide::init().expect("crypto failed to init");
            let (pk, sk) = box_::gen_keypair();

            let message = serde_json::json!({"username": "Kings's X", "password": "Black Flag"});
            let crypted = crypt(&message, &pk);

            let encrypted = encrypted_secret(
                "kings-x",
                SecretObjectType::Credential,
                SecretKind::DockerHub,
                crypted,
            );
            let decrypted = encrypted
                .into_decrypted(&pk, &sk)
                .expect("could not decrypt secret");

            assert_eq!("secret:abc123", decrypted.id);
            assert_eq!("kings-x", decrypted.name);
            assert_eq!(SecretObjectType::Credential, decrypted.object_type);
            assert_eq!(SecretKind::DockerHub, decrypted.kind);
            assert_eq!(message, decrypted.message);
        }
    }

    mod secret_object_type {
        use super::*;

        #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            object_type: SecretObjectType,
        }

        fn str() -> &'static str {
            r#"{"objectType":"credential"}"#
        }

        fn invalid() -> &'static str {
            r#"{"objectType":"nope"}"#
        }

        fn object() -> Object {
            Object {
                object_type: SecretObjectType::Credential,
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
            match serde_json::from_str::<Object>(invalid()) {
                Err(_) => assert!(true),
                Ok(_) => panic!("deserialize should not succeed"),
            }
        }
    }

    mod secret_kind {
        use super::*;

        #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            kind: SecretKind,
        }

        fn str() -> &'static str {
            r#"{"kind":"dockerHub"}"#
        }

        fn invalid() -> &'static str {
            r#"{"kind":"nope"}"#
        }

        fn object() -> Object {
            Object {
                kind: SecretKind::DockerHub,
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
            match serde_json::from_str::<Object>(invalid()) {
                Err(_) => assert!(true),
                Ok(_) => panic!("deserialize should not succeed"),
            }
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
            match serde_json::from_str::<Object>(invalid()) {
                Err(_) => assert!(true),
                Ok(_) => panic!("deserialize should not succeed"),
            }
        }

        #[test]
        fn default() {
            // This test is intended to catch if and when we update the default variant for this
            // type
            assert_eq!(SecretVersion::V1, SecretVersion::default())
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
            match serde_json::from_str::<Object>(invalid()) {
                Err(_) => assert!(true),
                Ok(_) => panic!("deserialize should not succeed"),
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
