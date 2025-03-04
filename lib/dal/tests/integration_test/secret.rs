use dal::diagram::geometry::RawGeometry;
use dal::prop::PropPath;
use dal::property_editor::values::PropertyEditorValues;
use dal::qualification::QualificationSubCheckStatus;
use dal::secret::DecryptedSecret;
use dal::{Component, DalContext, EncryptedSecret, Prop, Secret, SecretAlgorithm, SecretVersion};
use dal_test::expected::{self, ExpectComponent, ExpectView};
use dal_test::helpers::{
    create_component_for_default_schema_name_in_default_view, encrypt_message, ChangeSetTestHelpers,
};
use dal_test::{helpers::generate_fake_name, test, WorkspaceSignup};
use pretty_assertions_sorted::assert_eq;
use serde_json::Value;

mod with_actions;
mod with_schema_variant_authoring;

#[test]
async fn new(ctx: &DalContext, nw: &WorkspaceSignup) {
    let name = generate_fake_name().expect("could not generate fake name");

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
    let name = generate_fake_name().expect("could not generate fake name");

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
    let name = generate_fake_name().expect("could not generate fake name");

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
async fn copy_paste_component_with_secrets_being_used(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    // Create a component and commit.
    let secret_component = ExpectComponent::create(ctx, "dummy-secret").await;
    let user_component = ExpectComponent::create(ctx, "fallout").await;
    secret_component
        .connect(ctx, "dummy", user_component, "dummy")
        .await;
    let secret_prop = secret_component
        .prop(ctx, ["root", "secrets", "dummy"])
        .await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Create a secret with a value that will pass the qualification and commit.
    let secret_message =
        encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message");
    let secret = Secret::new(
        ctx,
        "secret that will pass the qualification",
        "dummy",
        None,
        &secret_message,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Set the secret on the component and commit
    let dummy_secret_attribute_value_id = secret_prop.attribute_value(ctx).await.id();
    Secret::attach_for_attribute_value(ctx, dummy_secret_attribute_value_id, Some(secret.id()))
        .await
        .expect("could not attach secret");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    let default_view_id = ExpectView::get_id_for_default(ctx).await;

    // Copy and paste the secret component
    {
        let component = secret_component.component(ctx).await;

        let geometry = component
            .geometry(ctx, default_view_id)
            .await
            .expect("couldn't get geometry");

        component
            .copy_without_connections(
                ctx,
                default_view_id,
                RawGeometry {
                    x: geometry.x(),
                    y: geometry.y(),
                    width: None,
                    height: None,
                },
            )
            .await
            .expect("paste");
    }
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Copy and paste the user component
    let user_component_geometry = user_component
        .component(ctx)
        .await
        .geometry(ctx, default_view_id)
        .await
        .expect("couldn't get geometry");

    user_component
        .component(ctx)
        .await
        .copy_without_connections(
            ctx,
            default_view_id,
            RawGeometry {
                x: user_component_geometry.x(),
                y: user_component_geometry.y(),
                width: None,
                height: None,
            },
        )
        .await
        .expect("paste");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;
}

#[test]
async fn consumed_secrets_work_when_dvu_not_up_to_date(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    // Create a secret and consumer component.
    let secret_component = ExpectComponent::create(ctx, "dummy-secret").await;
    let user_component = ExpectComponent::create(ctx, "fallout").await;
    secret_component
        .connect(ctx, "dummy", user_component, "dummy")
        .await;
    let secret_prop = secret_component
        .prop(ctx, ["root", "secrets", "dummy"])
        .await;
    let user_secret_prop = user_component.prop(ctx, ["root", "secrets", "dummy"]).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Create a secret with a value and commit.
    let secret_message =
        encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message");
    let secret = Secret::new(
        ctx,
        "secret",
        "dummy",
        None,
        &secret_message,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    let secret_message = secret.encrypted_secret_key();
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Set the secret on the component and commit
    let dummy_secret_attribute_value_id = secret_prop.attribute_value(ctx).await.id();
    Secret::attach_for_attribute_value(ctx, dummy_secret_attribute_value_id, Some(secret.id()))
        .await
        .expect("could not attach secret");
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    let found_secret = Secret::get_by_id_or_error(ctx, secret.id())
        .await
        .expect("could not perform get by id or secret not found");
    assert_eq!(secret_message, found_secret.encrypted_secret_key());

    // Check the property editor values.
    let (value, _) = PropertyEditorValues::assemble(ctx, secret_component.id())
        .await
        .expect("able to list prop values")
        .find_with_value_by_prop_id(secret_prop.prop().id())
        .expect("secret prop");
    assert_eq!(Value::from(secret.id().to_string()), value);

    let (value, _) = PropertyEditorValues::assemble(ctx, user_component.id())
        .await
        .expect("able to list prop values")
        .find_with_value_by_prop_id(user_secret_prop.prop().id())
        .expect("secret prop");
    assert_eq!(Value::from(secret.id().to_string()), value);

    // Update the secret.
    let updated_secret_message = encrypt_message(
        ctx,
        nw.key_pair.pk(),
        &serde_json::json![{"value": "sweeney"}],
    )
    .await
    .expect("could not encrypt message");
    let updated_secret = Secret::get_by_id_or_error(ctx, secret.id())
        .await
        .expect("no secret")
        .update_encrypted_contents(
            ctx,
            updated_secret_message.as_slice(),
            nw.key_pair.pk(),
            Default::default(),
            Default::default(),
        )
        .await
        .expect("could not update encrypted contents");
    let updated_secret_message = updated_secret.encrypted_secret_key();

    // Validate that the secret has the new value.
    let found_secret = Secret::get_by_id_or_error(ctx, secret.id())
        .await
        .expect("could not perform get by id or secret not found");
    assert_eq!(updated_secret_message, found_secret.encrypted_secret_key());

    // Check the property editor values. Since the DVU is not up to date, we expect the value
    // yielded to be null.
    let (value, _) = PropertyEditorValues::assemble(ctx, secret_component.id())
        .await
        .expect("able to list prop values")
        .find_with_value_by_prop_id(secret_prop.prop().id())
        .expect("secret prop");
    assert_eq!(Value::Null, value);

    let (value, _) = PropertyEditorValues::assemble(ctx, user_component.id())
        .await
        .expect("able to list prop values")
        .find_with_value_by_prop_id(user_secret_prop.prop().id())
        .expect("secret prop");
    assert_eq!(Value::Null, value);

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // Check the property editor values again after committing.
    let (value, _) = PropertyEditorValues::assemble(ctx, secret_component.id())
        .await
        .expect("able to list prop values")
        .find_with_value_by_prop_id(secret_prop.prop().id())
        .expect("secret prop");
    assert_eq!(Value::from(secret.id().to_string()), value);

    // Check the property editor values.
    let (value, _) = PropertyEditorValues::assemble(ctx, user_component.id())
        .await
        .expect("able to list prop values")
        .find_with_value_by_prop_id(user_secret_prop.prop().id())
        .expect("secret prop");
    assert_eq!(Value::from(secret.id().to_string()), value);
}

#[test]
async fn update_encrypted_contents_with_dependent_values(
    ctx: &mut DalContext,
    nw: &WorkspaceSignup,
) {
    // Create a component and commit.
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "dummy-secret",
        "secret-definition",
    )
    .await
    .expect("could not create component");
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
    .await
    .expect("could not encrypt message");
    let secret = Secret::new(
        ctx,
        generate_fake_name().expect("could not generate fake name"),
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
        encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message");
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
    let component = ExpectComponent::create(ctx, "dummy-secret").await;
    let secret_name = "dummy";
    let output_socket = component.output_socket(ctx, secret_name).await;
    let secret_prop = component.prop(ctx, ["root", "secrets", secret_name]).await;
    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    // First scenario: create and use a secret that will fail the qualification.
    {
        // Create a secret with a value that will fail the qualification and commit.
        let encrypted_message_that_will_fail_the_qualification = encrypt_message(
            ctx,
            nw.key_pair.pk(),
            &serde_json::json![{"value": "howard"}],
        )
        .await
        .expect("could not encrypt message");
        let secret_that_will_fail_the_qualification = Secret::new(
            ctx,
            "secret that will fail the qualification",
            secret_name.to_string(),
            None,
            &encrypted_message_that_will_fail_the_qualification,
            nw.key_pair.pk(),
            Default::default(),
            Default::default(),
        )
        .await
        .expect("cannot create secret");
        expected::commit_and_update_snapshot_to_visibility(ctx).await;

        // Update the reference to secret prop with the secret it that will fail the qualification
        // and commit.
        Secret::attach_for_attribute_value(
            ctx,
            secret_prop.attribute_value(ctx).await.id(),
            Some(secret_that_will_fail_the_qualification.id()),
        )
        .await
        .expect("could not attach secret");
        expected::commit_and_update_snapshot_to_visibility(ctx).await;

        // Check that the output socket value looks correct.
        assert_eq!(
            Secret::payload_for_prototype_execution(
                ctx,
                secret_that_will_fail_the_qualification.id()
            )
            .await
            .expect("could not get payload"), // expected
            output_socket.get(ctx).await // actual
        );

        // Check that the qualification fails.
        let qualifications = Component::list_qualifications(ctx, component.id())
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
            encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
                .await
                .expect("could not encrypt message");
        let secret_that_will_pass_the_qualification = Secret::new(
            ctx,
            "secret that will pass the qualification",
            secret_name.to_string(),
            None,
            &encrypted_message_that_will_pass_the_qualification,
            nw.key_pair.pk(),
            Default::default(),
            Default::default(),
        )
        .await
        .expect("cannot create secret");
        expected::commit_and_update_snapshot_to_visibility(ctx).await;

        // Update the reference to secret prop with the secret it that will pass the qualification
        // and commit.
        Secret::attach_for_attribute_value(
            ctx,
            secret_prop.attribute_value(ctx).await.id(),
            Some(secret_that_will_pass_the_qualification.id()),
        )
        .await
        .expect("could not attach secret");
        expected::commit_and_update_snapshot_to_visibility(ctx).await;

        // Check that the output socket value looks correct.
        assert_eq!(
            Secret::payload_for_prototype_execution(
                ctx,
                secret_that_will_pass_the_qualification.id()
            )
            .await
            .expect("could not get payload"), // expected
            output_socket.get(ctx).await // actual
        );

        // Check that the qualification passes.
        let qualifications = Component::list_qualifications(ctx, component.id())
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
