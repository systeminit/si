use dal::attribute::prototype::argument::AttributePrototypeArgument;
use dal::diagram::Diagram;
use dal::{
    AttributeValue, Component, DalContext, InputSocket, OutputSocket, Schema, SchemaVariant,
};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;

#[test]
async fn connect_components(ctx: &mut DalContext) {
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

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    // Create a second component for a second source
    let lunch_component =
        Component::new(ctx, "were saving for lunch", docker_image_schema_variant_id)
            .await
            .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component 2 creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let royel_component = Component::new(ctx, "royel otis", butane_schema_variant_id)
        .await
        .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after butane component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

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

    ctx.blocking_commit().await.expect("blocking commit failed");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

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

    ctx.blocking_commit().await.expect("blocking commit failed");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    //dbg!(royel_component.incoming_connections(ctx).await.expect("ok"));

    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let materialized_view = AttributeValue::get_by_id(ctx, units_value_id)
        .await
        .expect("value exists")
        .materialized_view(ctx)
        .await
        .expect("able to get units materialized_view")
        .expect("units has a materialized_view");

    dbg!(lunch_component
        .materialized_view(ctx)
        .await
        .expect("get docker image materialized_view"));

    assert!(matches!(materialized_view, serde_json::Value::Array(_)));

    if let serde_json::Value::Array(units_array) = materialized_view {
        assert_eq!(2, units_array.len())
    }

    // Assemble the diagram and check the edges.
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble the diagram");
    assert_eq!(2, diagram.edges.len());
}

#[test]
async fn connect_to_one_destination_with_multiple_candidates_of_same_schema_variant_on_diagram(
    ctx: &mut DalContext,
) {
    let source = create_component_for_schema_name(ctx, "fallout", "source").await;
    let source_sv_id = Component::schema_variant_id(ctx, source.id())
        .await
        .expect("find variant id for component");

    let destination = create_component_for_schema_name(ctx, "starfield", "destination").await;
    let destination_sv_id = Component::schema_variant_id(ctx, destination.id())
        .await
        .expect("find variant id for component");
    create_component_for_schema_name(ctx, "starfield", "not destination").await;

    ctx.blocking_commit()
        .await
        .expect("blocking commit after butane component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let output_socket = OutputSocket::find_with_name(ctx, "bethesda", source_sv_id)
        .await
        .expect("could not perform find output socket")
        .expect("output socket not found");

    let input_socket = InputSocket::find_with_name(ctx, "bethesda", destination_sv_id)
        .await
        .expect("could not perform find input socket")
        .expect("input socket not found");

    Component::connect(
        ctx,
        source.id(),
        output_socket.id(),
        destination.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after butane component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble the diagram");

    assert_eq!(diagram.components.len(), 3);
    assert_eq!(diagram.edges.len(), 1);
    let edge = &diagram.edges[0];
    assert_eq!(edge.from_component_id, source.id());
    assert_eq!(edge.to_component_id, destination.id());
}

#[test]
async fn remove_connection(ctx: &mut DalContext) {
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

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    // Create a second component for a second source
    let lunch_component =
        Component::new(ctx, "were saving for lunch", docker_image_schema_variant_id)
            .await
            .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component 2 creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let royel_component = Component::new(ctx, "royel otis", butane_schema_variant_id)
        .await
        .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after butane component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    // Connect the components!
    let inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        oysters_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components")
    .expect("duplicate connection");

    ctx.blocking_commit().await.expect("blocking commit failed");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

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

    ctx.blocking_commit().await.expect("blocking commit failed");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    //dbg!(royel_component.incoming_connections(ctx).await.expect("ok"));

    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let materialized_view = AttributeValue::get_by_id(ctx, units_value_id)
        .await
        .expect("value exists")
        .materialized_view(ctx)
        .await
        .expect("able to get units materialized_view")
        .expect("units has a materialized_view");

    dbg!(lunch_component
        .materialized_view(ctx)
        .await
        .expect("get docker image materialized_view"));

    assert!(matches!(materialized_view, serde_json::Value::Array(_)));

    if let serde_json::Value::Array(units_array) = materialized_view {
        assert_eq!(2, units_array.len())
    }

    // Assemble the diagram and check the edges.
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble the diagram");
    assert_eq!(2, diagram.edges.len());

    // Disconnect Component 1
    AttributePrototypeArgument::remove(ctx, inter_component_attribute_prototype_argument_id)
        .await
        .expect("Unable to remove inter component attribute prototype argument");

    ctx.blocking_commit().await.expect("blocking commit failed");
    ctx.update_snapshot_to_visibility()
        .await
        .expect("Unable to update snapshot to visibility");

    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let materialized_view = AttributeValue::get_by_id(ctx, units_value_id)
        .await
        .expect("value exists")
        .materialized_view(ctx)
        .await
        .expect("able to get units materialized_view")
        .expect("units has a materialized_view");

    dbg!(lunch_component
        .materialized_view(ctx)
        .await
        .expect("get docker image materialized_view"));

    assert!(matches!(materialized_view, serde_json::Value::Array(_)));

    if let serde_json::Value::Array(units_array) = materialized_view {
        assert_eq!(1, units_array.len())
    }

    // Assemble the diagram and check the edges.
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble the diagram");
    assert_eq!(1, diagram.edges.len());
}
