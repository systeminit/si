use dal::diagram::Diagram;
use dal::{
    AttributeValue, Component, DalContext, ExternalProvider, InternalProvider, Schema,
    SchemaVariant,
};
use dal_test::test;

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

    // Find the providers we want to use.
    let docker_image_external_providers =
        ExternalProvider::list(ctx, docker_image_schema_variant_id)
            .await
            .expect("could not list external providers");
    let external_provider = docker_image_external_providers
        .iter()
        .find(|e| e.name() == "Container Image")
        .expect("could not find external provider");
    let butane_explicit_internal_providers = InternalProvider::list(ctx, butane_schema_variant_id)
        .await
        .expect("could not list explicit internal providers");
    let explicit_internal_provider = butane_explicit_internal_providers
        .iter()
        .find(|e| e.name() == "Container Image")
        .expect("could not find explicit internal provider");

    // Create a component for both the source and the destination
    let oysters_component = Component::new(
        ctx,
        "oysters in my pocket",
        docker_image_schema_variant_id,
        None,
    )
    .await
    .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    // Create a second component for a second source
    let lunch_component = Component::new(
        ctx,
        "were saving for lunch",
        docker_image_schema_variant_id,
        None,
    )
    .await
    .expect("could not create component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component 2 creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let royel_component = Component::new(ctx, "royel otis", butane_schema_variant_id, None)
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
        external_provider.id(),
        royel_component.id(),
        explicit_internal_provider.id(),
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
        external_provider.id(),
        royel_component.id(),
        explicit_internal_provider.id(),
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
        .iter()
        .next()
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
