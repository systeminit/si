use dal::func::binding::attribute::AttributeBinding;
use dal::func::binding::{EventualParent, FuncBinding};
use dal::{DalContext, Func, SchemaVariant};
use dal_test::helpers::{create_unlocked_variant_copy_for_schema_name, ChangeSetTestHelpers};
use dal_test::test;

#[test]
async fn detach_attribute_func(ctx: &mut DalContext) {
    let schema_variant_id = create_unlocked_variant_copy_for_schema_name(ctx, "starfield")
        .await
        .expect("could not create unlocked copy");
    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    let total_funcs = funcs.len();

    // Detach one action func to the schema variant and commit.
    let func_id = Func::find_id_by_name(ctx, "test:falloutEntriesToGalaxies")
        .await
        .expect("unable to find the func")
        .expect("no func found");

    let bindings = FuncBinding::get_attribute_bindings_for_func_id(ctx, func_id)
        .await
        .expect("could not get bindings");

    let prototype: AttributeBinding = bindings
        .into_iter()
        .find(|p| p.eventual_parent == EventualParent::SchemaVariant(schema_variant_id))
        .expect("has a prototype for this schema variant");

    AttributeBinding::reset_attribute_binding(ctx, prototype.attribute_prototype_id)
        .await
        .expect("could not reset attribute prototype");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Now, let's list all funcs and see what's left.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    assert_eq!(
        total_funcs - 1, // expected
        funcs.len()      // actual
    );
    assert!(!funcs.iter().any(|summary| summary.id == func_id));
}
