use dal::prop::PropPath;
use dal::property_editor::values::PropertyEditorValues;
use dal::qualification::QualificationSubCheckStatus;
use dal::{AttributeValue, Component, DalContext, EncryptedSecret, OutputSocket, Prop};
use dal_test::test_harness::create_component_for_schema_name;
use dal_test::test_harness::encrypt_message;
use dal_test::{test, WorkspaceSignup};

#[test]
async fn secret_definition_works_with_dummy_qualification(
    ctx: &mut DalContext,
    nw: &WorkspaceSignup,
) {
    let secret_definition_component =
        create_component_for_schema_name(ctx, "dummy-secret", "secret-definition").await;

    let secret_definition_schema_variant_id =
        Component::schema_variant_id(ctx, secret_definition_component.id())
            .await
            .expect("could not get schema variant id for component");

    let secret_definition_component_id = secret_definition_component.id();

    // This is the name of the secret definition from the test exclusive schema.
    let secret_definition_name = "dummy";

    // Cache the output socket that will contain the secret id.
    let output_socket = OutputSocket::find_with_name(
        ctx,
        secret_definition_name,
        secret_definition_schema_variant_id,
    )
    .await
    .expect("could not perform find with name")
    .expect("output socket not found");

    // Cache the prop we need for attribute value update.
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        secret_definition_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // First scenario: create and use a secret that will fail the qualification.
    {
        // Create a secret with a value that will fail the qualification.
        let encrypted_message_that_will_fail_the_qualification = encrypt_message(
            ctx,
            nw.key_pair.pk(),
            &serde_json::json![{"value": "howard"}],
        )
        .await;
        let secret_that_will_fail_the_qualification = EncryptedSecret::new(
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

        // Commit and update snapshot to visibility.
        let conflicts = ctx.blocking_commit().await.expect("unable to commit");
        assert!(conflicts.is_none());
        ctx.update_snapshot_to_visibility()
            .await
            .expect("unable to update snapshot to visibility");

        // Update the reference to secret prop with the secret it that will fail the qualification.
        let property_values = PropertyEditorValues::assemble(ctx, secret_definition_component_id)
            .await
            .expect("unable to list prop values");
        let reference_to_secret_attribute_value_id = property_values
            .find_by_prop_id(reference_to_secret_prop.id)
            .expect("unable to find attribute value");

        let fail_value =
            serde_json::json!(secret_that_will_fail_the_qualification.id().to_string());
        AttributeValue::update(
            ctx,
            reference_to_secret_attribute_value_id,
            Some(fail_value.clone()),
        )
        .await
        .expect("unable to perform attribute value update");

        // Commit and update snapshot to visibility.
        let conflicts = ctx.blocking_commit().await.expect("unable to commit");
        assert!(conflicts.is_none());
        ctx.update_snapshot_to_visibility()
            .await
            .expect("unable to update snapshot to visibility");

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
        assert_eq!(fail_value, output_socket_attribute_value);

        // Check that the qualification fails.
        let qualifications =
            Component::list_qualifications(ctx, secret_definition_component_id)
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
        // Create a secret with a value that will pass the qualification.
        let encrypted_message_that_will_pass_the_qualification =
            encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}]).await;
        let secret_that_will_pass_the_qualification = EncryptedSecret::new(
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

        // Commit and update snapshot to visibility.
        let conflicts = ctx.blocking_commit().await.expect("unable to commit");
        assert!(conflicts.is_none());
        ctx.update_snapshot_to_visibility()
            .await
            .expect("unable to update snapshot to visibility");

        // Update the reference to secret prop with the secret it that will pass the qualification.
        let property_values = PropertyEditorValues::assemble(ctx, secret_definition_component_id)
            .await
            .expect("unable to list prop values");
        let reference_to_secret_attribute_value_id = property_values
            .find_by_prop_id(reference_to_secret_prop.id)
            .expect("could not find attribute value");

        let success_value =
            serde_json::json!(secret_that_will_pass_the_qualification.id().to_string());
        AttributeValue::update(
            ctx,
            reference_to_secret_attribute_value_id,
            Some(success_value.clone()),
        )
        .await
        .expect("unable to perform attribute value update");

        // Commit and update snapshot to visibility.
        let conflicts = ctx.blocking_commit().await.expect("unable to commit");
        assert!(conflicts.is_none());
        ctx.update_snapshot_to_visibility()
            .await
            .expect("unable to update snapshot to visibility");

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
        assert_eq!(success_value, output_socket_attribute_value);

        // Check that the qualification passes.
        let qualifications =
            Component::list_qualifications(ctx, secret_definition_component_id)
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
