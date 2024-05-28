use dal::prop::PropPath;
use dal::property_editor::values::PropertyEditorValues;
use dal::qualification::QualificationSubCheckStatus;
use dal::secret::DecryptedSecret;
use dal::{
    AttributeValue, Component, DalContext, EncryptedSecret, OutputSocket, Prop, Secret,
    SecretAlgorithm, SecretVersion,
};
use dal_test::helpers::{create_component_for_schema_name, encrypt_message, ChangeSetTestHelpers};
use dal_test::{helpers::generate_fake_name, test, WorkspaceSignup};
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;

mod bench;
mod with_actions;

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
    let encrypted_secret = EncryptedSecret::get_by_key(ctx, secret.encrypted_secret_key())
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
    assert_eq!(
        secret.encrypted_secret_key(),
        found_secret.encrypted_secret_key()
    );

    // Ensure that the decrypted contents match our message.
    let decrypted = EncryptedSecret::get_by_key(ctx, found_secret.encrypted_secret_key())
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
    assert_eq!(
        secret.encrypted_secret_key(),
        found_secret.encrypted_secret_key()
    );

    // Ensure that the decrypted message matches the original message.
    let decrypted = EncryptedSecret::get_by_key(ctx, found_secret.encrypted_secret_key())
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
    let original_key = secret.encrypted_secret_key();
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
    assert_ne!(original_key, updated_secret.encrypted_secret_key());
    assert_eq!(
        found_updated_secret.encrypted_secret_key(),
        updated_secret.encrypted_secret_key()
    );

    // Check that the decrypted contents match.
    let updated_decrypted =
        EncryptedSecret::get_by_key(ctx, found_updated_secret.encrypted_secret_key())
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
        double_updated_secret.encrypted_secret_key(),
        found_double_updated_secret.encrypted_secret_key()
    );
    assert_eq!(
        double_updated_secret.encrypted_secret_key(),
        found_updated_secret.encrypted_secret_key()
    );

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
        EncryptedSecret::get_by_key(ctx, found_double_updated_secret.encrypted_secret_key())
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

#[test]
async fn update_encrypted_contents_with_dependent_values(
    ctx: &mut DalContext,
    nw: &WorkspaceSignup,
) {
    // Create a component and commit.
    let component =
        create_component_for_schema_name(ctx, "dummy-secret", "secret-definition").await;
    let schema_variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("could not get schema variant id for component");
    let component_id = component.id();
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Cache the name of the secret definition from the test exclusive schema and cache the prop we
    // need for attribute value update.
    let secret_definition_name = "dummy";
    let dummy_secret_prop = Prop::find_prop_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // Create a secret with a value that will fail the qualification and commit.
    let encrypted_message_that_will_fail_the_qualification = encrypt_message(
        ctx,
        nw.key_pair.pk(),
        &serde_json::json![{"value": "howard"}],
    )
    .await;
    let secret = Secret::new(
        ctx,
        generate_fake_name(),
        secret_definition_name,
        None,
        &encrypted_message_that_will_fail_the_qualification,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Use the secret in the component and commit.
    let property_values = PropertyEditorValues::assemble(ctx, component_id)
        .await
        .expect("unable to list prop values");
    let dummy_secret_attribute_value_id = property_values
        .find_by_prop_id(dummy_secret_prop.id)
        .expect("unable to find attribute value");
    Secret::attach_for_attribute_value(ctx, dummy_secret_attribute_value_id, Some(secret.id()))
        .await
        .expect("could not attach secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check that the qualification fails.
    let qualifications = Component::list_qualifications(ctx, component_id)
        .await
        .expect("could not list qualifications");
    let qualification = qualifications
        .iter()
        .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
        .expect("qualification not found")
        .to_owned();
    assert_eq!(
        QualificationSubCheckStatus::Failure, // expected
        qualification.result.expect("no result found").status  // actual
    );

    // Update the encrypted contents.
    let encrypted_message_that_will_pass_the_qualification =
        encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}]).await;
    let updated_secret = secret
        .update_encrypted_contents(
            ctx,
            encrypted_message_that_will_pass_the_qualification.as_slice(),
            nw.key_pair.pk(),
            Default::default(),
            Default::default(),
        )
        .await
        .expect("could not update encrypted contents");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check that the qualification succeeds.
    let qualifications = Component::list_qualifications(ctx, component_id)
        .await
        .expect("could not list qualifications");
    let qualification = qualifications
        .iter()
        .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
        .expect("qualification not found")
        .to_owned();
    assert_eq!(
        QualificationSubCheckStatus::Success, // expected
        qualification.result.expect("no result found").status  // actual
    );

    // Unset the secret and commit.
    let property_values = PropertyEditorValues::assemble(ctx, component_id)
        .await
        .expect("unable to list prop values");
    let dummy_secret_attribute_value_id = property_values
        .find_by_prop_id(dummy_secret_prop.id)
        .expect("unable to find attribute value");
    Secret::attach_for_attribute_value(ctx, dummy_secret_attribute_value_id, None)
        .await
        .expect("could not attach for attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the qualification fails.
    let qualifications = Component::list_qualifications(ctx, component_id)
        .await
        .expect("could not list qualifications");
    let qualification = qualifications
        .iter()
        .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
        .expect("qualification not found")
        .to_owned();
    assert_eq!(
        QualificationSubCheckStatus::Failure, // expected
        qualification.result.expect("no result found").status  // actual
    );

    // Use the secret again.
    let property_values = PropertyEditorValues::assemble(ctx, component_id)
        .await
        .expect("unable to list prop values");
    let dummy_secret_attribute_value_id = property_values
        .find_by_prop_id(dummy_secret_prop.id)
        .expect("unable to find attribute value");
    Secret::attach_for_attribute_value(
        ctx,
        dummy_secret_attribute_value_id,
        Some(updated_secret.id()),
    )
    .await
    .expect("could not attach secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Ensure that the qualification succeeds.
    let qualifications = Component::list_qualifications(ctx, component_id)
        .await
        .expect("could not list qualifications");
    let qualification = qualifications
        .iter()
        .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
        .expect("qualification not found")
        .to_owned();
    assert_eq!(
        QualificationSubCheckStatus::Success, // expected
        qualification.result.expect("no result found").status  // actual
    );
}

#[test]
async fn secret_definition_works_with_dummy_qualification(
    ctx: &mut DalContext,
    nw: &WorkspaceSignup,
) {
    // Create a component and commit.
    let secret_definition_component =
        create_component_for_schema_name(ctx, "dummy-secret", "secret-definition").await;
    let secret_definition_component_id = secret_definition_component.id();
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Cache the name of the secret definition from the test exclusive schema. Afterward, cache
    // the variables we need throughout the test.
    let secret_definition_name = "dummy";
    let secret_definition_schema_variant_id =
        Component::schema_variant_id(ctx, secret_definition_component.id())
            .await
            .expect("could not get schema variant id for component");
    let output_socket = OutputSocket::find_with_name(
        ctx,
        secret_definition_name,
        secret_definition_schema_variant_id,
    )
    .await
    .expect("could not perform find with name")
    .expect("output socket not found");
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        secret_definition_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // First scenario: create and use a secret that will fail the qualification.
    {
        // Create a secret with a value that will fail the qualification and commit.
        let encrypted_message_that_will_fail_the_qualification = encrypt_message(
            ctx,
            nw.key_pair.pk(),
            &serde_json::json![{"value": "howard"}],
        )
        .await;
        let secret_that_will_fail_the_qualification = Secret::new(
            ctx,
            "secret that will fail the qualification",
            secret_definition_name.to_string(),
            None,
            &encrypted_message_that_will_fail_the_qualification,
            nw.key_pair.pk(),
            Default::default(),
            Default::default(),
        )
        .await
        .expect("cannot create secret");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Update the reference to secret prop with the secret it that will fail the qualification
        // and commit.
        let property_values = PropertyEditorValues::assemble(ctx, secret_definition_component_id)
            .await
            .expect("unable to list prop values");
        let reference_to_secret_attribute_value_id = property_values
            .find_by_prop_id(reference_to_secret_prop.id)
            .expect("unable to find attribute value");
        Secret::attach_for_attribute_value(
            ctx,
            reference_to_secret_attribute_value_id,
            Some(secret_that_will_fail_the_qualification.id()),
        )
        .await
        .expect("could not attach secret");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Check that the output socket value looks correct.
        let mut output_socket_attribute_value_ids =
            OutputSocket::attribute_values_for_output_socket_id(ctx, output_socket.id())
                .await
                .expect("could not perform attribute values for output socket id");
        let output_socket_attribute_value_id = output_socket_attribute_value_ids
            .pop()
            .expect("no output attribute value found");
        assert!(output_socket_attribute_value_ids.is_empty());
        let output_socket_attribute_value =
            AttributeValue::get_by_id(ctx, output_socket_attribute_value_id)
                .await
                .expect("could not get attribute value by id")
                .value(ctx)
                .await
                .expect("could not get value")
                .expect("no value found");
        assert_eq!(
            Secret::payload_for_prototype_execution(
                ctx,
                secret_that_will_fail_the_qualification.id()
            )
            .await
            .expect("could not get payload"), // expected
            output_socket_attribute_value // actual
        );

        // Check that the qualification fails.
        let qualifications = Component::list_qualifications(ctx, secret_definition_component_id)
            .await
            .expect("could not list qualifications");
        let qualification = qualifications
            .into_iter()
            .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
            .expect("could not find qualification");
        assert_eq!(
            QualificationSubCheckStatus::Failure, // expected
            qualification.result.expect("no result found").status  // actual
        );
    }

    // Second scenario: create and use a secret that will pass the qualification.
    {
        // Create a secret with a value that will pass the qualification and commit.
        let encrypted_message_that_will_pass_the_qualification =
            encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}]).await;
        let secret_that_will_pass_the_qualification = Secret::new(
            ctx,
            "secret that will pass the qualification",
            secret_definition_name.to_string(),
            None,
            &encrypted_message_that_will_pass_the_qualification,
            nw.key_pair.pk(),
            Default::default(),
            Default::default(),
        )
        .await
        .expect("cannot create secret");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Update the reference to secret prop with the secret it that will pass the qualification
        // and commit.
        let property_values = PropertyEditorValues::assemble(ctx, secret_definition_component_id)
            .await
            .expect("unable to list prop values");
        let reference_to_secret_attribute_value_id = property_values
            .find_by_prop_id(reference_to_secret_prop.id)
            .expect("could not find attribute value");
        Secret::attach_for_attribute_value(
            ctx,
            reference_to_secret_attribute_value_id,
            Some(secret_that_will_pass_the_qualification.id()),
        )
        .await
        .expect("could not attach secret");
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
            .await
            .expect("could not commit and update snapshot to visibility");

        // Check that the output socket value looks correct.
        let mut output_socket_attribute_value_ids =
            OutputSocket::attribute_values_for_output_socket_id(ctx, output_socket.id())
                .await
                .expect("could not perform attribute values for output socket id");
        let output_socket_attribute_value_id = output_socket_attribute_value_ids
            .pop()
            .expect("no output attribute value found");
        assert!(output_socket_attribute_value_ids.is_empty());
        let output_socket_attribute_value =
            AttributeValue::get_by_id(ctx, output_socket_attribute_value_id)
                .await
                .expect("could not get attribute value by id")
                .value(ctx)
                .await
                .expect("could not get value")
                .expect("no value found");
        assert_eq!(
            Secret::payload_for_prototype_execution(
                ctx,
                secret_that_will_pass_the_qualification.id()
            )
            .await
            .expect("could not get payload"), // expected
            output_socket_attribute_value // actual
        );

        // Check that the qualification passes.
        let qualifications = Component::list_qualifications(ctx, secret_definition_component_id)
            .await
            .expect("could not list qualifications");
        let qualification = qualifications
            .into_iter()
            .find(|q| q.qualification_name == "test:qualificationDummySecretStringIsTodd")
            .expect("could not find qualification");
        assert_eq!(
            QualificationSubCheckStatus::Success, // expected
            qualification.result.expect("no result found").status  // actual
        );
    }
}

fn prepare_decrypted_secret_for_assertions(decrypted_secret: &DecryptedSecret) -> Value {
    // We don't provide a direct getter for the raw decrypted message (higher effort should mean
    // less chance of developer error when handling `DecryptedSecret` types), so we'll serialize to
    // a `Value` to compare messages
    let decrypted_value =
        serde_json::to_value(decrypted_secret).expect("failed to serialize decrypted contents");
    decrypted_value["message"].to_owned()
}
