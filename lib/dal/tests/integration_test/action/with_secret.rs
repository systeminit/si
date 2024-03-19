use dal::prop::PropPath;
use dal::property_editor::values::PropertyEditorValues;
use dal::{
    Action, ActionKind, ActionPrototype, AttributeValue, ChangeSetPointer, Component, DalContext,
    EncryptedSecret, InputSocket, OutputSocket, Prop,
};
use dal_test::test_harness::{create_component_for_schema_name, encrypt_message};
use dal_test::{test, WorkspaceSignup};

#[test]
async fn create_action_using_secret(ctx: &mut DalContext, nw: &WorkspaceSignup) {
    let source_component = create_component_for_schema_name(ctx, "dummy-secret", "source").await;
    let source_schema_variant_id = Component::schema_variant_id(ctx, source_component.id())
        .await
        .expect("could not get schema variant id for component");

    let destination_component = create_component_for_schema_name(ctx, "fallout", "source").await;
    let destination_schema_variant_id =
        Component::schema_variant_id(ctx, destination_component.id())
            .await
            .expect("could not get schema variant id for component");

    // Ensure the destination component has the actions that it needs.
    let list = Action::for_component(ctx, destination_component.id())
        .await
        .expect("unable to list actions for component");
    dbg!(list);
    let component = create_component_for_schema_name(ctx, "starfield", "pooop").await;
    let list = Action::for_component(ctx, component.id())
        .await
        .expect("unable to list actions for component");
    dbg!(list);

    // This is the name of the secret definition from the test exclusive schema.
    let secret_definition_name = "dummy";

    // Cache the prop we need for attribute value update.
    let reference_to_secret_prop = Prop::find_prop_by_path(
        ctx,
        source_schema_variant_id,
        &PropPath::new(["root", "secrets", secret_definition_name]),
    )
    .await
    .expect("could not find prop by path");

    // Connect the two components to propagate the secret value.
    let source_output_socket = OutputSocket::find_with_name(ctx, "dummy", source_schema_variant_id)
        .await
        .expect("could not perform find with name")
        .expect("output socket not found by name");
    let destination_input_socket =
        InputSocket::find_with_name(ctx, "dummy", destination_schema_variant_id)
            .await
            .expect("could not perform find with name")
            .expect("input socket not found by name");
    Component::connect(
        ctx,
        source_component.id(),
        source_output_socket.id(),
        destination_component.id(),
        destination_input_socket.id(),
    )
    .await
    .expect("could not connect");

    // Commit and update snapshot to visibility.
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    // Create and use a secret.
    let secret = EncryptedSecret::new(
        ctx,
        "johnqt",
        secret_definition_name.to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}]).await,
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret");
    let property_values = PropertyEditorValues::assemble(ctx, source_component.id())
        .await
        .expect("unable to list prop values");
    let reference_to_secret_attribute_value_id = property_values
        .find_by_prop_id(reference_to_secret_prop.id)
        .expect("could not find attribute value");
    AttributeValue::update(
        ctx,
        reference_to_secret_attribute_value_id,
        Some(serde_json::json!(secret.id().to_string())),
    )
    .await
    .expect("unable to perform attribute value update");

    // Commit and update snapshot to visibility.
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");

    dbg!(destination_component
        .materialized_view(ctx)
        .await
        .unwrap()
        .unwrap());

    let applied_change_set = ChangeSetPointer::apply_to_base_change_set(ctx)
        .await
        .expect("could not do this shit");
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(
        applied_change_set.base_change_set_id.unwrap(),
    )
    .await
    .unwrap();

    dbg!(destination_component
        .materialized_view(ctx)
        .await
        .unwrap()
        .unwrap());
    assert!(false);
}
