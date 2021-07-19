use crate::SimpleStorable;
use base64;
use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, NatsTxnError, PgTxn};
use sodiumoxide::crypto::box_::{self, PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use thiserror::Error;

const KEY_PAIR_GET_CURRENT: &str = include_str!("./queries/key_pair_get_current.sql");

#[derive(Debug, Error)]
pub enum KeyPairError {
    // Not using `BillingAccountError` as this leads us to a circular dependency of errors
    #[error("error in billing account: {0}")]
    BillingAccount(Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub type KeyPairResult<T> = Result<T, KeyPairError>;

mod key_pair_box_public_key_serde {
    use super::encode_public_key;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use sodiumoxide::crypto::box_::PublicKey as BoxPublicKey;

    pub fn serialize<S>(box_public_key: &BoxPublicKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = encode_public_key(box_public_key);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BoxPublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let box_buffer =
            base64::decode_config(s, base64::STANDARD_NO_PAD).map_err(serde::de::Error::custom)?;
        let pk = BoxPublicKey::from_slice(&box_buffer).ok_or(serde::de::Error::custom(format!(
            "cannot deserialize public key"
        )));
        pk
    }
}

mod key_pair_box_secret_key_serde {
    use super::encode_secret_key;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use sodiumoxide::crypto::box_::SecretKey as BoxSecretKey;

    pub fn serialize<S>(box_secret_key: &BoxSecretKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = encode_secret_key(box_secret_key);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BoxSecretKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let box_buffer =
            base64::decode_config(s, base64::STANDARD_NO_PAD).map_err(serde::de::Error::custom)?;
        let pk = BoxSecretKey::from_slice(&box_buffer).ok_or(serde::de::Error::custom(format!(
            "cannot deserialize secret key"
        )));
        pk
    }
}

/// A database-persisted libsodium box key pair.
///
/// Both the public key and secret key are accessible and therefore this type should *only* be used
/// internally when decrypting secrets for use by `veritech`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPair {
    pub id: String,
    pub name: String,
    #[serde(with = "key_pair_box_public_key_serde")]
    pub public_key: BoxPublicKey,
    #[serde(with = "key_pair_box_secret_key_serde")]
    pub secret_key: BoxSecretKey,
    pub si_storable: SimpleStorable,
}

impl KeyPair {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        billing_account_id: impl AsRef<str>,
    ) -> KeyPairResult<Self> {
        let name = name.into();
        let billing_account_id = billing_account_id.as_ref();
        let (public_key, secret_key) = box_::gen_keypair();

        let row = txn
            .query_one(
                "SELECT object FROM key_pair_create_v1($1, $2, $3, $4)",
                &[
                    &name,
                    &billing_account_id,
                    &encode_public_key(&public_key),
                    &encode_secret_key(&secret_key),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: KeyPair = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        id: impl AsRef<str> + std::fmt::Debug,
        billing_account_id: impl AsRef<str> + std::fmt::Debug,
    ) -> KeyPairResult<Self> {
        let id = id.as_ref();
        let billing_account_id = billing_account_id.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM key_pair_get_v1($1, $2)",
                &[&id, &billing_account_id],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }
}

fn encode_public_key(key: &BoxPublicKey) -> String {
    let s = base64::encode_config(key.as_ref(), base64::STANDARD_NO_PAD);
    s
}

fn encode_secret_key(key: &BoxSecretKey) -> String {
    let s = base64::encode_config(key.as_ref(), base64::STANDARD_NO_PAD);
    s
}

/// A database-persisted libsodium box public key.
///
/// This type only contains the public half of the underlying key pair and is therefore safe to
/// expose via external API.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub id: String,
    pub name: String,
    #[serde(with = "key_pair_box_public_key_serde")]
    pub public_key: BoxPublicKey,
    pub si_storable: SimpleStorable,
}

impl PublicKey {
    pub async fn get_current(
        txn: &PgTxn<'_>,
        billing_account_id: impl AsRef<str>,
    ) -> KeyPairResult<Self> {
        let billing_account_id = billing_account_id.as_ref();

        let row = txn
            .query_one(KEY_PAIR_GET_CURRENT, &[&billing_account_id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let public_key: PublicKey = serde_json::from_value(json)?;
        Ok(public_key)
    }
}

impl From<KeyPair> for PublicKey {
    fn from(value: KeyPair) -> Self {
        Self {
            id: value.id,
            name: value.name,
            public_key: value.public_key,
            si_storable: value.si_storable,
        }
    }
}
