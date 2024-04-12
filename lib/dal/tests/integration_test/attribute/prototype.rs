use dal::prop::PropPath;
use dal::{AttributePrototype, DalContext, Func, Prop, Schema};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn find_for_prop(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("unable to get schema")
        .expect("schema not found");
    let schema_variant_id = schema
        .get_default_schema_variant(ctx)
        .await
        .expect("unable to get schema variant")
        .expect("schema variant not found");
    let func_name = "test:generateCode";

    // Find the sole attribute prototype via its func. Ensure that we find one and only one.
    let func_id = Func::find_by_name(ctx, func_name)
        .await
        .expect("could not perform find by name")
        .expect("func not found");
    let mut attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, func_id)
        .await
        .expect("could not list attribute prototype ids for func id");
    let attribute_prototype_id = attribute_prototype_ids
        .pop()
        .expect("empty attribute prototype ids");
    assert!(attribute_prototype_ids.is_empty());

    // Ensure that find for prop works with a key.
    let prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "code", "codeItem"]),
    )
    .await
    .expect("could not find prop by path");
    let found_attribute_prototype_id =
        AttributePrototype::find_for_prop(ctx, prop_id, &Some(func_name.to_owned()))
            .await
            .expect("could not perform find for prop")
            .expect("no attribute prototype found");
    assert_eq!(
        attribute_prototype_id,       // expected
        found_attribute_prototype_id  // actual
    );
}
