use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::Action;
use dal::component::resource::ResourceData;
use dal::func::intrinsics::IntrinsicFunc;
use dal::{AttributeValue, Func, InputSocket, OutputSocket};
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::helpers::create_component_for_default_schema_name;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

#[test]
async fn delete(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "swifty", "shake it off")
        .await
        .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(component
        .delete(ctx)
        .await
        .expect("unable to delete component")
        .is_none());
}

#[test]
async fn delete_enqueues_destroy_action(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "dummy-secret", "component")
        .await
        .expect("could not create component");
    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );
    component
        .set_resource(ctx, resource_data)
        .await
        .expect("Unable to set resource");
    let schema_variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("Unable to get schema variant id");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    ActionPrototype::new(
        ctx,
        ActionKind::Destroy,
        "Destroy action".to_string(),
        None,
        schema_variant_id,
        Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
            .await
            .expect("Unable to find identity func"),
    )
    .await
    .expect("Unable to create destroy action");

    assert!(Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions")
        .is_empty());

    component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion");

    let action_ids = Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions");
    assert_eq!(1, action_ids.len());
}

#[test]
async fn delete_on_already_to_delete_does_not_enqueue_destroy_action(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name(ctx, "dummy-secret", "component")
        .await
        .expect("could not create component");
    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );
    component
        .set_resource(ctx, resource_data)
        .await
        .expect("Unable to set resource");
    let schema_variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("Unable to get schema variant id");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    ActionPrototype::new(
        ctx,
        ActionKind::Destroy,
        "Destroy action".to_string(),
        None,
        schema_variant_id,
        Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
            .await
            .expect("Unable to find identity func"),
    )
    .await
    .expect("Unable to create destroy action");

    assert!(Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions")
        .is_empty());

    let component = component
        .set_to_delete(ctx, true)
        .await
        .expect("Unable to set to_delete");

    let action_ids = Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions");
    assert_eq!(1, action_ids.len());
    for action_id in action_ids {
        Action::remove_by_id(ctx, action_id)
            .await
            .expect("Unable to remove action");
    }

    assert!(Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions")
        .is_empty());

    component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion");

    assert!(Action::all_ids(ctx)
        .await
        .expect("Unable to list enqueued actions")
        .is_empty());
}

#[test]
async fn delete_updates_downstream_components(ctx: &mut DalContext) {
    // Get the source schema variant id.
    let docker_image_schema = Schema::find_by_name(ctx, "Docker Image")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let mut docker_image_schema_variants =
        SchemaVariant::list_for_schema(ctx, docker_image_schema.id())
            .await
            .expect("could not list schema variants for schema");
    let docker_image_schema_variant = docker_image_schema_variants
        .pop()
        .expect("schema variants are empty");
    let docker_image_schema_variant_id = docker_image_schema_variant.id();

    // Get the destination schema variant id.
    let butane_schema = Schema::find_by_name(ctx, "Butane")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let mut butane_schema_variants = SchemaVariant::list_for_schema(ctx, butane_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let butane_schema_variant = butane_schema_variants
        .pop()
        .expect("schema variants are empty");
    let butane_schema_variant_id = butane_schema_variant.id();

    // Find the sockets we want to use.
    let output_socket =
        OutputSocket::find_with_name(ctx, "Container Image", docker_image_schema_variant_id)
            .await
            .expect("could not perform find output socket")
            .expect("output socket not found");
    let input_socket =
        InputSocket::find_with_name(ctx, "Container Image", butane_schema_variant_id)
            .await
            .expect("could not perform find input socket")
            .expect("input socket not found");

    // Create a component for both the source and the destination
    let oysters_component =
        Component::new(ctx, "oysters in my pocket", docker_image_schema_variant_id)
            .await
            .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a second component for a second source
    let lunch_component =
        Component::new(ctx, "were saving for lunch", docker_image_schema_variant_id)
            .await
            .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let royel_component = Component::new(ctx, "royel otis", butane_schema_variant_id)
        .await
        .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect the components!
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        oysters_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect component 2
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        lunch_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::get_by_id_or_error(ctx, units_value_id)
        .await
        .expect("value exists")
        .view(ctx)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));

    oysters_component
        .set_resource(
            ctx,
            ResourceData::new(
                ResourceStatus::Ok,
                Some(serde_json::json!({
                    "key": "value",
                })),
            ),
        )
        .await
        .expect("unable to ser resource");

    // Delete component.
    let _oysters_component = oysters_component
        .delete(ctx)
        .await
        .expect("Unable to delete oysters component")
        .expect("component fully deleted");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify post-update data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::get_by_id_or_error(ctx, units_value_id)
        .await
        .expect("value exists")
        .view(ctx)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(!units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));
}

#[test]
async fn delete_undo_updates_inputs(ctx: &mut DalContext) {
    // Get the source schema variant id.
    let docker_image_schema = Schema::find_by_name(ctx, "Docker Image")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let mut docker_image_schema_variants =
        SchemaVariant::list_for_schema(ctx, docker_image_schema.id())
            .await
            .expect("could not list schema variants for schema");
    let docker_image_schema_variant = docker_image_schema_variants
        .pop()
        .expect("schema variants are empty");
    let docker_image_schema_variant_id = docker_image_schema_variant.id();

    // Get the destination schema variant id.
    let butane_schema = Schema::find_by_name(ctx, "Butane")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let mut butane_schema_variants = SchemaVariant::list_for_schema(ctx, butane_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let butane_schema_variant = butane_schema_variants
        .pop()
        .expect("schema variants are empty");
    let butane_schema_variant_id = butane_schema_variant.id();

    // Find the sockets we want to use.
    let output_socket =
        OutputSocket::find_with_name(ctx, "Container Image", docker_image_schema_variant_id)
            .await
            .expect("could not perform find output socket")
            .expect("output socket not found");
    let input_socket =
        InputSocket::find_with_name(ctx, "Container Image", butane_schema_variant_id)
            .await
            .expect("could not perform find input socket")
            .expect("input socket not found");

    // Create a component for both the source and the destination
    let oysters_component =
        Component::new(ctx, "oysters in my pocket", docker_image_schema_variant_id)
            .await
            .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a second component for a second source
    let lunch_component =
        Component::new(ctx, "were saving for lunch", docker_image_schema_variant_id)
            .await
            .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let royel_component = Component::new(ctx, "royel otis", butane_schema_variant_id)
        .await
        .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect the components!
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        oysters_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect component 2
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        lunch_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::get_by_id_or_error(ctx, units_value_id)
        .await
        .expect("value exists")
        .view(ctx)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    assert!(units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));

    oysters_component
        .set_resource(
            ctx,
            ResourceData::new(
                ResourceStatus::Ok,
                Some(serde_json::json!({
                    "key": "value",
                })),
            ),
        )
        .await
        .expect("unable to ser resource");

    // Delete component.
    let _oysters_component = oysters_component
        .delete(ctx)
        .await
        .expect("Unable to delete oysters component")
        .expect("component fully deleted");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify post-update data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::get_by_id_or_error(ctx, units_value_id)
        .await
        .expect("value exists")
        .view(ctx)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(!units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));

    royel_component
        .set_resource(
            ctx,
            ResourceData::new(
                ResourceStatus::Ok,
                Some(serde_json::json!({
                    "key": "value",
                })),
            ),
        )
        .await
        .expect("unable to ser resource");

    // Delete the destination component, so it pulls data from both the deleted & not deleted
    // components.
    let royel_component = royel_component
        .delete(ctx)
        .await
        .expect("Unable to delete royel component")
        .expect("component got deleted");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify post-delete data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::get_by_id_or_error(ctx, units_value_id)
        .await
        .expect("value exists")
        .view(ctx)
        .await
        .expect("able to get units view")
        .expect("units has a view");

    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));

    let royel_component = royel_component
        .set_to_delete(ctx, false)
        .await
        .expect("Unable to clear to_delete");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify post clear to_delete data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::get_by_id_or_error(ctx, units_value_id)
        .await
        .expect("value exists")
        .view(ctx)
        .await
        .expect("able to get units view")
        .expect("units has a view");

    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(!units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));
}
