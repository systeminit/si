use crate::{
    data::{Connection, Db},
    models::{
        key_pair::KeyPair,
        {insert_model, KeyPairError, ModelError, SiStorable, SiStorableError},
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
    #[error("failed to deserialize decrypted message as json: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("error in key pair: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("secret is not found")]
    NotFound,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
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
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        object_type: SecretObjectType,
        kind: SecretKind,
        crypted: impl Into<Vec<u8>>,
        key_pair_id: impl Into<String>,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> SecretResult<Self> {
        Ok(EncryptedSecret::new(
            db,
            nats,
            name,
            object_type,
            kind,
            crypted,
            key_pair_id,
            version,
            algorithm,
            billing_account_id,
            organization_id,
            workspace_id,
            created_by_user_id,
        )
        .await?
        .into())
    }
}

/// A database-persisted encrypted secret.
///
/// This type contains the raw encrypted payload as well as the necessary encryption metadata and
/// therefore should *only* be used internally when decrypting secrets for use by `veritech`.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EncryptedSecret {
    pub id: String,
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    crypted: Vec<u8>,
    key_pair_id: String,
    version: SecretVersion,
    algorithm: SecretAlgorithm,
    pub si_storable: SiStorable,
}

impl EncryptedSecret {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        object_type: SecretObjectType,
        kind: SecretKind,
        crypted: impl Into<Vec<u8>>,
        key_pair_id: impl Into<String>,
        version: SecretVersion,
        algorithm: SecretAlgorithm,
        billing_account_id: String,
        organization_id: String,
        workspace_id: String,
        created_by_user_id: String,
    ) -> SecretResult<Self> {
        let name = name.into();
        let crypted = crypted.into();
        let key_pair_id = key_pair_id.into();

        let si_storable = SiStorable::new(
            db,
            "secret",
            billing_account_id,
            organization_id,
            workspace_id,
            Some(created_by_user_id),
        )
        .await?;
        let id = si_storable.object_id.clone();
        let model = Self {
            id,
            name,
            object_type,
            kind,
            crypted,
            key_pair_id,
            version,
            algorithm,
            si_storable,
        };
        insert_model(db, nats, &model.id, &model).await?;

        Ok(model)
    }

    // TODO(fnichol): this function is not yet used, so has `dead_code` allowed for the moment.
    // Once the `veritech` prep code needs this, drop the lint.
    #[allow(dead_code)]
    pub(crate) async fn decrypt(self, db: &Db) -> SecretResult<DecryptedSecret> {
        let key_pair =
            KeyPair::get(db, &self.key_pair_id, &self.si_storable.billing_account_id).await?;

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
pub(crate) struct DecryptedSecret {
    pub id: String,
    pub name: String,
    pub object_type: SecretObjectType,
    pub kind: SecretKind,
    message: Value,
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
