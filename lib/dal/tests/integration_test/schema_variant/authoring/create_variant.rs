use dal::{
    ChangeSet,
    DalContext,
    Func,
    FuncBackendResponseType,
    func::FuncKind,
    schema::variant::authoring::VariantAuthoringClient,
};
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
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
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
    assert_eq!(new_schema.name(), &asset_name);

    // we initialize display_name to match the schema name
    assert_eq!(variant.display_name(), &asset_name);
    assert_eq!(
        variant.get_color(ctx).await.expect("unable to get color"),
        color.clone()
    );
    assert!(variant.asset_func_id().is_some());

    let func = Func::get_by_id(
        ctx,
        variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get asset authoring func");

    let scaffold_func_name = format!("{asset_name}Scaffold_");
    assert!(func.name.contains(&scaffold_func_name));
    assert_eq!(func.kind, FuncKind::SchemaVariantDefinition);
    assert_eq!(
        func.backend_response_type,
        FuncBackendResponseType::SchemaVariantDefinition
    );
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(
        Some(
            "function main() {\n  const asset = new AssetBuilder();\n  return asset.build();\n}"
                .to_string()
        ),
        func.code_plaintext().expect("Unable to get code plaintext")
    );
}
