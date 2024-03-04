use dal::prop::PropPath;
use dal::property_editor::values::PropertyEditorValues;
use dal::{
    AttributeValue, Component, DalContext, EncryptedSecret, ExternalProvider, Prop, Schema,
    SchemaVariant,
};
use dal_test::test_harness::encrypt_message;
use dal_test::{test, WorkspaceSignup};

/// Run with the following environment variable:
/// ```
/// SI_TEST_BUILTIN_SCHEMAS=test
/// ```
#[test]
async fn secret_definition_works_with_dummy_qualification(
    ctx: &mut DalContext,
    nw: &WorkspaceSignup,
) {
    let secret_definition_schema = Schema::find_by_name(ctx, "bethesda-secret")
        .await
        .expect("could not find schema")
        .expect("schema not found");
    let secret_definition_schema_variant =
        SchemaVariant::list_for_schema(ctx, secret_definition_schema.id())
            .await
            .expect("failed listing schema variants")
            .pop()
            .expect("no schema variant found");
    let secret_definition_schema_variant_id = secret_definition_schema_variant.id();

    let secret_definition_component = Component::new(
        ctx,
        "secret-definition",
        secret_definition_schema_variant_id,
        None,
    )
    .await
    .expect("could not create component");
    let secret_definition_component_id = secret_definition_component.id();

    // This is the name of the secret definition from the "BethesdaSecret" test exclusive schema.
    let secret_definition_name = "fake";

    // Cache the output socket that will contain the secret id.
    let output_socket = ExternalProvider::find_with_name(
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
            ExternalProvider::attribute_values_for_external_provider_id(ctx, output_socket.id())
                .await
                .expect("could not perform attribute values for external provider id");
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

        // TODO(nick): restore the qualification check.
        // // Check that the qualification fails.
        // let mut qualifications =
        //     Component::list_qualifications(ctx, secret_definition_component_id)
        //         .await
        //         .expect("could not list qualifications");
        // let qualification = qualifications.pop().expect("no qualifications found");
        // assert!(qualifications.is_empty());
        // assert_eq!(
        //     QualificationSubCheckStatus::Failure, // expected
        //     qualification.result.expect("no result found").status  // actual
        // );
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
            ExternalProvider::attribute_values_for_external_provider_id(ctx, output_socket.id())
                .await
                .expect("could not perform attribute values for external provider id");
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

        // TODO(nick): restore the qualification check.
        // // Check that the qualification passes.
        // let mut qualifications =
        //     Component::list_qualifications(ctx, secret_definition_component_id)
        //         .await
        //         .expect("could not list qualifications");
        // let qualification = qualifications.pop().expect("no qualifications found");
        // assert!(qualifications.is_empty());
        // assert_eq!(
        //     QualificationSubCheckStatus::Success, // expected
        //     qualification.result.expect("no result found").status  // actual
        // );
    }
}
