use dal::{
    BillingAccountSignup, DalContext, EncryptedSecret, Secret, SecretAlgorithm, SecretKind,
    SecretObjectType, SecretVersion, StandardModel,
};
use dal_test::{
    test,
    test_harness::{create_secret, generate_fake_name},
};

#[test]
async fn new_encrypted_secret(ctx: &DalContext, nba: &BillingAccountSignup) {
    let name = generate_fake_name();

    let secret = EncryptedSecret::new(
        ctx,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        "im-crypted-bytes-maybe".as_bytes(),
        *nba.key_pair.id(),
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
        *nba.billing_account.pk(),
    )
    .await
    .expect("failed to create secret");

    assert_eq!(secret.name(), name);
    assert_eq!(secret.object_type(), &SecretObjectType::Credential);
    assert_eq!(secret.kind(), &SecretKind::DockerHub);

    let key_pair = secret
        .key_pair(ctx)
        .await
        .expect("failed to fetch key pair")
        .expect("failed to find key pair");
    assert_eq!(key_pair.pk(), nba.key_pair.pk());
}

#[test]
async fn secret_get_by_id(ctx: &DalContext, nba: &BillingAccountSignup) {
    let og_secret = create_secret(ctx, *nba.key_pair.id(), *nba.billing_account.pk()).await;

    let secret = Secret::get_by_id(ctx, og_secret.id())
        .await
        .expect("failed to get secret")
        .expect("failed to find secret in current tenancy and visibility");
    assert_eq!(secret, og_secret);
}

#[test]
async fn encrypted_secret_get_by_id(ctx: &DalContext, nba: &BillingAccountSignup) {
    let secret = create_secret(ctx, *nba.key_pair.id(), *nba.billing_account.pk()).await;

    let encrypted_secret = EncryptedSecret::get_by_id(ctx, secret.id())
        .await
        .expect("failed to get encrypted secret")
        .expect("failed to find encrypted secret in current tenancy and visibility");
    assert_eq!(secret.id(), encrypted_secret.id());
    assert_eq!(secret.pk(), encrypted_secret.pk());
    assert_eq!(secret.name(), encrypted_secret.name());
    assert_eq!(secret.object_type(), encrypted_secret.object_type());
    assert_eq!(secret.kind(), encrypted_secret.kind());
}

#[test]
async fn secret_update_name(ctx: &DalContext, nba: &BillingAccountSignup) {
    let mut secret = create_secret(ctx, *nba.key_pair.id(), *nba.billing_account.pk()).await;

    let original_name = secret.name().to_string();
    secret
        .set_name(ctx, "even-more-secret")
        .await
        .expect("failed to set name");

    assert_ne!(secret.name(), original_name);
    assert_eq!(secret.name(), "even-more-secret");
}

#[test]
async fn encrypt_decrypt_round_trip(ctx: &DalContext, nba: &BillingAccountSignup) {
    let pkey = nba.key_pair.public_key();
    let name = generate_fake_name();

    let message = serde_json::json!({"song": "Bar Round Here"});
    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serilaze message"),
        pkey,
    );

    let secret = EncryptedSecret::new(
        ctx,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &crypted,
        *nba.key_pair.id(),
        Default::default(),
        Default::default(),
        *nba.billing_account.pk(),
    )
    .await
    .expect("failed to create encrypted secret");

    let decrypted = EncryptedSecret::get_by_id(ctx, secret.id())
        .await
        .expect("failed to fetch encrypted secret")
        .expect("failed to find encrypted secret for tenancy and/or visibility")
        .decrypt(ctx)
        .await
        .expect("failed to decrypt encrypted secret");
    assert_eq!(decrypted.name(), secret.name());
    assert_eq!(decrypted.object_type(), *secret.object_type());
    assert_eq!(decrypted.kind(), *secret.kind());

    // We don't provide a direct getter for the raw decrypted message (higher effort should mean
    // less chance of developer error when handling `DecryptedSecret` types), so we'll serialize to
    // a `Value` to compare messages
    let decrypted_value =
        serde_json::to_value(&decrypted).expect("failed to serial decrypted into Value");
    assert_eq!(decrypted_value["message"], message);
}
