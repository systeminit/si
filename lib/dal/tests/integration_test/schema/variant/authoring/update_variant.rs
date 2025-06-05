use std::collections::HashSet;

use dal::{
    AttributePrototype,
    AttributePrototypeId,
    Component,
    ComponentType,
    DalContext,
    Func,
    Prop,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    attribute::prototype::AttributePrototypeEventualParent,
    diagram::Diagram,
    func::{
        authoring::FuncAuthoringClient,
        binding::{
            EventualParent,
            FuncBinding,
        },
    },
    prop::PropPath,
    qualification::QualificationSubCheckStatus,
    schema::variant::{
        authoring::VariantAuthoringClient,
        leaves::{
            LeafInputLocation,
            LeafKind,
        },
    },
};
use dal_test::{
    expected::commit_and_update_snapshot_to_visibility,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        create_component_for_unlocked_schema_name_on_default_view,
    },
    test,
};

#[test]
async fn update_variant(ctx: &mut DalContext) {
    // Let's create a new asset
    let asset_name = "paulsTestAsset".to_string();
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let first_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        description.clone(),
        link.clone(),
        category.clone(),
        color.clone(),
    )
    .await
    .expect("Unable to create new asset");
    let schema = first_variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get the default schema variant id");
    assert_eq!(default_schema_variant, first_variant.id());

    // Now let's update the variant
    let new_code = "function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()\n  return new AssetBuilder().addProp(myProp).build()\n}".to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        first_variant.id(),
        &schema.name,
        first_variant.display_name(),
        first_variant.category(),
        first_variant.description(),
        first_variant.link(),
        first_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        first_variant.component_type(),
        Some(new_code),
    )
    .await
    .expect("save variant contents");

    let updated_sv_id = VariantAuthoringClient::regenerate_variant(ctx, first_variant.id())
        .await
        .expect("unable to update asset");

    assert_eq!(first_variant.id(), updated_sv_id);

    // Add a component to the diagram
    create_component_for_default_schema_name_in_default_view(
        ctx,
        schema.name.clone(),
        "demo component",
    )
    .await
    .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    pretty_assertions_sorted::assert_eq!(1, diagram.components.len());

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

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

    VariantAuthoringClient::save_variant_content(
        ctx,
        first_variant.id(),
        &schema.name,
        first_variant.display_name(),
        first_variant.category(),
        first_variant.description(),
        first_variant.link(),
        first_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        first_variant.component_type(),
        Some(new_code),
    )
    .await
    .expect("save variant contents");

    let second_updated_sv_id = VariantAuthoringClient::regenerate_variant(ctx, first_variant.id())
        .await
        .expect("regenerate asset");

    // We should have a NEW schema variant id as there is a component on the graph
    assert_ne!(second_updated_sv_id, first_variant.id());

    // Let's ensure that our latest prop is visible in the component
    create_component_for_default_schema_name_in_default_view(
        ctx,
        schema.name.clone(),
        "demo component 2",
    )
    .await
    .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
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
    let updated_default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get the default schema variant id");
    assert_eq!(updated_default_schema_variant, second_updated_sv_id);
}

#[test]
async fn update_variant_with_new_metadata(ctx: &mut DalContext) {
    // Let's create a new asset with some initial metadata
    let asset_name = "paulsTestAsset".to_string();
    let first_description = Some("first description".to_string());
    let first_link = Some("https://firstlink.com/".to_string());
    let first_category = "Integration Tests".to_string();
    let first_color = "#00b0b0".to_string();

    let first_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
        first_description.clone(),
        first_link.clone(),
        first_category.clone(),
        first_color.clone(),
    )
    .await
    .expect("Unable to create new asset");

    let first_sv_id = first_variant.id;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    let asset_func = Func::get_by_id(
        ctx,
        first_variant
            .asset_func_id()
            .expect("unable to get asset func id"),
    )
    .await
    .expect("could not get asset func");

    let asset_code = asset_func
        .code_plaintext()
        .expect("could not get code plaintext")
        .expect("is not an option");

    let schema = first_variant
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    let default_schema_variant = Schema::default_variant_id(ctx, schema.id())
        .await
        .expect("unable to get the default schema variant id");

    assert_eq!(default_schema_variant, first_variant.id());

    let first_variant = SchemaVariant::get_by_id(ctx, first_sv_id)
        .await
        .expect("could not get schema variant");

    // ensure everything got set correctly the first time
    assert_eq!(first_variant.id(), first_sv_id);
    assert_eq!(first_variant.category(), first_category);
    assert_eq!(first_variant.description(), first_description);
    assert_eq!(first_variant.display_name(), asset_name);
    assert_eq!(first_variant.color(), first_color);
    assert_eq!(first_variant.link(), first_link);

    // now let's update the metadata
    let second_asset_name = "britsTestAsset".to_string();
    let second_display_name = "britsTestAssetDisplay".to_string();
    let second_description = Some("second description".to_string());
    let second_link = Some("https://secondlink.com/".to_string());
    let second_category = "Integration Tests 2".to_string();
    let second_color = "#00b0b1".to_string();
    let second_component_type = ComponentType::ConfigurationFrameDown;

    // save the metadata and commit
    VariantAuthoringClient::save_variant_content(
        ctx,
        first_sv_id,
        &second_asset_name.clone(),
        second_display_name.clone(),
        second_category.clone(),
        second_description.clone(),
        second_link.clone(),
        second_color.clone(),
        second_component_type,
        Some(asset_code.clone()),
    )
    .await
    .expect("save variant contents");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // ensure metadata is correct for the schema variant
    let second_variant = SchemaVariant::get_by_id(ctx, first_sv_id)
        .await
        .expect("could not get schema variant");
    let schema = second_variant
        .schema(ctx)
        .await
        .expect("could not get schema");

    assert_eq!(schema.name, second_asset_name);
    assert_eq!(second_variant.id(), first_sv_id);
    assert_eq!(second_variant.category(), second_category);
    assert_eq!(second_variant.description(), second_description);
    assert_eq!(second_variant.display_name(), second_display_name);
    assert_eq!(second_variant.color(), second_color);
    assert_eq!(second_variant.link(), second_link);
    assert_eq!(second_variant.component_type(), second_component_type);

    // now let's create a component with the new variant
    let component =
        create_component_for_unlocked_schema_name_on_default_view(ctx, schema.name(), "component")
            .await
            .expect("could not create compoennt");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // let's make sure the component has the correct default values
    let component_name = component.name(ctx).await.expect("could not get name");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");

    assert_eq!(1, diagram.components.len());

    let diagram_component = diagram.components.first().expect("has one component");
    let first_component_id = diagram_component.id;
    assert_eq!(diagram_component.color, second_color);
    assert_eq!(
        diagram_component.component_type,
        second_component_type.to_string()
    );
    assert_eq!(diagram_component.schema_category, second_category);
    assert_eq!(diagram_component.schema_variant_id, first_sv_id);
    assert_eq!(diagram_component.schema_name, second_asset_name);
    assert_eq!(diagram_component.display_name, component_name);

    // now let's regenerate
    let updated_sv_id_after_regen = VariantAuthoringClient::regenerate_variant(ctx, first_sv_id)
        .await
        .expect("unable to update asset");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // get updated schema variant after regenerating
    let updated_variant_after_regen = SchemaVariant::get_by_id(ctx, updated_sv_id_after_regen)
        .await
        .expect("could not get schema variant");

    // let's ensure schema variant is as it was before
    assert_ne!(updated_variant_after_regen.id(), first_sv_id);
    assert_eq!(updated_variant_after_regen.category(), second_category);
    assert_eq!(
        updated_variant_after_regen.description(),
        second_description
    );
    assert_eq!(
        updated_variant_after_regen.display_name(),
        second_display_name
    );
    assert_eq!(updated_variant_after_regen.color(), second_color);
    assert_eq!(updated_variant_after_regen.link(), second_link);

    // component is as it was but with new schema variant id
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, diagram.components.len());

    let diagram_component = diagram.components.first().expect("has one component");
    assert_eq!(diagram_component.color, second_color);
    assert_eq!(
        diagram_component.component_type,
        second_component_type.to_string()
    );
    assert_eq!(diagram_component.schema_category, second_category);
    assert_eq!(
        diagram_component.schema_variant_id,
        updated_variant_after_regen.id()
    );
    assert_eq!(diagram_component.schema_name, second_asset_name);
    assert_eq!(diagram_component.display_name, component_name);

    // now let's update metadata again
    let third_display_name = "britsTestAsset2".to_string();
    let third_description = Some("third description".to_string());
    let third_link = Some("https://thirdlink.com/".to_string());
    let third_category = "Integration Tests 3".to_string();
    let third_color = "#00b1b1".to_string();
    let third_component_type = ComponentType::ConfigurationFrameUp;
    VariantAuthoringClient::save_variant_content(
        ctx,
        updated_sv_id_after_regen,
        &schema.name,
        third_display_name.clone(),
        third_category.clone(),
        third_description.clone(),
        third_link.clone(),
        third_color.clone(),
        third_component_type,
        Some(asset_code.clone()),
    )
    .await
    .expect("save variant contents");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // make sure everything saved as expected
    let updated_sv_after_metadata_change = SchemaVariant::get_by_id(ctx, updated_sv_id_after_regen)
        .await
        .expect("could not get schema variant");
    assert_eq!(
        updated_sv_after_metadata_change.id(),
        updated_sv_id_after_regen
    );
    assert_eq!(updated_sv_after_metadata_change.category(), third_category);
    assert_eq!(
        updated_sv_after_metadata_change.description(),
        third_description
    );
    assert_eq!(
        updated_sv_after_metadata_change.display_name(),
        third_display_name
    );
    assert_eq!(updated_sv_after_metadata_change.color(), third_color);
    assert_eq!(updated_sv_after_metadata_change.link(), third_link);
    assert_eq!(
        updated_sv_after_metadata_change.component_type(),
        third_component_type
    );

    // first component is not modified but reflects new schema variant specific metadata
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, diagram.components.len());

    let diagram_component = diagram.components.first().expect("has one component");
    assert_eq!(diagram_component.color, third_color);
    assert_eq!(
        diagram_component.component_type,
        second_component_type.to_string()
    );
    assert_eq!(diagram_component.schema_category, third_category);
    assert_eq!(
        diagram_component.schema_variant_id,
        updated_sv_after_metadata_change.id()
    );
    assert_eq!(diagram_component.schema_name, second_asset_name);
    assert_eq!(diagram_component.display_name, component_name);

    // now regen again, which should produce a new schema variant id as there is now a component
    let updated_sv_id_after_regen =
        VariantAuthoringClient::regenerate_variant(ctx, updated_sv_after_metadata_change.id)
            .await
            .expect("unable to update asset");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    assert_ne!(updated_sv_id_after_regen, first_sv_id);
    let updated_variant_after_regen = SchemaVariant::get_by_id(ctx, updated_sv_id_after_regen)
        .await
        .expect("could not get schema variant");

    // make sure the metadata matches
    assert_eq!(updated_variant_after_regen.category(), third_category);
    assert_eq!(updated_variant_after_regen.description(), third_description);
    assert_eq!(
        updated_variant_after_regen.display_name(),
        third_display_name
    );
    assert_eq!(updated_variant_after_regen.color(), third_color);
    assert_eq!(updated_variant_after_regen.link(), third_link);
    assert_eq!(
        updated_variant_after_regen.component_type(),
        third_component_type
    );

    // component has been upgraded to the new variant with the old type though
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, diagram.components.len());

    let diagram_component = diagram.components.first().expect("has one component");
    assert_eq!(diagram_component.color, third_color);
    assert_eq!(
        diagram_component.component_type,
        second_component_type.to_string()
    );
    assert_eq!(diagram_component.schema_category, third_category);
    assert_eq!(
        diagram_component.schema_variant_id,
        updated_sv_id_after_regen
    );
    assert_eq!(diagram_component.schema_name, second_asset_name);
    assert_eq!(diagram_component.display_name, component_name);

    // now create a second component which should have the latest default values in all places (including type)
    let component =
        create_component_for_unlocked_schema_name_on_default_view(ctx, schema.name(), "component")
            .await
            .expect("could not create compoennt");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    let second_component_name = component.name(ctx).await.expect("could not get name");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(2, diagram.components.len());

    let second_diagram_component = diagram
        .components
        .iter()
        .find(|c| first_component_id != c.id)
        .expect("could not find qualification");

    assert_eq!(second_diagram_component.color, third_color);
    assert_eq!(
        second_diagram_component.component_type,
        third_component_type.to_string()
    );
    assert_eq!(second_diagram_component.schema_category, third_category);
    assert_eq!(
        second_diagram_component.schema_variant_id,
        updated_sv_id_after_regen
    );
    assert_eq!(second_diagram_component.schema_name, second_asset_name);
    assert_eq!(second_diagram_component.display_name, second_component_name);
}

#[test]
async fn update_variant_with_new_prototypes_for_new_func(ctx: &mut DalContext) {
    let first_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "helix",
        None,
        None,
        "modal editors",
        "#00b0b0",
    )
    .await
    .expect("could not create variant");
    let schema = first_variant
        .schema(ctx)
        .await
        .expect("could not get schema");

    // Update the variant with a new string prop.
    VariantAuthoringClient::save_variant_content(
        ctx,
        first_variant.id(),
        &schema.name,
        first_variant.display_name(),
        first_variant.category(),
        first_variant.description(),
        first_variant.link(),
        first_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        first_variant.component_type(),
        Some("function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()\n  return new AssetBuilder().addProp(myProp).build()\n}".to_string()),
    )
        .await
        .expect("save variant contents");

    let updated_variant_id = VariantAuthoringClient::regenerate_variant(ctx, first_variant.id())
        .await
        .expect("unable to update variant");
    assert_eq!(
        first_variant.id(), // expected
        updated_variant_id  // actual
    );

    // Check that the prop exists from the update.
    let prop_id = Prop::find_prop_id_by_path(
        ctx,
        updated_variant_id,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await
    .expect("could not find prop id by path");

    // Create a new func and attach it to the new prop.
    let created_func = FuncAuthoringClient::create_new_attribute_func(
        ctx,
        Some("zellij".to_string()),
        Some(EventualParent::SchemaVariant(updated_variant_id)),
        dal::func::binding::AttributeFuncDestination::Prop(prop_id),
        vec![],
    )
    .await
    .expect("could not create func");

    // Create a component using the new variant (and schema).
    create_component_for_default_schema_name_in_default_view(ctx, &schema.name, "component")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, diagram.components.len());

    // Update the variant again, but this time, we should get a new schema variant id.
    VariantAuthoringClient::save_variant_content(
        ctx,
        first_variant.id(),
        &schema.name,
        first_variant.display_name(),
        first_variant.category(),
        first_variant.description(),
        first_variant.link(),
        first_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        first_variant.component_type(),
        Some("function main() {\n const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build();\n const anotherProp = new PropBuilder().setName(\"anotherProp\").setKind(\"integer\").build();\n  return new AssetBuilder().addProp(myProp).addProp(anotherProp).build()\n}".to_string()),
    )
        .await
        .expect("save variant contents");

    let second_updated_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, first_variant.id())
            .await
            .expect("could not update variant");
    assert_ne!(second_updated_variant_id, first_variant.id());

    // Commit to ensure graph cleanup
    commit_and_update_snapshot_to_visibility(ctx).await;

    // Create another component and check that the second prop exists on it.
    create_component_for_default_schema_name_in_default_view(ctx, schema.name, "component two")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(2, diagram.components.len());
    Prop::find_prop_id_by_path(
        ctx,
        second_updated_variant_id,
        &PropPath::new(["root", "domain", "anotherProp"]),
    )
    .await
    .expect("could not find prop id by path");

    // Check that all actual prototype pairs are what we expect.
    let mut actual_prototype_pairs = HashSet::new();
    let attribute_prototype_ids = AttributePrototype::list_ids_for_func_id(ctx, created_func.id)
        .await
        .expect("could not list ids for func id");
    assert_eq!(
        1,                             // expected,
        attribute_prototype_ids.len()  // actual
    );
    for id in attribute_prototype_ids {
        let eventual_parent = AttributePrototype::eventual_parent(ctx, id)
            .await
            .expect("could not find eventual parent");
        let schema_variant_id = match eventual_parent {
            AttributePrototypeEventualParent::SchemaVariantFromProp(schema_variant_id, _) => {
                schema_variant_id
            }
            _ => panic!("unexpected eventual parent: {eventual_parent:?}"),
        };
        actual_prototype_pairs.insert((id, EventualParent::SchemaVariant(schema_variant_id)));
    }

    // Ensure that the associations match the eventual parents.
    let bindings = FuncBinding::get_attribute_bindings_for_func_id(ctx, created_func.id)
        .await
        .expect("could not get bindings");

    assert_eq!(
        1,              // expected
        bindings.len()  // actual
    );
    let actual_prototype_pairs_from_associations: HashSet<(AttributePrototypeId, EventualParent)> =
        HashSet::from_iter(
            bindings
                .iter()
                .map(|p| (p.attribute_prototype_id, p.eventual_parent)),
        );
    assert_eq!(
        actual_prototype_pairs,                   // expected
        actual_prototype_pairs_from_associations  // actual
    );

    // Check that the variants of the pairs are what we expect. Check the total number of pairs.
    let expected_schema_variant_ids_in_pairs: HashSet<EventualParent> =
        HashSet::from([EventualParent::SchemaVariant(second_updated_variant_id)]);
    let actual_schema_variant_ids_in_pairs: HashSet<EventualParent> =
        HashSet::from_iter(actual_prototype_pairs.iter().map(|pair| pair.1));
    assert_eq!(
        expected_schema_variant_ids_in_pairs, // expected
        actual_schema_variant_ids_in_pairs    // actual
    );
}

#[test]
async fn update_variant_with_leaf_func(ctx: &mut DalContext) {
    let schema_variant = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "helix",
        None,
        None,
        "modal editors",
        "#00b0b0",
    )
    .await
    .expect("could not create variant");
    let schema = schema_variant
        .schema(ctx)
        .await
        .expect("could not get schema");

    // Update the variant with two string props. Ensure that we mutated the existing variant instead of creating a new one.
    let code = "\
        function main() {
            const input1 = new PropBuilder()
                .setName(\"input1\")
                .setKind(\"string\")
                .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\")
                    .build())
                .build();
            const calculate = new PropBuilder()
                .setName(\"calculated\")
                .setKind(\"string\")
                .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\")
                    .build())
                .build();
            return new AssetBuilder().addProp(input1).addProp(calculate).build()
        }";

    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant.id(),
        &schema.name,
        schema_variant.display_name(),
        schema_variant.category(),
        schema_variant.description(),
        schema_variant.link(),
        schema_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        schema_variant.component_type(),
        Some(code),
    )
    .await
    .expect("save variant contents");

    let first_update_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant.id())
            .await
            .expect("unable to update variant");
    assert_eq!(
        schema_variant.id(),     // expected
        first_update_variant_id  // actual
    );

    // Create a component.
    let component_one =
        create_component_for_default_schema_name_in_default_view(ctx, &schema.name, "one")
            .await
            .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, diagram.components.len());

    // Commit after creating the component because qualifications rely on dependent values update.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a qualification.
    let qualification_one_name = "qualification one";
    let created_func_one = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(qualification_one_name.to_string()),
        LeafKind::Qualification,
        EventualParent::SchemaVariant(first_update_variant_id),
        &[LeafInputLocation::Domain],
    )
    .await
    .expect("could not create func");

    // Add code to the qualification.
    let code = "\
        async function main(component: Input): Promise < Output > {
            if (component.domain?.input1) {
                var y: number = +component.domain?.input1 ?? 0;
                if (y > 0) {
                    return {
                        result: 'success',
                        message: 'Component qualified'
                    };
                }
                return {
                    result: 'failure',
                    message: 'Component not qualified'
                };

            }
            return {
                result: 'success',
                message: 'Component qualified'
            };

        }";

    FuncAuthoringClient::save_code(ctx, created_func_one.id, code.to_string())
        .await
        .expect("could not save code");

    // Create a second component.
    let component_two =
        create_component_for_default_schema_name_in_default_view(ctx, &schema.name, "two")
            .await
            .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(2, diagram.components.len());

    // Commit after creating the component because qualifications rely on dependent values update.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Collect the qualifications for all components and the prototypes for all functions.
    let component_one_qualifications = Component::list_qualifications(ctx, component_one.id())
        .await
        .expect("could not list qualifiications");
    let component_two_qualifications = Component::list_qualifications(ctx, component_two.id())
        .await
        .expect("could not list qualifications");

    let bindings = FuncBinding::get_qualification_bindings_for_func_id(ctx, created_func_one.id)
        .await
        .expect("could not get bindings");

    // Check the qualifications.
    assert_eq!(
        1,                                  // expected
        component_one_qualifications.len()  // actual
    );
    assert_eq!(
        1,                                  // expected
        component_two_qualifications.len()  // actual
    );
    let component_one_qualification_one_result = component_one_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    let component_two_qualification_one_result = component_two_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    assert_eq!(
        QualificationSubCheckStatus::Success,          // expected
        component_one_qualification_one_result.status  // actual
    );
    assert_eq!(
        QualificationSubCheckStatus::Success,          // expected
        component_two_qualification_one_result.status  // actual
    );

    // Check the prototype bags.
    assert_eq!(
        1,              // expected
        bindings.len(), // actual
    );
    let bag = bindings.first().expect("no bags found");
    let bag_schema_variant_id = bag.eventual_parent;
    assert_eq!(
        EventualParent::SchemaVariant(first_update_variant_id), // expected
        bag_schema_variant_id                                   // actual
    );

    // Update the variant with a third string prop. Ensure that a new schema variant was created.
    let code = "\
        function main() {
            const input1 = new PropBuilder()
                .setName(\"input1\")
                .setKind(\"string\")
                .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\")
                    .build())
                .build();
            const input2 = new PropBuilder()
                .setName(\"input2\")
                .setKind(\"string\")
                .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\")
                    .build())
                .build();
            const calculate = new PropBuilder()
                .setName(\"calculated\")
                .setKind(\"string\")
                .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\")
                    .build())
                .build();
            return new AssetBuilder().addProp(input1).addProp(input2).addProp(calculate).build()
        }";
    let schema_variant = SchemaVariant::get_by_id(ctx, first_update_variant_id)
        .await
        .expect("could not get schema variant");

    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant.id(),
        &schema.name,
        schema_variant.display_name(),
        schema_variant.category(),
        schema_variant.description(),
        schema_variant.link(),
        schema_variant
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        schema_variant.component_type(),
        Some(code),
    )
    .await
    .expect("save variant contents");

    let second_update_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant.id())
            .await
            .expect("unable to update variant");
    assert_ne!(first_update_variant_id, second_update_variant_id);

    // Create a second qualification.
    let qualification_two_name = "qualification two";
    let created_func_two = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(qualification_two_name.to_string()),
        LeafKind::Qualification,
        EventualParent::SchemaVariant(second_update_variant_id),
        &[],
    )
    .await
    .expect("can create qualification");

    // Add code to the second qualification.
    let code = "\
        async function main(component: Input): Promise < Output > {
            if (component.domain?.input2) {
                var y: number = +component.domain?.input2 ?? 0;
                if (y > 0) {
                    return {
                        result: 'success',
                        message: 'Component qualified'
                    };
                }
                return {
                    result: 'failure',
                    message: 'Component not qualified'
                };

            }
            return {
                result: 'success',
                message: 'Component qualified'
            };
        }";
    // let func = Func::get_by_id(ctx, created_func_two.id)
    //     .await
    //     .expect("could not get func");
    FuncAuthoringClient::save_code(ctx, created_func_two.id, code.to_string())
        .await
        .expect("can save code");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot");

    // Check the qualifications for all components.
    let component_one_qualifications = Component::list_qualifications(ctx, component_one.id())
        .await
        .expect("could not list qualifications");
    let component_two_qualifications = Component::list_qualifications(ctx, component_two.id())
        .await
        .expect("could not list qualifications");

    assert_eq!(
        2,                                  // expected
        component_one_qualifications.len()  // actual
    );
    assert_eq!(
        2,                                  // expected
        component_two_qualifications.len()  // actual
    );
    let component_one_qualification_one_result = component_one_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    let component_two_qualification_one_result = component_two_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    assert_eq!(
        QualificationSubCheckStatus::Success,          // expected
        component_one_qualification_one_result.status  // actual
    );
    assert_eq!(
        QualificationSubCheckStatus::Success,          // expected
        component_two_qualification_one_result.status  // actual
    );

    // Check the bindings for the first qualification.

    let bindings = FuncBinding::get_qualification_bindings_for_func_id(ctx, created_func_one.id)
        .await
        .expect("could not get binding");

    assert_eq!(
        1,              // expected
        bindings.len(), // actual
    );
    let actual: HashSet<SchemaVariantId> = HashSet::from_iter(bindings.iter().map(|b| {
        if let EventualParent::SchemaVariant(sv) = b.eventual_parent {
            Some(sv)
        } else {
            None
        }
        .expect("is a schema variant")
    }));
    let expected = HashSet::from_iter([second_update_variant_id]);
    assert_eq!(
        expected, // expected
        actual    // actual
    );

    // Check the bindings for the second qualification.

    let bindings = FuncBinding::get_qualification_bindings_for_func_id(ctx, created_func_two.id)
        .await
        .expect("could not get bindings");

    assert_eq!(
        1,              // expected
        bindings.len(), // actual
    );
    let bag = bindings.first().expect("no bags found");
    let bag_schema_variant_id = if let EventualParent::SchemaVariant(sv_id) = bag.eventual_parent {
        Some(sv_id)
    } else {
        None
    }
    .expect("schema variant is some");
    assert_eq!(
        second_update_variant_id, // expected
        bag_schema_variant_id     // actual
    );

    // Create a third component and re-check all qualifications.
    let component_three =
        create_component_for_unlocked_schema_name_on_default_view(ctx, &schema.name, "three")
            .await
            .expect("could not create component");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(3, diagram.components.len());

    // Commit after creating the component because qualifications rely on dependent values update.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Check qualifications for all components.
    // With the new regenerate flow, all components get the updates, not just the original two
    let component_one_qualifications = Component::list_qualifications(ctx, component_one.id())
        .await
        .expect("could not list qualifications");
    let component_two_qualifications = Component::list_qualifications(ctx, component_two.id())
        .await
        .expect("could not list qualifications");
    let component_three_qualifications = Component::list_qualifications(ctx, component_three.id())
        .await
        .expect("could not list qualifications");
    assert_eq!(
        2,                                  // expected
        component_one_qualifications.len()  // actual
    );
    assert_eq!(
        2,                                  // expected
        component_two_qualifications.len()  // actual
    );
    assert_eq!(
        2,                                    // expected
        component_three_qualifications.len()  // actual
    );
    let component_one_qualification_one_result = component_one_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    let component_two_qualification_one_result = component_two_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    let component_three_qualification_one_result = component_three_qualifications
        .iter()
        .find(|q| qualification_one_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    let component_three_qualification_two_result = component_three_qualifications
        .iter()
        .find(|q| qualification_two_name == q.qualification_name)
        .expect("could not find qualification")
        .result
        .to_owned()
        .expect("no result found");
    assert_eq!(
        QualificationSubCheckStatus::Success,          // expected
        component_one_qualification_one_result.status  // actual
    );
    assert_eq!(
        QualificationSubCheckStatus::Success,          // expected
        component_two_qualification_one_result.status  // actual
    );
    assert_eq!(
        QualificationSubCheckStatus::Success,            // expected
        component_three_qualification_one_result.status  // actual
    );
    assert_eq!(
        QualificationSubCheckStatus::Success,            // expected
        component_three_qualification_two_result.status  // actual
    );

    // Re-check the bindings after the third component was created.
    {
        // Check the bindings for the first qualification.

        let bindings =
            FuncBinding::get_qualification_bindings_for_func_id(ctx, created_func_one.id)
                .await
                .expect("could not create bindings");

        assert_eq!(
            1,              // expected
            bindings.len(), // actual
        );
        let actual: HashSet<SchemaVariantId> = HashSet::from_iter(bindings.iter().map(|b| {
            if let EventualParent::SchemaVariant(sv) = b.eventual_parent {
                Some(sv)
            } else {
                None
            }
            .expect("is a schema variant")
        }));
        let expected = HashSet::from_iter([second_update_variant_id]);
        assert_eq!(
            expected, // expected
            actual    // actual
        );

        // Check the bindings for the second qualification.
        let bindings =
            FuncBinding::get_qualification_bindings_for_func_id(ctx, created_func_two.id)
                .await
                .expect("could not create bindings");

        assert_eq!(
            1,              // expected
            bindings.len(), // actual
        );
        let bag = bindings.first().expect("no bags found");
        let bag_schema_variant_id =
            if let EventualParent::SchemaVariant(sv_id) = bag.eventual_parent {
                Some(sv_id)
            } else {
                None
            }
            .expect("schema variant is some");
        assert_eq!(
            second_update_variant_id, // expected
            bag_schema_variant_id     // actual
        );
    }
}
