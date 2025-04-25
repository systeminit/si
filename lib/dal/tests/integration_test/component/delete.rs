use std::time::Duration;

use dal::{
    AttributeValue,
    Component,
    ComponentType,
    DalContext,
    Func,
    InputSocket,
    OutputSocket,
    Schema,
    SchemaVariant,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    component::{
        delete::{
            ComponentDeletionStatus,
            delete_components,
        },
        frame::Frame,
        resource::ResourceData,
    },
    func::intrinsics::IntrinsicFunc,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        create_component_for_schema_name_with_type_on_default_view,
        create_named_component_for_schema_variant_on_default_view,
        get_component_input_socket_value,
        update_attribute_value_for_component,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

#[test]
async fn delete(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "shake it off")
            .await
            .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    assert!(
        component
            .delete(ctx)
            .await
            .expect("unable to delete component")
            .is_none()
    );
}

#[test]
async fn delete_enqueues_destroy_action(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "component")
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

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );

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
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "dummy-secret", "component")
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

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );

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

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );

    component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion");

    assert!(
        Action::all_ids(ctx)
            .await
            .expect("Unable to list enqueued actions")
            .is_empty()
    );
}

#[test]
async fn delete_updates_downstream_components(ctx: &mut DalContext) {
    // Get the source schema variant id.
    let docker_image_schema = Schema::get_by_name(ctx, "Docker Image")
        .await
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
    let butane_schema = Schema::get_by_name(ctx, "Butane")
        .await
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
    let oysters_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "oysters in my pocket",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a second component for a second source
    let lunch_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "were saving for lunch",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let royel_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "royel otis",
        butane_schema_variant_id,
    )
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

    let view = AttributeValue::get_by_id(ctx, units_value_id)
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

    let view = AttributeValue::get_by_id(ctx, units_value_id)
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
    let docker_image_schema = Schema::get_by_name(ctx, "Docker Image")
        .await
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
    let butane_schema = Schema::get_by_name(ctx, "Butane")
        .await
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
    let oysters_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "oysters in my pocket",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a second component for a second source
    let lunch_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "were saving for lunch",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let royel_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "royel otis",
        butane_schema_variant_id,
    )
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

    let view = AttributeValue::get_by_id(ctx, units_value_id)
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

    let view = AttributeValue::get_by_id(ctx, units_value_id)
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

    let view = AttributeValue::get_by_id(ctx, units_value_id)
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

    let view = AttributeValue::get_by_id(ctx, units_value_id)
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
async fn delete_with_frames_without_resources(ctx: &mut DalContext) {
    // Scenario:
    // 1. Create a 3 level nested frame
    // 2. Remove only the middle one (which is not really possible via the UI but that's ok)
    // 3. Make sure the component is re-parented to the outer most frame and that data flows as expected
    // create a frame
    let outer_frame = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large odd lego",
        "large odd",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // cache id to use throughout the test
    let outer_frame_id = outer_frame.id();

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // set a value on the outer frame that will pass to the component
    update_attribute_value_for_component(
        ctx,
        outer_frame_id,
        &["root", "domain", "six"],
        serde_json::json!["6"],
    )
    .await
    .expect("could not set attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // create another frame that won't pass data
    let inner_frame = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "swifty",
        "swifty",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let inner_frame_id = inner_frame.id();

    Frame::upsert_parent(ctx, inner_frame.id(), outer_frame.id())
        .await
        .expect("could not upsert frame");

    // create a component that takes input from the top most
    let component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large even lego",
        "large even",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");

    let component_id = component.id();

    Frame::upsert_parent(ctx, component.id(), inner_frame.id())
        .await
        .expect("could not upsert frame");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // make sure values propagated accordingly
    let component_av_six = get_component_input_socket_value(ctx, component_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six, "6");

    // delete inner frame
    let inner_component = Component::get_by_id(ctx, inner_frame_id)
        .await
        .expect("could not get component");
    let deleted_inner = inner_component.delete(ctx).await.expect("could not delete");

    // component is really removed
    assert!(deleted_inner.is_none());

    // ensure component is re-parented
    let component = Component::get_by_id(ctx, component_id)
        .await
        .expect("could not get component");
    assert_eq!(
        component
            .parent(ctx)
            .await
            .expect("could not get parent")
            .expect("is some"),
        outer_frame_id
    );
}
#[test]
async fn delete_with_frames_and_resources(ctx: &mut DalContext) {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");
    // create a frame
    let outer_frame = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large odd lego",
        "large odd",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");
    let outer_frame_id = outer_frame.id();

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // cache id to use throughout the test

    // set a value on the outer frame that will pass to the component
    update_attribute_value_for_component(
        ctx,
        outer_frame_id,
        &["root", "domain", "six"],
        serde_json::json!["6"],
    )
    .await
    .expect("could not set attribute value");

    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );
    outer_frame
        .set_resource(ctx, resource_data.clone())
        .await
        .expect("could not set resource");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // create another frame that won't pass data
    let inner_frame = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "swifty",
        "swifty",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let inner_frame_id = inner_frame.id();

    Frame::upsert_parent(ctx, inner_frame.id(), outer_frame.id())
        .await
        .expect("could not upsert frame");

    // create a component that takes input from the top most
    let component = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large even lego",
        "large even",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");
    component
        .set_resource(ctx, resource_data)
        .await
        .expect("could not set resource");
    let component_id = component.id();

    Frame::upsert_parent(ctx, component.id(), inner_frame.id())
        .await
        .expect("could not upsert frame");

    // let's remove all create actions and set the resource manually to simulate create + refresh
    let all_actions = Action::list_topologically(ctx)
        .await
        .expect("could not get actions");
    for action in all_actions {
        Action::remove_by_id(ctx, action)
            .await
            .expect("could not remove action");
    }
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // make sure values propagated accordingly
    let component_av_six = get_component_input_socket_value(ctx, component_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six, "6");

    // Apply to the base change set to simulate running actions
    assert!(
        ctx.parent_is_head()
            .await
            .expect("could not perform parent is head")
    );

    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // check that the resource is set for the frame
    let outer_frame = Component::get_by_id(ctx, outer_frame_id)
        .await
        .expect("could not get component");
    let outer_frame_resource = outer_frame
        .resource(ctx)
        .await
        .expect("could not get resource");
    assert!(outer_frame_resource.is_some());
    let resource = outer_frame_resource.expect("is some");
    assert_eq!(
        resource.payload,
        Some(serde_json::json![{"resource": "something"}])
    );
    assert_eq!(resource.status, ResourceStatus::Ok);

    let component = Component::get_by_id(ctx, component_id)
        .await
        .expect("could not get component");
    let component_resource = component
        .resource(ctx)
        .await
        .expect("could not get resource");
    assert!(component_resource.is_some());
    let resource = component_resource.expect("is some");
    assert_eq!(
        resource.payload,
        Some(serde_json::json![{"resource": "something"}])
    );
    assert_eq!(resource.status, ResourceStatus::Ok);

    // check there's no resource for swifty
    let inner_frame = Component::get_by_id(ctx, inner_frame_id)
        .await
        .expect("coudl not get component");
    assert!(
        inner_frame
            .resource(ctx)
            .await
            .expect("could not get resource")
            .is_none()
    );

    // Fork Head
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");

    // delete all components (as if you deleted the parent in sdf)
    // ensure everything is set to delete
    let outer_frame = Component::get_by_id(ctx, outer_frame_id)
        .await
        .expect("could not get component");
    let deleted_outer = outer_frame
        .delete(ctx)
        .await
        .expect("could not delete component");
    assert!(deleted_outer.is_some());

    let inner_frame = Component::get_by_id(ctx, inner_frame_id)
        .await
        .expect("could not get component");
    let deleted_inner = inner_frame
        .delete(ctx)
        .await
        .expect("could not delete component");
    assert!(deleted_inner.is_some());

    let component = Component::get_by_id(ctx, component_id)
        .await
        .expect("could not get component");
    let deleted_component = component
        .delete(ctx)
        .await
        .expect("could not delete component")
        .expect("is some");

    assert!(&deleted_component.to_delete());

    let components_to_delete = Component::list_to_be_deleted(ctx)
        .await
        .expect("could not list components to be deleted");
    assert_eq!(components_to_delete.len(), 3);

    // make sure values propagated accordingly
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    let component_av_six = get_component_input_socket_value(ctx, component_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six, "6");

    // make sure there are 3 components on the diagram
    let all_components = Component::list(ctx)
        .await
        .expect("could not list components");
    assert_eq!(all_components.len(), 3);

    // now manually remove the resource of the outer_frame (so only one delete action has to run)
    let outer_frame = Component::get_by_id(ctx, outer_frame_id)
        .await
        .expect("could not get component");
    outer_frame
        .clear_resource(ctx)
        .await
        .expect("could not clear the resource");
    // dequeue the delete actions for this component
    let actions_for_outer = Action::find_for_component_id(ctx, outer_frame_id)
        .await
        .expect("could not list actions for outer frame");
    assert_eq!(actions_for_outer.len(), 1);
    Action::remove_all_for_component_id(ctx, outer_frame_id)
        .await
        .expect("could not remove actions");
    let actions_for_swifty = Action::find_for_component_id(ctx, inner_frame_id)
        .await
        .expect("could not find actions for inner frame");
    assert_eq!(actions_for_swifty.len(), 1);
    Action::remove_all_for_component_id(ctx, inner_frame_id)
        .await
        .expect("could not remove actions for inner");

    // should only be one action left for the component
    let actions_enqueued = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    assert_eq!(actions_enqueued.len(), 1);
    let action = Action::find_for_component_id(ctx, component_id)
        .await
        .expect("could not find actions for component");
    assert_eq!(action, actions_enqueued);
    let action_enqueued = Action::get_by_id(ctx, *actions_enqueued.first().expect("is some"))
        .await
        .expect("could not get action");
    assert_eq!(action_enqueued.is_eligible_to_dispatch(), true);

    // make sure values propagated accordingly
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    let component_av_six = get_component_input_socket_value(ctx, component_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six, "6");

    // apply and let the one delete action run
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply to head");
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx)
        .await
        .expect("could not run actions");
    // loop until the other components are removed
    let total_count = 50;
    let mut count = 0;

    while count < total_count {
        ctx.update_snapshot_to_visibility()
            .await
            .expect("could not update snapshot");
        let components = Component::list(ctx)
            .await
            .expect("could not list components");
        if components.is_empty() {
            break;
        }
        count += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let components = Component::list(ctx)
        .await
        .expect("could not list components");
    // make sure there are no more components left!
    assert_eq!(components.len(), 0);
}

#[test]
async fn delete_with_multiple_frames(ctx: &mut DalContext) {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");
    // create a frame
    let outer_frame = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large odd lego",
        "large odd",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");
    let outer_frame_id = outer_frame.id();

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // cache id to use throughout the test

    // set a value on the outer frame that will pass to the component
    update_attribute_value_for_component(
        ctx,
        outer_frame_id,
        &["root", "domain", "six"],
        serde_json::json!["6"],
    )
    .await
    .expect("could not set attribute value");

    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // create 2 other frames that won't pass data
    let inner_frame_1 = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "swifty",
        "swifty",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let inner_frame_1_id = inner_frame_1.id();
    let inner_frame_2 = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "swifty",
        "swifty 2",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    let inner_frame_id_2 = inner_frame_2.id();

    Frame::upsert_parent(ctx, inner_frame_1.id(), outer_frame.id())
        .await
        .expect("could not upsert frame");
    Frame::upsert_parent(ctx, inner_frame_2.id(), outer_frame.id())
        .await
        .expect("could not upsert frame");

    // create 2 components that take input from the top most, in each inner frame
    let component_1 = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large even lego",
        "large even",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");
    component_1
        .set_resource(ctx, resource_data.clone())
        .await
        .expect("could not set resource");
    let component_1_id = component_1.id();

    Frame::upsert_parent(ctx, component_1_id, inner_frame_1.id())
        .await
        .expect("could not upsert frame");

    let component_2 = create_component_for_schema_name_with_type_on_default_view(
        ctx,
        "large even lego",
        "large even 2",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");
    component_2
        .set_resource(ctx, resource_data)
        .await
        .expect("could not set resource");
    let component_2_id = component_2.id();

    Frame::upsert_parent(ctx, component_2_id, inner_frame_2.id())
        .await
        .expect("could not upsert frame");

    // let's remove all create actions and set the resource manually to simulate create + refresh
    let all_actions = Action::list_topologically(ctx)
        .await
        .expect("could not get actions");
    for action in all_actions {
        Action::remove_by_id(ctx, action)
            .await
            .expect("could not remove action");
    }
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // make sure values propagated accordingly
    let component_1_av_six = get_component_input_socket_value(ctx, component_2_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_1_av_six, "6");
    let component_2_av_six = get_component_input_socket_value(ctx, component_2_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_2_av_six, "6");

    // Apply to the base change set to simulate running actions
    assert!(
        ctx.parent_is_head()
            .await
            .expect("could not perform parent is head")
    );

    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // check that the resource is not set for the outer most frame
    let outer_frame = Component::get_by_id(ctx, outer_frame_id)
        .await
        .expect("could not get component");
    assert!(
        outer_frame
            .resource(ctx)
            .await
            .expect("could not get resource")
            .is_none()
    );

    // both inner components have resources
    let component_1 = Component::get_by_id(ctx, component_2_id)
        .await
        .expect("could not get component");
    let component_1_resource = component_1
        .resource(ctx)
        .await
        .expect("could not get resource");
    assert!(component_1_resource.is_some());
    let resource_1 = component_1_resource.expect("is some");
    assert_eq!(
        resource_1.payload,
        Some(serde_json::json![{"resource": "something"}])
    );
    assert_eq!(resource_1.status, ResourceStatus::Ok);

    let component_1 = Component::get_by_id(ctx, component_1_id)
        .await
        .expect("could not get component");
    let component_1_resource = component_1
        .resource(ctx)
        .await
        .expect("could not get resource");
    assert!(component_1_resource.is_some());
    let resource_1 = component_1_resource.expect("is some");
    assert_eq!(
        resource_1.payload,
        Some(serde_json::json![{"resource": "something"}])
    );
    assert_eq!(resource_1.status, ResourceStatus::Ok);

    // check there's no resource for both inner frames
    let inner_frame_1 = Component::get_by_id(ctx, inner_frame_1_id)
        .await
        .expect("coudl not get component");
    assert!(
        inner_frame_1
            .resource(ctx)
            .await
            .expect("could not get resource")
            .is_none()
    );
    let inner_frame_2 = Component::get_by_id(ctx, inner_frame_id_2)
        .await
        .expect("coudl not get component");
    assert!(
        inner_frame_2
            .resource(ctx)
            .await
            .expect("could not get resource")
            .is_none()
    );

    // Fork Head
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");

    // delete all components (as if you deleted the parent in sdf)
    // ensure everything is set to delete
    let outer_frame = Component::get_by_id(ctx, outer_frame_id)
        .await
        .expect("could not get component");
    let deleted_outer = outer_frame
        .delete(ctx)
        .await
        .expect("could not delete component");
    assert!(deleted_outer.is_some());

    let inner_frame_1 = Component::get_by_id(ctx, inner_frame_1_id)
        .await
        .expect("could not get component");
    let deleted_inner_1 = inner_frame_1
        .delete(ctx)
        .await
        .expect("could not delete component");
    assert!(deleted_inner_1.is_some());

    let inner_frame_2 = Component::get_by_id(ctx, inner_frame_id_2)
        .await
        .expect("could not get component");
    let deleted_inner_1 = inner_frame_2
        .delete(ctx)
        .await
        .expect("could not delete component");
    assert!(deleted_inner_1.is_some());

    let component_1 = Component::get_by_id(ctx, component_1_id)
        .await
        .expect("could not get component");
    let deleted_component_1 = component_1
        .delete(ctx)
        .await
        .expect("could not delete component")
        .expect("is some");

    assert!(&deleted_component_1.to_delete());

    let component_2 = Component::get_by_id(ctx, component_2_id)
        .await
        .expect("could not get component");
    let deleted_component_2 = component_2
        .delete(ctx)
        .await
        .expect("could not delete component")
        .expect("is some");

    assert!(&deleted_component_2.to_delete());

    let components_to_delete = Component::list_to_be_deleted(ctx)
        .await
        .expect("could not list components to be deleted");
    assert_eq!(components_to_delete.len(), 5);

    // make sure values propagated accordingly
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    let component_av_six_1 = get_component_input_socket_value(ctx, component_1_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six_1, "6");

    let component_av_six_2 = get_component_input_socket_value(ctx, component_2_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six_2, "6");

    // make sure there are 5 components on the diagram
    let all_components = Component::list(ctx)
        .await
        .expect("could not list components");
    assert_eq!(all_components.len(), 5);

    // now manually remove the resource of all of the frames and dequeue their actions (so only the inner 2 component's delete action has to run)
    let outer_frame = Component::get_by_id(ctx, outer_frame_id)
        .await
        .expect("could not get component");
    outer_frame
        .clear_resource(ctx)
        .await
        .expect("could not clear the resource");
    // dequeue the delete actions for this component
    let actions_for_outer = Action::find_for_component_id(ctx, outer_frame_id)
        .await
        .expect("could not list actions for outer frame");
    assert_eq!(actions_for_outer.len(), 1);
    Action::remove_all_for_component_id(ctx, outer_frame_id)
        .await
        .expect("could not remove actions");
    let actions_for_inner_1 = Action::find_for_component_id(ctx, inner_frame_1_id)
        .await
        .expect("could not find actions for inner frame");
    assert_eq!(actions_for_inner_1.len(), 1);
    Action::remove_all_for_component_id(ctx, inner_frame_1_id)
        .await
        .expect("could not remove actions for inner");
    let actions_for_inner_2 = Action::find_for_component_id(ctx, inner_frame_id_2)
        .await
        .expect("could not find actions for inner frame");
    assert_eq!(actions_for_inner_2.len(), 1);
    Action::remove_all_for_component_id(ctx, inner_frame_id_2)
        .await
        .expect("could not remove actions for inner");

    // should only be two actions left for the components
    let actions_enqueued = Action::list_topologically(ctx)
        .await
        .expect("could not list actions");
    assert_eq!(actions_enqueued.len(), 2);
    for action_id in actions_enqueued {
        let action = Action::get_by_id(ctx, action_id)
            .await
            .expect("couldn't get action");
        assert_eq!(action.is_eligible_to_dispatch(), true);
    }

    // make sure values are still propagated accordingly
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    let component_av_six = get_component_input_socket_value(ctx, component_2_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six, "6");

    let component_av_six = get_component_input_socket_value(ctx, component_1_id, "six")
        .await
        .expect("could not get socket value")
        .expect("should have a value");
    assert_eq!(component_av_six, "6");

    // apply and let the two delete actions run
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply to head");
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx)
        .await
        .expect("could not run actions");
    // loop until the other components are removed
    let total_count = 50;
    let mut count = 0;
    while count < total_count {
        ctx.update_snapshot_to_visibility()
            .await
            .expect("could not update snapshot");
        let components = Component::list(ctx)
            .await
            .expect("could not list components");
        if components.is_empty() {
            break;
        }
        count += 1;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let components = Component::list(ctx)
        .await
        .expect("could not list components");
    // make sure there are no more components left!
    assert_eq!(components.len(), 0);
}

#[test]
async fn delete_multiple_components(ctx: &mut DalContext) -> Result<()> {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    let component_still_on_head = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component still on head",
    )
    .await?;

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;

    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;

    let component_with_resource_to_delete =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to delete",
        )
        .await?;

    let component_with_resource_to_erase =
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "small odd lego",
            "component with resource to erase",
        )
        .await?;

    let resource_data = ResourceData::new(
        ResourceStatus::Ok,
        Some(serde_json::json![{"resource": "something"}]),
    );

    component_with_resource_to_delete
        .set_resource(ctx, resource_data.clone())
        .await?;
    component_with_resource_to_erase
        .set_resource(ctx, resource_data.clone())
        .await?;

    let component_to_delete = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "component to delete",
    )
    .await?;

    let expected_deletion_statuses = &[
        (component_to_delete.id(), ComponentDeletionStatus::Deleted),
        (
            component_with_resource_to_delete.id(),
            ComponentDeletionStatus::MarkedForDeletion,
        ),
        (
            component_still_on_head.id(),
            ComponentDeletionStatus::StillExistsOnHead,
        ),
        (
            component_with_resource_to_erase.id(),
            ComponentDeletionStatus::Deleted,
        ),
    ];

    let mut deletion_statuses = delete_components(
        ctx,
        &[
            component_to_delete.id(),
            component_with_resource_to_delete.id(),
            component_still_on_head.id(),
        ],
        false,
    )
    .await?;

    deletion_statuses
        .extend(delete_components(ctx, &[component_with_resource_to_erase.id()], true).await?);

    for (component_id, status) in expected_deletion_statuses {
        assert_eq!(Some(status), deletion_statuses.get(component_id));
    }

    assert!(
        Component::try_get_by_id(ctx, component_to_delete.id())
            .await?
            .is_none(),
        "deleted component should be gone"
    );

    assert!(
        Component::try_get_by_id(ctx, component_still_on_head.id())
            .await?
            .is_none(),
        "deleted component that is still on head should be gone in this change set"
    );

    assert!(
        Component::exists_on_head(ctx, &[component_still_on_head.id()])
            .await?
            .contains(&component_still_on_head.id()),
        "component should still exist on head"
    );

    assert!(
        Component::try_get_by_id(ctx, component_with_resource_to_erase.id())
            .await?
            .is_none(),
        "erased component should be gone"
    );

    let component_with_resource_to_delete =
        Component::get_by_id(ctx, component_with_resource_to_delete.id()).await?;
    assert!(
        component_with_resource_to_delete.to_delete(),
        "component with resource should be marked as to delete"
    );

    Ok(())
}
