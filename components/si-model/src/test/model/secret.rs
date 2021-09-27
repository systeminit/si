use crate::generate_fake_name;
use crate::model::billing_account::NewBillingAccount;

use names::{Generator, Name};
use si_data::{NatsTxn, PgTxn};
use si_model::{PublicKey, Secret, SecretAlgorithm, SecretKind, SecretObjectType, SecretVersion};

pub async fn create_secret(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    billing_account_id: impl AsRef<str>,
    workspace_id: impl Into<String>,
) -> Secret {
    Secret::new(
        txn,
        nats,
        generate_fake_name(),
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        generate_fake_name(),
        PublicKey::get_current(&txn, billing_account_id.as_ref())
            .await
            .expect("could not get current public key")
            .id
            .clone(),
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        workspace_id.into(),
    )
    .await
    .expect("cannot create secret")
}

pub async fn encrypt_message(
    txn: &PgTxn<'_>,
    nba: &NewBillingAccount,
    message: &serde_json::Value,
) -> Vec<u8> {
    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize"),
        &public_key.public_key,
    );
    crypted
}

pub async fn create_secret_with_message(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nba: &NewBillingAccount,
    message: serde_json::Value,
) -> Secret {
    let public_key = PublicKey::get_current(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get current public key");

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize"),
        &public_key.public_key,
    );

    let secret = Secret::new(
        txn,
        nats,
        Generator::with_naming(Name::Numbered).next().unwrap(),
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        crypted,
        public_key.id,
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        nba.workspace.id.clone(),
    )
    .await
    .expect("cannot create secret");
    secret
}
