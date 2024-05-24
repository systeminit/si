use dal::diagram::Diagram;
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{DalContext, Prop};
use dal_test::helpers::create_component_for_schema_name;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn upgrade_component(ctx: &mut DalContext) {
    // Let's create a new asset
    let asset_name = "paulsTestAsset".to_string();
    let display_name = None;
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant_zero = VariantAuthoringClient::create_variant(
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

    let my_asset_schema = variant_zero
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = my_asset_schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");
    assert!(default_schema_variant.is_some());
    assert_eq!(default_schema_variant, Some(variant_zero.id()));

    // Now let's update the variant
    let first_code_update = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()\n  return new AssetBuilder().addProp(myProp).build()\n}".to_string();
    let updated_variant_id = VariantAuthoringClient::update_variant(
        ctx,
        variant_zero.id(),
        my_asset_schema.name.clone(),
        variant_zero.display_name(),
        variant_zero.category().to_string(),
        variant_zero
            .get_color(ctx)
            .await
            .expect("Unable to get color of variant"),
        variant_zero.link(),
        first_code_update,
        variant_zero.description(),
        variant_zero.component_type(),
    )
    .await
    .expect("unable to update asset");

    // We should still see that the schema variant we updated is the same as we have no components on the graph
    assert_eq!(variant_zero.id(), updated_variant_id);

    // Add a component to the diagram
    let initial_component =
        create_component_for_schema_name(ctx, my_asset_schema.name.clone(), "demo component").await;
    let initial_diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, initial_diagram.components.len());

    // Let's ensure that our prop is visible in the component
    Prop::find_prop_id_by_path(
        ctx,
        updated_variant_id,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await
    .expect("able to find testProp prop");

    let initial_component_schema_variant = initial_component
        .schema_variant(ctx)
        .await
        .expect("Unable to get schema variant for component");
    assert_eq!(initial_component_schema_variant.id(), variant_zero.id());

    // Now let's update the asset a second time!
    let second_code_update = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build();\n const anotherProp = new PropBuilder().setName(\"anotherProp\").setKind(\"integer\").build();\n  return new AssetBuilder().addProp(myProp).addProp(anotherProp).build()\n}".to_string();
    let variant_one = VariantAuthoringClient::update_variant(
        ctx,
        variant_zero.id(),
        my_asset_schema.name.clone(),
        variant_zero.display_name(),
        variant_zero.category().to_string(),
        variant_zero
            .get_color(ctx)
            .await
            .expect("Unable to get color of variant"),
        variant_zero.link(),
        second_code_update,
        variant_zero.description(),
        variant_zero.component_type(),
    )
    .await
    .expect("unable to update asset");

    // We should have a NEW schema variant id as there is a component on the graph
    assert_ne!(variant_one, variant_zero.id());

    // Check that the components exist for the new variant
    Prop::find_prop_id_by_path(
        ctx,
        variant_one,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await
    .expect("able to find testProp prop for variant one");

    Prop::find_prop_id_by_path(
        ctx,
        variant_one,
        &PropPath::new(["root", "domain", "anotherProp"]),
    )
    .await
    .expect("able to find anotherProp prop for variant one");

    let one_component_on_the_graph = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(one_component_on_the_graph.components.len(), 1);
    let my_upgradable_component = one_component_on_the_graph
        .components
        .first()
        .expect("unable to get the upgradable component on the graph");
    assert!(my_upgradable_component.can_be_upgraded);

    let my_upgraded_comp = initial_component
        .upgrade_to_new_variant(ctx, variant_one)
        .await
        .expect("unable to upgrade the component");

    let upgraded_graph = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    let upgraded_component = upgraded_graph
        .components
        .first()
        .expect("unable to get the upgraded component on the graph");
    assert_eq!(
        upgraded_component.can_be_upgraded, false,
        "the old asset should not be on the graph anymore, and the current one should be upgraded"
    );

    let upgraded_component_schema_variant = my_upgraded_comp
        .schema_variant(ctx)
        .await
        .expect("unable to get schema variant for my upgraded component");

    assert_eq!(variant_one, upgraded_component_schema_variant.id());
}
