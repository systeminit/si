use dal::func::FuncAssociations;
use dal::schema::variant::leaves::LeafInputLocation;
use dal::{DalContext, Func, Schema};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn for_qualification(ctx: &mut DalContext) {
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("could not perform find by name")
        .expect("no schema found");
    let schema_variant_id = schema
        .get_default_schema_variant(ctx)
        .await
        .expect("could not perform get default schema variant")
        .expect("default schema variant not found");

    let func_id = Func::find_by_name(ctx, "test:qualificationDummySecretStringIsTodd")
        .await
        .expect("could not perform find func by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");

    assert_eq!(
        FuncAssociations::Qualification {
            schema_variant_ids: vec![schema_variant_id],
            component_ids: vec![],
            inputs: vec![LeafInputLocation::Secrets]
        }, // expected
        associations.expect("no associations found") // actual
    );
}
