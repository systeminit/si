use dal::component::{DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::diagram::Diagram;
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn create_and_determine_lineage(ctx: &DalContext) {
    // List all schemas in the workspace. Pick the first one alphabetically.
    let mut schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    let schema = schemas.pop().expect("schemas are empty");

    // Ensure we can get it by id.
    let found_schema = Schema::get_by_id(&ctx, schema.id())
        .await
        .expect("could not get schema by id");
    assert_eq!(
        schema.id(),       // expected
        found_schema.id()  // actual
    );

    // Pick a schema variant.
    let mut schema_variants = SchemaVariant::list_for_schema(ctx, found_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let schema_variant = schema_variants.pop().expect("schemas are empty");
    let schema_variant_id = schema_variant.id();

    // Create a component and set geometry.
    let name = "fsu not top four";
    let component = Component::new(&ctx, name, schema_variant_id, None)
        .await
        .expect("could not create component");
    let component = component
        .set_geometry(
            &ctx,
            "1",
            "-1",
            Some(DEFAULT_COMPONENT_WIDTH),
            Some(DEFAULT_COMPONENT_HEIGHT),
        )
        .await
        .expect("could not set geometry");

    // Determine the schema variant from the component. Ensure it is the same as before.
    let post_creation_schema_variant = Component::schema_variant(ctx, component.id())
        .await
        .expect("could not get schema variant for component");
    assert_eq!(
        schema_variant_id,                 // expected
        post_creation_schema_variant.id()  // actual
    );

    // Determine the schema from the schema variant. Ensure it is the same as before.
    let post_creation_schema = SchemaVariant::schema(ctx, post_creation_schema_variant.id())
        .await
        .expect("could not get schema for schema variant");
    assert_eq!(
        schema.id(),               // expected
        post_creation_schema.id()  // actual
    );

    // Assemble the diagram just to make sure it works.
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    dbg!(diagram);
}
