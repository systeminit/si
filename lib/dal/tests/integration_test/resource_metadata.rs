use dal::{
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    Prop,
    Secret,
    Workspace,
    prop::PropPath,
    property_editor::values::PropertyEditorValues,
    resource_metadata,
};
use dal_test::{
    WorkspaceSignup,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        encrypt_message,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_events::{
    ResourceMetadata,
    ResourceStatus,
};

#[test]
async fn list(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    let source_component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "source")
            .await
            .expect("could not create component");
    let source_schema_variant_id = Component::schema_variant_id(ctx, source_component.id())
        .await
        .expect("could not get schema variant id for component");
    let destination_component =
        create_component_for_default_schema_name_in_default_view(ctx, "fallout", "destination")
            .await
            .expect("could not create component");
    let destination_schema_variant_id =
        Component::schema_variant_id(ctx, destination_component.id())
            .await
            .expect("could not get schema variant id for component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Cache the name of the secret definition from the test exclusive schema. Afterward, cache the
    // prop we need for attribute value update.
    let secret_definition_name = "dummy";
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        source_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // Connect the two components to propagate the secret value and commit.
    let source_output_socket = OutputSocket::find_with_name(ctx, "dummy", source_schema_variant_id)
        .await
        .expect("could not perform find with name")
        .expect("output socket not found by name");
    let destination_input_socket =
        InputSocket::find_with_name(ctx, "dummy", destination_schema_variant_id)
            .await
            .expect("could not perform find with name")
            .expect("input socket not found by name");
    Component::connect_for_tests(
        ctx,
        source_component.id(),
        source_output_socket.id(),
        destination_component.id(),
        destination_input_socket.id(),
    )
    .await
    .expect("could not connect");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create the secret and commit.
    let secret = Secret::new(
        ctx,
        "toto wolff",
        secret_definition_name.to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message"),
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Use the secret in the source component and commit.
    let property_values = PropertyEditorValues::assemble(ctx, source_component.id())
        .await
        .expect("unable to list prop values");
    let reference_to_secret_attribute_value_id = property_values
        .find_by_prop_id(reference_to_secret_prop.id)
        .expect("could not find attribute value");
    Secret::attach_for_attribute_value(
        ctx,
        reference_to_secret_attribute_value_id,
        Some(secret.id()),
    )
    .await
    .expect("could not attach secret");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Set the workspace token to mimic how it works with the auth-api.
    Workspace::get_by_pk(ctx, ctx.tenancy().workspace_pk().expect("workspace"))
        .await
        .expect("could not get workspace")
        .set_token(ctx, "token".to_string())
        .await
        .expect("could not set token");

    // Ensure that the parent is head so that the "create" action will execute by default.
    // Technically, this primarily validates the test setup rather than the system itself, but it
    // serves a secondary function of ensuring no prior functions cause this assertion to fail.
    assert!(
        ctx.parent_is_head()
            .await
            .expect("could not perform parent is head")
    );

    // Apply to the base change set and commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Wait for all actions to run.
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx)
        .await
        .expect("deadline for actions to run exceeded");

    // Validate that both components look as expected on HEAD.
    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "source",
                "type": "component",
            },
            "secrets": {
                "dummy": secret.encrypted_secret_key().to_string()
            },
            "resource_value": {},
            "qualification": {
                "test:qualificationDummySecretStringIsTodd": {
                    "result": "success",
                    "message": "dummy secret string matches expected value"
                },
            },
        }], // expected
        source_component
            .view(ctx)
            .await
            .expect("could not get materialized view")
            .expect("empty materialized view") // actual
    );
    let last_synced_av_id = destination_component
        .attribute_values_for_prop(ctx, &["root", "resource", "last_synced"])
        .await
        .expect("should be able to find avs for last synced")
        .pop()
        .expect("should have an av for last synced");
    let last_synced_value = AttributeValue::view(ctx, last_synced_av_id)
        .await
        .expect("should be able to get value for last synced av");
    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "destination",
                "type": "component",
            },
            "domain": {
                "name": "destination",
                "active": true
            },
            "secrets": {
                "dummy": secret.encrypted_secret_key().to_string()
            },
            "resource": {
                "status": "ok",
                "payload": { "poop" :true },
                "last_synced": last_synced_value.clone().unwrap_or(serde_json::Value::Null),
            },
            "resource_value": {}
        }], // expected
        destination_component
            .view(ctx)
            .await
            .expect("could not get materialized view")
            .expect("empty materialized view") // actual
    );

    // Finally, we can collect the resource metadata.
    let metadata = resource_metadata::list(ctx)
        .await
        .expect("could not collect resource metadata");
    let expected = ResourceMetadata {
        component_id: destination_component.id(),
        status: ResourceStatus::Ok,
        last_synced: serde_json::from_value(last_synced_value.unwrap_or(serde_json::Value::Null))
            .expect("could not deserialize"),
    };
    assert_eq!(
        vec![expected], // expected
        metadata,       // actual
    );
}
