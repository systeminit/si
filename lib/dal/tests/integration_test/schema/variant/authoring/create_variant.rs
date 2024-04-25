use dal::func::FuncKind;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, DalContext, Func, FuncBackendResponseType};
use dal_test::test;

#[test]
async fn create_variant(ctx: &mut DalContext) {
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let asset_name = "paulsTestAsset".to_string();
    let display_name = None;
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant = VariantAuthoringClient::create_variant(
        ctx,
        asset_name.clone(),
        display_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await
    .expect("Unable to create new asset");

    let new_schema = variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    assert_eq!(variant.category(), category.clone());
    assert_eq!(new_schema.name(), asset_name.clone());
    assert_eq!(variant.display_name(), display_name.clone());
    assert_eq!(
        variant.get_color(ctx).await.expect("unable to get color"),
        color.clone()
    );
    assert!(variant.asset_func_id().is_some());

    let maybe_func = Func::get_by_id(
        ctx,
        variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get asset authoring func");

    assert!(maybe_func.is_some());

    let func = maybe_func.unwrap();

    let scaffold_func_name = format!("{}Scaffold_", asset_name);
    assert!(func.name.contains(&scaffold_func_name));
    assert_eq!(func.kind, FuncKind::SchemaVariantDefinition);
    assert_eq!(
        func.backend_response_type,
        FuncBackendResponseType::SchemaVariantDefinition
    );
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some("function main() {\n  return new AssetBuilder().build()\n}".to_string()),
        func.code_plaintext().expect("Unable to get code plaintext")
    );
}
