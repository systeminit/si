use dal::func::authoring::FuncAuthoringClient;
use dal::func::view::FuncView;
use dal::func::FuncAssociations;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::schema::variant::leaves::LeafInputLocation;
use dal::{DalContext, Func};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn existing_code_gen_func_using_secrets_for_new_schema_variant(ctx: &mut DalContext) {
    // Create a new schema variant and commit.
    let schema_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx, "ergo sum", None, None, None, "bungie", "#00b0b0",
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
    let (mut schema_variant_ids, component_ids, mut inputs) =
        match func_view.associations.expect("no associations found") {
            FuncAssociations::CodeGeneration {
                schema_variant_ids,
                component_ids,
                inputs,
            } => (schema_variant_ids, component_ids, inputs),
            associations => panic!("unexpected associations kind: {associations:?}"),
        };

    // Add the schema variant and commit.
    schema_variant_ids.push(schema_variant.id());
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name.clone(),
        func_view.name.clone(),
        func_view.description.clone(),
        func_view.code.clone(),
        Some(FuncAssociations::CodeGeneration {
            schema_variant_ids: schema_variant_ids.clone(),
            component_ids: component_ids.clone(),
            inputs: inputs.clone(),
        }),
    )
    .await
    .expect("could not save func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Add the secrets input and commit.
    inputs.push(LeafInputLocation::Secrets);
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        func_view.code,
        Some(FuncAssociations::CodeGeneration {
            schema_variant_ids,
            component_ids,
            inputs,
        }),
    )
    .await
    .expect("could not save func");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Update the schema variant and ensure that it succeeds.
    let schema = schema_variant
        .schema(ctx)
        .await
        .expect("could not get schema");
    let _updated_schema_variant_id = VariantAuthoringClient::update_variant(
        ctx,
        schema_variant.id(),
        schema.name,
        schema_variant.display_name(),
        schema_variant.category().to_string(),
        schema_variant
            .get_color(ctx)
            .await
            .expect("could not get color"),
        schema_variant.link(),
        "function main() { return new AssetBuilder().build() }",
        Some("let's update the description".to_string()),
        schema_variant.component_type(),
    )
    .await
    .expect("could not upgrade variant");
}
