use crate::WriteTenancy;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use sodiumoxide::crypto::box_::{self, PublicKey as BoxPublicKey, SecretKey as BoxSecretKey};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_accessor_ro,
    standard_model_belongs_to, BillingAccount, BillingAccountId, DalContext, HistoryEvent,
    HistoryEventError, StandardModel, StandardModelError, Timestamp, Visibility,
};

mod key_pair_box_public_key_serde;
mod key_pair_box_secret_key_serde;

const PUBLIC_KEY_GET_CURRENT: &str = include_str!("./queries/public_key_get_current.sql");

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
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("no current key pair found when one was expected")]
    NoCurrentKeyPair,
}

pub type KeyPairResult<T> = Result<T, KeyPairError>;

pk!(KeyPairPk);
pk!(KeyPairId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct KeyPair {
    pk: KeyPairPk,
    id: KeyPairId,
    name: String,
    #[serde(with = "key_pair_box_public_key_serde")]
    public_key: BoxPublicKey,
    #[serde(with = "key_pair_box_secret_key_serde")]
    secret_key: BoxSecretKey,
    created_lamport_clock: u64,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: KeyPair,
    pk: KeyPairPk,
    id: KeyPairId,
    table_name: "key_pairs",
    history_event_label_base: "key_pair",
    history_event_message_name: "Key Pair"
}

impl KeyPair {
    pub async fn new(ctx: &DalContext, name: impl AsRef<str>) -> KeyPairResult<Self> {
        let name = name.as_ref();
        let (public_key, secret_key) = box_::gen_keypair();

        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM key_pair_create_v1($1, $2, $3, $4, $5)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &name,
                    &encode_public_key(&public_key),
                    &encode_secret_key(&secret_key),
                ],
            )
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let _history_event = HistoryEvent::new(
            ctx,
            Self::history_event_label(vec!["create"]),
            Self::history_event_message("created"),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_current(
        ctx: &DalContext,
        billing_account_id: &BillingAccountId,
    ) -> KeyPairResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                PUBLIC_KEY_GET_CURRENT,
                &[ctx.read_tenancy(), ctx.visibility(), &billing_account_id],
            )
            .await?;
        let object = standard_model::object_from_row(row)?;
        Ok(object)
    }

    standard_model_accessor!(name, String, KeyPairResult);
    standard_model_accessor_ro!(public_key, BoxPublicKey);
    standard_model_accessor_ro!(secret_key, BoxSecretKey);
    standard_model_accessor_ro!(created_lamport_clock, u64);

    standard_model_belongs_to!(
         lookup_fn: billing_account,
         set_fn: set_billing_account,
         unset_fn: unset_billing_account,
         table: "key_pair_belongs_to_billing_account",
         model_table: "billing_accounts",
         belongs_to_id: BillingAccountId,
         returns: BillingAccount,
         result: KeyPairResult,
    );
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
///
/// The field "public_key" is base64 encoded into a string when this struct is serialized, and
/// decoded when deserialized. Thus, the DAL "PublicKey" (this struct) must be used for transport
/// between SI components rather than the nested "BoxPublicKey".
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PublicKey {
    pk: KeyPairPk,
    id: KeyPairId,
    name: String,
    /// This field is base64 encoded into a string. Consumers will have to base64 decode it.
    #[serde(with = "key_pair_box_public_key_serde")]
    public_key: BoxPublicKey,
    created_lamport_clock: u64,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl PublicKey {
    pub async fn get_current(
        ctx: &DalContext,
        billing_account_id: &BillingAccountId,
    ) -> KeyPairResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                PUBLIC_KEY_GET_CURRENT,
                &[ctx.read_tenancy(), ctx.visibility(), &billing_account_id],
            )
            .await?;
        let object = standard_model::object_from_row(row)?;
        Ok(object)
    }

    pub fn pk(&self) -> &KeyPairPk {
        &self.pk
    }

    pub fn public_key(&self) -> &BoxPublicKey {
        &self.public_key
    }
}
