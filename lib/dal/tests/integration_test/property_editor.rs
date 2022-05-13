use crate::dal::test;
use dal::{property_editor::PropertyEditorSchema, DalContext, Schema, StandardModel};

#[test]
async fn for_schema_variant(ctx: &DalContext<'_, '_>) {
    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let schema_variant_id = schema
        .default_schema_variant_id()
        .expect("missing default schema variant id");
    let property_editor_schema = PropertyEditorSchema::for_schema_variant(ctx, *schema_variant_id)
        .await
        .expect("cannot create property editor schema from schema variant");
    dbg!(property_editor_schema);
    // NOTE: Some day, this test should.. test something. For now, though - we'll do it live.
}
