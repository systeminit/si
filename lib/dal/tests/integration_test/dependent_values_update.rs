use dal::{
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    Schema,
    SchemaVariant,
    component::resource::ResourceData,
    workspace_snapshot::DependentValueRoot,
};
use dal_test::{
    expected::{
        self,
        ExpectComponent,
    },
    helpers::{
        ChangeSetTestHelpers,
        create_named_component_for_schema_variant_on_default_view,
    },
    test,
};
use serde_json::json;
use veritech_client::ResourceStatus;

#[test]
async fn marked_for_deletion_to_normal_is_blocked(ctx: &mut DalContext) {
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

    let oysters_component = oysters_component
        .delete(ctx)
        .await
        .expect("Unable to mark for deletion")
        .expect("component got fully deleted");

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

    let view = AttributeValue::view(ctx, units_value_id)
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

    let view = AttributeValue::view(ctx, units_value_id)
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

    // Verify pre-delete data.
    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(units_json_string.contains("docker.io/library/oysters in my pocket\\n"));

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

    let view = AttributeValue::view(ctx, units_value_id)
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

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units_json_string = serde_json::to_string(&view).expect("Unable to stringify JSON");
    assert!(units_json_string.contains("docker.io/library/oysters on the floor\\n"));
}

/// Until we have a better system for signalling that a DVU has run and
/// finished, we can't actually verify that it executed these per-component. But
/// we can ensure that with a concurrency limit: (1) the job finishes and (2) it
/// produces the correct data
#[test]
async fn component_concurrency_limit(ctx: &mut DalContext) {
    // Give us a massive component concurrency level
    let mut workspace = ctx.get_workspace().await.expect("get workspace");
    workspace
        .set_component_concurrency_limit(ctx, Some(10000))
        .await
        .expect("set concurrency limit");
    ctx.commit_no_rebase().await.expect("commit");

    // create 1 etoile, and 16 morningstars
    let etoiles = ExpectComponent::create(ctx, "etoiles").await;

    let mut morningstars = vec![];
    for i in 0..16 {
        let name: String = (i + 1).to_string();
        let morningstar = ExpectComponent::create_named(ctx, "morningstar", name).await;

        etoiles
            .connect(
                ctx,
                "naming_and_necessity",
                morningstar,
                "naming_and_necessity",
            )
            .await;
        morningstars.push(morningstar);
    }

    assert!(
        DependentValueRoot::roots_exist(ctx)
            .await
            .expect("has dependent value roots"),
        "should have dvu roots to be processed"
    );

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    assert!(
        !DependentValueRoot::roots_exist(ctx)
            .await
            .expect("able to check for dependent value roots"),
        "all dvu roots should be processed and removed"
    );

    let mut workspace = ctx.get_workspace().await.expect("get workspace");
    workspace
        .set_component_concurrency_limit(ctx, Some(2))
        .await
        .expect("set concurrency limit");
    ctx.commit_no_rebase().await.expect("commit");

    let rigid_designator = etoiles
        .prop(
            ctx,
            [
                "root",
                "domain",
                "possible_world_a",
                "wormhole_1",
                "wormhole_2",
                "wormhole_3",
                "rigid_designator",
            ],
        )
        .await;
    rigid_designator.set(ctx, "hesperus").await;

    expected::commit_and_update_snapshot_to_visibility(ctx).await;

    assert!(
        !DependentValueRoot::roots_exist(ctx)
            .await
            .expect("call has dvu roots"),
        "all roots should be processed and off the graph"
    );

    assert!(
        !morningstars.is_empty(),
        "ensure we will do the checks below"
    );
    for morningstar in morningstars {
        let stars = morningstar.prop(ctx, ["root", "domain", "stars"]).await;
        assert_eq!(
            json!("phosphorus"),
            stars.get(ctx).await,
            "ensure values have flowed through to the morningstar components"
        )
    }
}
