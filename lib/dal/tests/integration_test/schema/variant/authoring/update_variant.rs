use dal::diagram::Diagram;
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{DalContext, Prop};
use dal_test::helpers::create_component_for_schema_name;
use dal_test::test;

#[test]
async fn update_variant(ctx: &mut DalContext) {
    // Let's create a new asset
    let asset_name = "paulsTestAsset".to_string();
    let display_name = None;
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let my_first_variant = VariantAuthoringClient::create_variant(
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

    let my_asset_schema = my_first_variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = my_asset_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");
    assert!(default_schema_variant.is_some());
    assert_eq!(default_schema_variant, Some(my_first_variant.id()));

    // Now let's update the variant
    let new_code = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()\n  return new AssetBuilder().addProp(myProp).build()\n}".to_string();
    let updated_sv_id = VariantAuthoringClient::update_variant(
        ctx,
        my_first_variant.id(),
        my_asset_schema.name.clone(),
        my_first_variant.display_name(),
        my_first_variant.category().to_string(),
        my_first_variant
            .get_color(ctx)
            .await
            .expect("Unable to get color of variant"),
        my_first_variant.link(),
        new_code,
        my_first_variant.description(),
        my_first_variant.component_type(),
    )
    .await
    .expect("unable to update asset");

    assert_eq!(my_first_variant.id(), updated_sv_id);

    // Add a component to the diagram
    create_component_for_schema_name(ctx, my_asset_schema.name.clone(), "demo component").await;
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    pretty_assertions_sorted::assert_eq!(1, diagram.components.len());

    // Let's ensure that our prop is visible in the component
    Prop::find_prop_id_by_path(
        ctx,
        updated_sv_id,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await
    .expect("able to find testProp prop");

    // Now let's update the asset a second time!
    let new_code = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build();\n const anotherProp = new PropBuilder().setName(\"anotherProp\").setKind(\"integer\").build();\n  return new AssetBuilder().addProp(myProp).addProp(anotherProp).build()\n}".to_string();
    let second_updated_sv_id = VariantAuthoringClient::update_variant(
        ctx,
        my_first_variant.id(),
        my_asset_schema.name.clone(),
        my_first_variant.display_name(),
        my_first_variant.category().to_string(),
        my_first_variant
            .get_color(ctx)
            .await
            .expect("Unable to get color of variant"),
        my_first_variant.link(),
        new_code,
        my_first_variant.description(),
        my_first_variant.component_type(),
    )
    .await
    .expect("unable to update asset");

    // We should have a NEW schema variant id as there is a component on the graph
    assert_ne!(second_updated_sv_id, my_first_variant.id());

    // Let's ensure that our latest prop is visible in the component
    create_component_for_schema_name(ctx, my_asset_schema.name.clone(), "demo component 2").await;
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    pretty_assertions_sorted::assert_eq!(2, diagram.components.len());
    Prop::find_prop_id_by_path(
        ctx,
        second_updated_sv_id,
        &PropPath::new(["root", "domain", "anotherProp"]),
    )
    .await
    .expect("able to find anotherProp prop");

    // Let's check that the default schema variant has been updated
    let updated_default_schema_variant = my_asset_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");
    assert!(updated_default_schema_variant.is_some());
    assert_eq!(updated_default_schema_variant, Some(second_updated_sv_id));
}
