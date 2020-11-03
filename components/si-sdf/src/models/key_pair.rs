use crate::{
    data::{Connection, Db},
    models::{generate_id, get_model, insert_model, BillingAccount, ModelError, SimpleStorable},
};
use serde::{Deserialize, Serialize};
use sodiumoxide::crypto::box_::{self, PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeyPairError {
    // Not using `BillingAccountError` as this leads us to a circular dependency of errors
    #[error("error in billing account: {0}")]
    BillingAccount(Box<dyn std::error::Error + Sync + Send + 'static>),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
}

pub type KeyPairResult<T> = Result<T, KeyPairError>;

/// A database-persisted libsodium box key pair.
///
/// Both the public key and secret key are accessible and therefore this type should *only* be used
/// internally when decrypting secrets for use by `veritech`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct KeyPair {
    pub id: String,
    pub name: String,
    pub public_key: BoxPublicKey,
    pub secret_key: BoxSecretKey,
    pub si_storable: SimpleStorable,
}

impl KeyPair {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: impl Into<String>,
        billing_account_id: impl AsRef<str>,
    ) -> KeyPairResult<Self> {
        let name = name.into();
        let (public_key, secret_key) = box_::gen_keypair();

        let id = generate_id("keyPair");
        let si_storable = SimpleStorable::new(&id, "keyPair", billing_account_id.as_ref());
        let model = Self {
            id,
            name,
            public_key,
            secret_key,
            si_storable,
        };
        insert_model(db, nats, &model.id, &model).await?;

        Ok(model)
    }

    pub(crate) async fn get(
        db: &Db,
        id: impl AsRef<str> + std::fmt::Debug,
        billing_account_id: impl AsRef<str> + std::fmt::Debug,
    ) -> KeyPairResult<Self> {
        get_model(db, id, billing_account_id)
            .await
            .map_err(KeyPairError::from)
    }
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
    pub public_key: BoxPublicKey,
    pub si_storable: SimpleStorable,
}

impl PublicKey {
    pub async fn get_current(db: &Db, billing_account_id: impl AsRef<str>) -> KeyPairResult<Self> {
        let billing_account = BillingAccount::get(db, billing_account_id)
            .await
            .map_err(|err| KeyPairError::BillingAccount(Box::new(err)))?;
        let object: Self =
            get_model(db, billing_account.current_key_pair_id, billing_account.id).await?;

        Ok(object)
    }
}
