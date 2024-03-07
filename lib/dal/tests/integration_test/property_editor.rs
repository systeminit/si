use dal::property_editor::schema::PropertyEditorSchema;
use dal::property_editor::values::PropertyEditorValues;
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::test;

#[test]
async fn assemble(ctx: &DalContext) {
    // List all schemas in the workspace. Pick the first one alphabetically.
    let mut schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    let schema = schemas.pop().expect("schemas are empty");

    // Pick a schema variant.
    let mut schema_variants = SchemaVariant::list_for_schema(ctx, schema.id())
        .await
        .expect("could not list schema variants for schema");
    let schema_variant = schema_variants.pop().expect("schemas are empty");
    let schema_variant_id = schema_variant.id();

    // Create a component and set geometry.
    let name = "steam deck";
    let component = Component::new(ctx, name, schema_variant_id)
        .await
        .expect("could not create component");

    // Assemble both property editor blobs.
    let property_editor_schema = PropertyEditorSchema::assemble(ctx, schema_variant_id)
        .await
        .expect("could not assemble property editor schema");
    let property_editor_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("could not assemble property editor schema");
    dbg!(property_editor_schema, property_editor_values);
}
