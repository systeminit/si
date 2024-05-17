use chrono::Utc;
use dal::func::backend::js_action::DeprecatedActionRunResult;
use dal::{
    AttributeValue, Component, DalContext, InputSocket, OutputSocket, Schema, SchemaVariant,
};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;
use veritech_client::ResourceStatus;

#[test]
async fn marked_for_deletion_to_normal_is_blocked(ctx: &mut DalContext) {
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
    oysters_component
        .set_resource(
            ctx,
            DeprecatedActionRunResult {
                status: Some(ResourceStatus::Ok),
                payload: Some(serde_json::json!({
                    "key": "value",
                })),
                message: None,
                last_synced: Some(Utc::now().to_rfc3339()),
            },
        )
        .await
        .expect("unable to ser resource");

    let oysters_component = oysters_component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion")
        .expect("component got fully deleted");

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

    // Modify deleted component.
    let oysters_image_av_id = oysters_component
        .attribute_values_for_prop(ctx, &["root", "domain", "image"])
        .await
        .expect("Unable to get AV for domain/image")
        .first()
        .copied()
        .expect("AV for domain/image not found");

    AttributeValue::update(
        ctx,
        oysters_image_av_id,
        Some(serde_json::value::Value::String(
            "oysters on the floor".to_string(),
        )),
        // Some(serde_json::json!("oysters on the floor")),
    )
    .await
    .expect("Unable to update domain/image");

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
    assert!(!units_json_string.contains("docker.io/library/oysters on the floor\\n"));
    assert!(!units_json_string.contains("docker.io/library/oysters in my pocket\\n"));
    assert!(units_json_string.contains("docker.io/library/were saving for lunch\\n"));
}

#[test]
async fn normal_to_marked_for_deletion_flows(ctx: &mut DalContext) {
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

    // Verify pre-delete data.
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

    royel_component
        .set_resource(
            ctx,
            DeprecatedActionRunResult {
                status: Some(ResourceStatus::Ok),
                payload: Some(serde_json::json!({
                    "key": "value",
                })),
                message: None,
                last_synced: Some(Utc::now().to_rfc3339()),
            },
        )
        .await
        .expect("unable to ser resource");

    // "Delete" the Butane component
    let royel_component = royel_component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion")
        .expect("component got fully deleted");

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

    // Modify normal component.
    let oysters_image_av_id = oysters_component
        .attribute_values_for_prop(ctx, &["root", "domain", "image"])
        .await
        .expect("Unable to get AV for domain/image")
        .first()
        .copied()
        .expect("AV for domain/image not found");

    AttributeValue::update(
        ctx,
        oysters_image_av_id,
        Some(serde_json::value::Value::String(
            "oysters on the floor".to_string(),
        )),
        // Some(serde_json::json!("oysters on the floor")),
    )
    .await
    .expect("Unable to update domain/image");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Verify post-delete updated data.
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
    assert!(units_json_string.contains("docker.io/library/oysters on the floor\\n"));
}
