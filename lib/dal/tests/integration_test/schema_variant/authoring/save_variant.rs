use dal::{
    ChangeSet,
    ComponentType,
    DalContext,
    Func,
    FuncBackendResponseType,
    Schema,
    SchemaVariant,
    func::FuncKind,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    helpers::ChangeSetTestHelpers,
    test,
};

#[test]
async fn save_variant(ctx: &mut DalContext) {
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let asset_name = "paulsTestAsset".to_string();
    let display_name = asset_name.clone();
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

    assert_eq!(variant.category(), &category);
    assert_eq!(new_schema.name(), &asset_name);
    // we update the display_name to match the schema name if display_name is none
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

    let updated_func_name = "Paul's Updated Func Name".to_string();
    let updated_description = Some("My Func Has A Description".to_string());
    let updated_func_content = "function main() {\n const myProp = new PropBuilder().setName(\"PaulsProp\").setKind(\"string\").build();\n  return new AssetBuilder().addProp(myProp).build()\n}"
            .to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        variant.id(),
        updated_func_name,
        &display_name,
        "category.clone()",
        updated_description.clone(),
        variant.link(),
        &color,
        variant.component_type(),
        Some(updated_func_content.clone()),
    )
    .await
    .expect("Unable to save the func");

    let func = Func::get_by_id(
        ctx,
        variant
            .asset_func_id()
            .expect("unable to get asset func id from variant"),
    )
    .await
    .expect("unable to get asset authoring func");

    assert_eq!(func.kind, FuncKind::SchemaVariantDefinition);
    assert_eq!(
        func.backend_response_type,
        FuncBackendResponseType::SchemaVariantDefinition
    );
    assert_eq!(Some("main".to_string()), func.handler);
    assert_eq!(updated_description, func.description);
    assert_eq!(
        Some(updated_func_content),
        func.code_plaintext().expect("Unable to get code plaintext")
    );
}

#[test]
async fn unlock_and_save_variant(ctx: &mut DalContext) {
    let new_change_set = ChangeSet::fork_head(ctx, "new change set")
        .await
        .expect("could not create new change set");
    ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
        .await
        .expect("could not update visibility");

    let schema = Schema::get_by_name(ctx, "dummy-secret")
        .await
        .expect("schema not found");
    let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("Unable to find the default schema variant id");
    let existing_variant = SchemaVariant::get_by_id(ctx, default_schema_variant)
        .await
        .expect("unable to lookup the default schema variant");

    let unlocked_schema_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, existing_variant.id())
            .await
            .expect("could not create unlocked copy");
    // new variant is unlocked
    assert!(!unlocked_schema_variant.is_locked());
    // data matches
    assert_eq!(
        unlocked_schema_variant.category(),
        existing_variant.category()
    );
    assert_eq!(
        unlocked_schema_variant.display_name(),
        existing_variant.display_name()
    );
    assert_eq!(
        unlocked_schema_variant.description(),
        existing_variant.description()
    );
    assert_eq!(unlocked_schema_variant.color(), existing_variant.color());
    assert_eq!(
        unlocked_schema_variant.component_type(),
        existing_variant.component_type()
    );

    assert_eq!(
        unlocked_schema_variant
            .get_asset_func(ctx)
            .await
            .expect("could not get asset func")
            .code_base64,
        existing_variant
            .get_asset_func(ctx)
            .await
            .expect("could not get asset func")
            .code_base64
    );

    // unlocked variant has a newer version
    assert!(
        *unlocked_schema_variant.version().to_string() > *existing_variant.version().to_string()
    );

    // now let's change some stuff

    let new_description = Some("fancy new description".to_string());
    let new_display_name = "fancy display name too".to_string();
    let new_category = "Fancy";
    let new_color = "#191919";
    let new_link = Some("https://fancy.ai".to_string());

    VariantAuthoringClient::save_variant_content(
        ctx,
        unlocked_schema_variant.id,
        unlocked_schema_variant
            .schema(ctx)
            .await
            .expect("could not get schema")
            .name,
        new_display_name.clone(),
        new_category,
        new_description.clone(),
        new_link.clone(),
        new_color,
        ComponentType::ConfigurationFrameDown,
        unlocked_schema_variant
            .get_asset_func(ctx)
            .await
            .expect("could not get asset func")
            .code_plaintext()
            .expect("got the code"),
    )
    .await
    .expect("could not save variant content");

    // commit changes
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    // apply change set and check head
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply to head");

    // check head and make sure everything looks good
    let schema = Schema::get_by_name(ctx, "dummy-secret")
        .await
        .expect("schema not found");
    let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("Unable to find the default schema variant id");
    let new_merged_variant = SchemaVariant::get_by_id(ctx, default_schema_variant)
        .await
        .expect("unable to lookup the default schema variant");
    assert!(new_merged_variant.is_locked());

    // schema variant ids should match
    assert_eq!(new_merged_variant.id(), unlocked_schema_variant.id());
    // new variant is now locked
    assert!(new_merged_variant.is_locked());

    // data matches

    assert_eq!(new_merged_variant.category(), new_category);
    assert_eq!(new_merged_variant.display_name(), new_display_name);
    assert_eq!(new_merged_variant.description(), new_description);
    assert_eq!(new_merged_variant.color(), new_color);
    assert_eq!(
        unlocked_schema_variant
            .get_asset_func(ctx)
            .await
            .expect("could not get asset func")
            .code_base64,
        new_merged_variant
            .get_asset_func(ctx)
            .await
            .expect("could not get asset func")
            .code_base64
    );

    assert_eq!(
        new_merged_variant.component_type(),
        ComponentType::ConfigurationFrameDown
    );

    // merged variant has a newer version
    assert!(new_merged_variant.version() > unlocked_schema_variant.version());

    // create a new changeset, and unlock another copy

    ChangeSetTestHelpers::fork_from_head_change_set_with_name(ctx, "change set 2")
        .await
        .expect("could not fork head");

    let second_editing_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, new_merged_variant.id())
            .await
            .expect("could not unlock schema variant");

    // unlocked variant is unlocked
    assert!(!second_editing_variant.is_locked());

    // data matches
    assert_eq!(
        second_editing_variant.component_type(),
        ComponentType::ConfigurationFrameDown
    );
    assert_eq!(second_editing_variant.description(), new_description);
    assert_eq!(second_editing_variant.category(), new_category);
    assert_eq!(second_editing_variant.display_name(), new_display_name);
    assert_eq!(second_editing_variant.color(), new_color);

    // version of newly unlocked variant is newer
    assert!(second_editing_variant.version() > new_merged_variant.version());
}
