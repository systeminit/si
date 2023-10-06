use dal::{
    DalContext, EncryptedSecret, Secret, SecretAlgorithm, SecretVersion, StandardModel,
    WorkspaceSignup,
};
use dal_test::{
    test,
    test_harness::{create_secret, generate_fake_name},
};

#[test]
async fn new_encrypted_secret(ctx: &DalContext, nw: &WorkspaceSignup) {
    let name = generate_fake_name();

    let secret = EncryptedSecret::new(
        ctx,
        &name,
        "Mock".to_owned(),
        Some("Description".to_owned()),
        "im-crypted-bytes-maybe".as_bytes(),
        nw.key_pair.pk(),
        SecretVersion::V1,
        SecretAlgorithm::Sealedbox,
    )
    .await
    .expect("failed to create secret");

    assert_eq!(secret.name(), name);
    assert_eq!(secret.definition(), "Mock");
    assert_eq!(secret.description().as_deref(), Some("Description"));

    let key_pair = secret
        .key_pair(ctx)
        .await
        .expect("failed to fetch key pair");
    assert_eq!(key_pair.pk(), nw.key_pair.pk());
}

#[test]
async fn secret_get_by_id(ctx: &DalContext, nw: &WorkspaceSignup) {
    let og_secret = create_secret(ctx, nw.key_pair.pk()).await;

    let secret = Secret::get_by_id(ctx, og_secret.id())
        .await
        .expect("failed to get secret")
        .expect("failed to find secret in current tenancy and visibility");
    assert_eq!(secret, og_secret);
}

#[test]
async fn encrypted_secret_get_by_id(ctx: &DalContext, nw: &WorkspaceSignup) {
    let secret = create_secret(ctx, nw.key_pair.pk()).await;

    let encrypted_secret = EncryptedSecret::get_by_id(ctx, secret.id())
        .await
        .expect("failed to get encrypted secret")
        .expect("failed to find encrypted secret in current tenancy and visibility");
    assert_eq!(secret.id(), encrypted_secret.id());
    assert_eq!(secret.pk(), encrypted_secret.pk());
    assert_eq!(secret.name(), encrypted_secret.name());
    assert_eq!(secret.description(), encrypted_secret.description());
    assert_eq!(secret.definition(), encrypted_secret.definition());
}

#[test]
async fn secret_update_name(ctx: &DalContext, nw: &WorkspaceSignup) {
    let mut secret = create_secret(ctx, nw.key_pair.pk()).await;

    let original_name = secret.name().to_string();
    secret
        .set_name(ctx, "even-more-secret")
        .await
        .expect("failed to set name");

    assert_ne!(secret.name(), original_name);
    assert_eq!(secret.name(), "even-more-secret");
}

#[test]
async fn encrypt_decrypt_round_trip(ctx: &DalContext, nw: &WorkspaceSignup) {
    let pkey = nw.key_pair.public_key();
    let name = generate_fake_name();

    let message = serde_json::json!({"song": "Bar Round Here"});
    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serilaze message"),
        pkey,
    );

    let secret = EncryptedSecret::new(
        ctx,
        &name,
        "imasecret".to_owned(),
        None,
        &crypted,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
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
    assert_eq!(decrypted.definition(), secret.definition());

    // We don't provide a direct getter for the raw decrypted message (higher effort should mean
    // less chance of developer error when handling `DecryptedSecret` types), so we'll serialize to
    // a `Value` to compare messages
    let decrypted_value =
        serde_json::to_value(&decrypted).expect("failed to serial decrypted into Value");
    assert_eq!(decrypted_value["message"], message);
}
