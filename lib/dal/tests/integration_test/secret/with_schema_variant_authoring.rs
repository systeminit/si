use dal::func::binding::leaf::LeafBinding;
use dal::func::binding::{EventualParent, FuncBinding};
use dal::func::view::FuncView;
use dal::func::FuncAssociations;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::schema::variant::leaves::{LeafInputLocation, LeafKind};
use dal::{DalContext, Func};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn existing_code_gen_func_using_secrets_for_new_schema_variant(ctx: &mut DalContext) {
    // Create a new schema variant and commit.
    let schema_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx, "ergo sum", None, None, "bungie", "#00b0b0",
    )
    .await
    .expect("could not create new asset");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Get the current func view of a func used by another schema variant. We want to use and
    // validate its associations.
    let func_id = Func::find_id_by_name(ctx, "test:generateStringCode")
        .await
        .expect("could not perform find by name")
        .expect("func not found");
    let func = Func::get_by_id_or_error(ctx, func_id)
        .await
        .expect("could not get func by id");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not get func view");
    dbg!(&func_view);
    let (_, _, mut inputs) = match func_view.associations.expect("no associations found") {
        FuncAssociations::CodeGeneration {
            schema_variant_ids,
            component_ids,
            inputs,
        } => (schema_variant_ids, component_ids, inputs),
        associations => panic!("unexpected associations kind: {associations:?}"),
    };

    let bindings = LeafBinding::create_leaf_func_binding(
        ctx,
        func_id,
        EventualParent::SchemaVariant(schema_variant.id()),
        LeafKind::CodeGeneration,
        inputs.as_slice(),
    )
    .await
    .expect("could not add leaf func");

    let FuncBinding::CodeGeneration(bindings) = bindings
        .iter()
        .find(|func_binding| {
            if let FuncBinding::CodeGeneration(binding) = func_binding {
                binding.eventual_parent == EventualParent::SchemaVariant(schema_variant.id())
            } else {
                false
            }
        })
        .expect("could not find binding for new variant")
    else {
        panic!("could not add leaf node")
    };

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Add the secrets input and commit.
    inputs.push(LeafInputLocation::Secrets);
    // can't update leaf bindings without unlocking all attached things!! because this changes the func args AND the prototype args :facepalm:
    LeafBinding::update_leaf_func_binding(ctx, bindings.attribute_prototype_id, inputs.as_slice())
        .await
        .expect("could not update leaf binding");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Update the schema variant and ensure that it succeeds.
    let schema = schema_variant
        .schema(ctx)
        .await
        .expect("could not get schema");

    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant.id(),
        schema.name,
        schema_variant.display_name(),
        schema_variant.category(),
        Some("let's update the description".to_string()),
        schema_variant.link(),
        schema_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        schema_variant.component_type(),
        Some("function main() { return new AssetBuilder().build() }"),
    )
    .await
    .expect("save variant contents");

    let _updated_schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant.id())
            .await
            .expect("could not upgrade variant");
}
