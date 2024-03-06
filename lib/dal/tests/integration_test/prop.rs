use dal::{prop::PropPath, DalContext, Prop, Schema, SchemaVariant};
use dal_test::test;

#[test]
async fn prop_path(ctx: &DalContext) {
    let starfield_schema = Schema::list(ctx)
        .await
        .expect("list schemas")
        .iter()
        .find(|schema| schema.name() == "starfield")
        .expect("starfield does not exist")
        .to_owned();

    let variant = SchemaVariant::list_for_schema(ctx, starfield_schema.id())
        .await
        .expect("get schema variants")
        .pop()
        .expect("get default variant");

    let name_path = PropPath::new(["root", "si", "name"]);
    let name_id = Prop::find_prop_id_by_path(ctx, variant.id(), &name_path)
        .await
        .expect("get name prop id");
    let fetched_name_path = Prop::path_by_id(ctx, name_id)
        .await
        .expect("get prop path by id");

    assert_eq!(name_path, fetched_name_path);
}
