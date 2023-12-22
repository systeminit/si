use dal::{Component, DalContext, ExternalProvider, InternalProvider, Schema, SchemaVariant};
use dal_test::test;

#[test]
async fn connect_components_simple(ctx: &DalContext) {
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
    let butane_explicit_internal_providers =
        InternalProvider::list_explicit(ctx, butane_schema_variant_id)
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
    .expect("could not create component");
    let royel_component = Component::new(ctx, "royel otis", butane_schema_variant_id, None)
        .expect("could not create component");

    // Connect the components!
    Component::connect(
        ctx,
        oysters_component.id(),
        external_provider.id(),
        royel_component.id(),
        explicit_internal_provider.id(),
    )
    .expect("could not connect components");
}
