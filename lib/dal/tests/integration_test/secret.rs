use dal::secret::DecryptedSecret;
use dal::{DalContext, EncryptedSecret, Secret, SecretAlgorithm, SecretVersion};
use dal_test::{helpers::generate_fake_name, test, WorkspaceSignup};
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;

#[test]
async fn new(ctx: &DalContext, nw: &WorkspaceSignup) {
    let name = generate_fake_name();

    // Ensure that secret creation works.
    let secret = Secret::new(
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
    assert_eq!(name, secret.name());
    assert_eq!("Mock", secret.definition());
    assert_eq!(Some("Description"), secret.description().as_deref());

    // Ensure that the underlying encrypted secret was created and that we can fetch its key pair.
    let encrypted_secret = EncryptedSecret::get_by_key(ctx, secret.key())
        .await
        .expect("failed to perform get by key for encrypted secret")
        .expect("no encrypted secret found");
    let key_pair = encrypted_secret
        .key_pair(ctx)
        .await
        .expect("failed to fetch key pair");
    assert_eq!(nw.key_pair.pk(), key_pair.pk());

    // Fetch the secret by id too.
    let found_secret = Secret::get_by_id_or_error(ctx, secret.id())
        .await
        .expect("could not perform get by id or secret not found");
    assert_eq!(secret, found_secret);
}

#[test]
async fn encrypt_decrypt_round_trip(ctx: &DalContext, nw: &WorkspaceSignup) {
    let pkey = nw.key_pair.public_key();
    let name = generate_fake_name();

    // Create an encrypted message.
    let message = serde_json::json!({"song": "Bar Round Here"});
    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize message"),
        pkey,
    );

    // Create a secret with the encrypted message.
    let secret = Secret::new(
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

    // Ensure that the fetched secret looks as we expected.
    let found_secret = Secret::get_by_id_or_error(ctx, secret.id())
        .await
        .expect("could not perform get by id or secret not found");
    assert_eq!(secret.name(), found_secret.name());
    assert_eq!(secret.description(), found_secret.description());
    assert_eq!(secret.definition(), found_secret.definition());
    assert_eq!(secret.key(), found_secret.key());

    // Ensure that the decrypted contents match our messag.e
    let decrypted = EncryptedSecret::get_by_key(ctx, found_secret.key())
        .await
        .expect("failed to perform get by key for encrypted secret")
        .expect("no encrypted secret found")
        .decrypt(ctx)
        .await
        .expect("failed to decrypt encrypted secret");
    let actual_message = prepare_decrypted_secret_for_assertions(&decrypted);
    assert_eq!(message, actual_message);
}

#[test]
async fn update_metadata_and_encrypted_contents(ctx: &DalContext, nw: &WorkspaceSignup) {
    let pkey = nw.key_pair.public_key();
    let key_pair_pk = nw.key_pair.pk();
    let version = SecretVersion::default();
    let algorithm = SecretAlgorithm::default();
    let name = generate_fake_name();

    // Create a message to encrypt and use for the secret.
    let message = serde_json::json!({"song": "Smile", "artist": "midwxst"});
    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&message).expect("failed to serialize message"),
        pkey,
    );

    // Create the secret.
    let secret = Secret::new(
        ctx,
        &name,
        "my flight might be delayed, but I'm writing this test, so it's all good",
        None,
        &crypted,
        key_pair_pk,
        version,
        algorithm,
    )
    .await
    .expect("failed to create encrypted secret");

    // Ensure that the fetched secret looks as we expect.
    let found_secret = Secret::get_by_id_or_error(ctx, secret.id())
        .await
        .expect("could not perform get by id or secret not found");
    assert_eq!(secret.name(), found_secret.name());
    assert_eq!(secret.description(), found_secret.description());
    assert_eq!(secret.definition(), found_secret.definition());
    assert_eq!(secret.key(), found_secret.key());

    // Ensure that the decrypted message matches the original message.
    let decrypted = EncryptedSecret::get_by_key(ctx, found_secret.key())
        .await
        .expect("failed to perform get by key for encrypted secret")
        .expect("no encrypted secret found")
        .decrypt(ctx)
        .await
        .expect("failed to decrypt encrypted secret");
    let actual_message = prepare_decrypted_secret_for_assertions(&decrypted);
    assert_eq!(message, actual_message);

    // Update the encrypted contents and the secret.
    let updated_message =
        serde_json::json!({"song": "Smile", "artist": "midwxst", "featuredArtists": ["glaive"]});
    let updated_crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(&updated_message).expect("failed to serialize message"),
        pkey,
    );
    let original_key = secret.key();
    let updated_secret = secret
        .update_encrypted_contents(
            ctx,
            updated_crypted.as_slice(),
            key_pair_pk,
            version,
            algorithm,
        )
        .await
        .expect("could not update encrypted contents");
    let found_updated_secret = Secret::get_by_id_or_error(ctx, updated_secret.id())
        .await
        .expect("could not perform get by id or secret not found");

    // Check that the key has changed.
    assert_ne!(original_key, updated_secret.key());
    assert_eq!(found_updated_secret.key(), updated_secret.key());

    // Check that the decrypted contents match.
    let updated_decrypted = EncryptedSecret::get_by_key(ctx, found_updated_secret.key())
        .await
        .expect("failed to perform get by key for encrypted secret")
        .expect("no encrypted secret found")
        .decrypt(ctx)
        .await
        .expect("failed to decrypt encrypted secret");
    let actual_updated_message = prepare_decrypted_secret_for_assertions(&updated_decrypted);
    assert_eq!(updated_message, actual_updated_message);

    // Ensure the metadata has not changed
    assert_eq!(found_secret.name(), updated_secret.name());
    assert_eq!(found_secret.definition(), updated_secret.definition());
    assert_eq!(found_updated_secret.name(), updated_secret.name());
    assert_eq!(
        found_updated_secret.definition(),
        updated_secret.definition()
    );

    // Now, update the metadata.
    let double_updated_secret = updated_secret.update_metadata(ctx, name, Some("alright, so now we are in the air and I am writing this test offline, which is awesome!".to_string())).await.expect("could not update metadata");
    let found_double_updated_secret = Secret::get_by_id_or_error(ctx, double_updated_secret.id())
        .await
        .expect("could not perform get by id or secret not found");

    // Ensure the description has changed.
    assert_eq!(
        double_updated_secret.description(),
        found_double_updated_secret.description()
    );
    assert_ne!(
        double_updated_secret.description(),
        found_updated_secret.description()
    );

    // Ensure the key has not changed.
    assert_eq!(
        double_updated_secret.key(),
        found_double_updated_secret.key()
    );
    assert_eq!(double_updated_secret.key(), found_updated_secret.key());

    // Ensure the definition has not changed.
    assert_eq!(
        double_updated_secret.definition(),
        found_double_updated_secret.definition()
    );
    assert_eq!(
        double_updated_secret.definition(),
        found_updated_secret.definition()
    );

    // Check that the decrypted contents have not changed.
    let updated_decrypted_should_not_have_changed =
        EncryptedSecret::get_by_key(ctx, found_double_updated_secret.key())
            .await
            .expect("failed to perform get by key for encrypted secret")
            .expect("no encrypted secret found")
            .decrypt(ctx)
            .await
            .expect("failed to decrypt encrypted secret");
    let actual_updated_message_should_not_have_changed =
        prepare_decrypted_secret_for_assertions(&updated_decrypted_should_not_have_changed);
    assert_eq!(
        updated_message,
        actual_updated_message_should_not_have_changed
    );
}

fn prepare_decrypted_secret_for_assertions(decrypted_secret: &DecryptedSecret) -> Value {
    // We don't provide a direct getter for the raw decrypted message (higher effort should mean
    // less chance of developer error when handling `DecryptedSecret` types), so we'll serialize to
    // a `Value` to compare messages
    let decrypted_value =
        serde_json::to_value(decrypted_secret).expect("failed to serialize decrypted contents");
    decrypted_value["message"].to_owned()
}
