use std::collections::HashSet;

use dal::attribute::prototype::AttributePrototypeEventualParent;
use dal::diagram::Diagram;
use dal::func::authoring::{AttributeOutputLocation, CreateFuncOptions, FuncAuthoringClient};
use dal::func::binding::EventualParent;
use dal::func::view::FuncView;
use dal::func::{AttributePrototypeBag, FuncAssociations, FuncKind};
use dal::prop::PropPath;
use dal::qualification::QualificationSubCheckStatus;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::schema::variant::leaves::LeafKind;
use dal::{
    AttributePrototype, AttributePrototypeId, Component, DalContext, Func, Prop, SchemaVariant,
    SchemaVariantId,
};
use dal_test::helpers::{
    create_component_for_default_schema_name, create_component_for_unlocked_schema_name,
    ChangeSetTestHelpers,
};
use dal_test::test;

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

    let default_schema_variant = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");
    assert!(default_schema_variant.is_some());
    assert_eq!(default_schema_variant, Some(first_variant.id()));

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
    create_component_for_default_schema_name(ctx, schema.name.clone(), "demo component")
        .await
        .expect("could not create component");
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
    create_component_for_default_schema_name(ctx, schema.name.clone(), "demo component 2")
        .await
        .expect("could not create component");
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
    let updated_default_schema_variant = schema
        .get_default_schema_variant_id(ctx)
        .await
        .expect("unable to get the default schema variant id");
    assert!(updated_default_schema_variant.is_some());
    assert_eq!(updated_default_schema_variant, Some(second_updated_sv_id));
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
    let created_func = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Attribute,
        Some("zellij".to_string()),
        Some(CreateFuncOptions::AttributeOptions {
            output_location: AttributeOutputLocation::Prop { prop_id },
        }),
    )
    .await
    .expect("could not create func");

    // Create a component using the new variant (and schema).
    create_component_for_default_schema_name(ctx, &schema.name, "component")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble(ctx)
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
            .expect("could not udpate variant");
    assert_ne!(second_updated_variant_id, first_variant.id());

    // Create another component and check that the second prop exists on it.
    create_component_for_default_schema_name(ctx, schema.name, "component two")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble(ctx)
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
        2,                             // expected,
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
        actual_prototype_pairs.insert((id, schema_variant_id));
    }

    // Ensure that the associations match the eventual parents.
    let func = Func::get_by_id_or_error(ctx, created_func.id)
        .await
        .expect("could not get func");
    let (associations, _input_type) = FuncAssociations::from_func(ctx, &func)
        .await
        .expect("could not get associations");
    let actual_prototype_pairs_from_associations: HashSet<(AttributePrototypeId, SchemaVariantId)> =
        match associations.expect("no associations found") {
            FuncAssociations::Attribute { prototypes } => {
                assert_eq!(
                    2,                // expected,
                    prototypes.len()  // actual
                );
                HashSet::from_iter(
                    prototypes
                        .iter()
                        .map(|p| (p.id, p.schema_variant_id.expect("no schema variant id"))),
                )
            }
            associations => panic!("unexpected associations: {associations:?}"),
        };
    assert_eq!(
        actual_prototype_pairs,                   // expected
        actual_prototype_pairs_from_associations  // actual
    );

    // Check that the variants of the pairs are we what expect. Check the total number of pairs.
    let expected_schema_variant_ids_in_pairs: HashSet<SchemaVariantId> =
        HashSet::from([first_variant.id(), second_updated_variant_id]);
    let actual_schema_variant_ids_in_pairs: HashSet<SchemaVariantId> =
        HashSet::from_iter(actual_prototype_pairs.iter().map(|pair| pair.1));
    assert_eq!(
        expected_schema_variant_ids_in_pairs, // expected
        actual_schema_variant_ids_in_pairs    // actual
    );
    assert_eq!(
        2,                                        // expected
        actual_schema_variant_ids_in_pairs.len()  // actual
    )
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
    let component_one = create_component_for_default_schema_name(ctx, &schema.name, "one")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, diagram.components.len());

    // Commit after creating the component because qualifications rely on dependent values update.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a qualification.
    let qualification_one_name = "qualification one";
    let created_func_one = FuncAuthoringClient::create_func(
        ctx,
        FuncKind::Qualification,
        Some(qualification_one_name.to_string()),
        Some(CreateFuncOptions::QualificationOptions {
            schema_variant_id: first_update_variant_id,
        }),
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
    let func = Func::get_by_id_or_error(ctx, created_func_one.id)
        .await
        .expect("could not get func");
    let func_view = FuncView::assemble(ctx, &func)
        .await
        .expect("could not assemble func view");
    FuncAuthoringClient::save_func(
        ctx,
        func_view.id,
        func_view.display_name,
        func_view.name,
        func_view.description,
        Some(code.to_string()),
        func_view.associations,
    )
    .await
    .expect("could not save func");

    // Create a second component.
    let component_two = create_component_for_default_schema_name(ctx, &schema.name, "two")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble(ctx)
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
    let attribute_prototype_ids =
        AttributePrototype::list_ids_for_func_id(ctx, created_func_one.id)
            .await
            .expect("could not list attribute prototype ids");
    let mut bags = Vec::new();
    for id in attribute_prototype_ids {
        bags.push(
            AttributePrototypeBag::assemble(ctx, id)
                .await
                .expect("could not assemble"),
        );
    }

    // Check the qualifications.
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

    // Check the prototype bags.
    assert_eq!(
        1,          // expected
        bags.len(), // actual
    );
    let bag = bags.first().expect("no bags found");
    let bag_schema_variant_id = bag.schema_variant_id.expect("schema variant id not found");
    assert_eq!(
        first_update_variant_id, // expected
        bag_schema_variant_id    // actual
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
    let schema_variant = SchemaVariant::get_by_id_or_error(ctx, first_update_variant_id)
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
    // let func = Func::get_by_id_or_error(ctx, created_func_two.id)
    //     .await
    //     .expect("could not get func");
    FuncAuthoringClient::save_code(ctx, created_func_two.id, code.to_string())
        .await
        .expect("can save code");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot");

    // Check the qualifications for all components.
    let component_one_qualifications =
        dbg!(Component::list_qualifications(ctx, component_one.id())
            .await
            .expect("could not list qualifications"));
    let component_two_qualifications =
        dbg!(Component::list_qualifications(ctx, component_two.id())
            .await
            .expect("could not list qualifications"));

    assert_eq!(
        3,                                  // expected
        component_one_qualifications.len()  // actual
    );
    assert_eq!(
        3,                                  // expected
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

    // Check the prototype bag for the first qualification.
    let attribute_prototype_ids =
        AttributePrototype::list_ids_for_func_id(ctx, created_func_one.id)
            .await
            .expect("could not list attribute prototype ids");
    let mut bags = Vec::new();
    for id in attribute_prototype_ids {
        bags.push(
            AttributePrototypeBag::assemble(ctx, id)
                .await
                .expect("could not assemble"),
        );
    }
    assert_eq!(
        2,          // expected
        bags.len(), // actual
    );
    let acutal: HashSet<SchemaVariantId> = HashSet::from_iter(
        bags.iter()
            .map(|b| b.schema_variant_id.expect("schema variant id not found")),
    );
    let expected = HashSet::from_iter([first_update_variant_id, second_update_variant_id]);
    assert_eq!(
        expected, // expected
        acutal    // actual
    );

    // Check the prototype bag for the second qualification.
    let attribute_prototype_ids =
        AttributePrototype::list_ids_for_func_id(ctx, created_func_two.id)
            .await
            .expect("could not list attribute prototype ids");
    let mut bags = Vec::new();
    for id in attribute_prototype_ids {
        bags.push(
            AttributePrototypeBag::assemble(ctx, id)
                .await
                .expect("could not assemble"),
        );
    }
    assert_eq!(
        1,          // expected
        bags.len(), // actual
    );
    let bag = bags.first().expect("no bags found");
    let bag_schema_variant_id = bag.schema_variant_id.expect("schema variant id not found");
    assert_eq!(
        second_update_variant_id, // expected
        bag_schema_variant_id     // actual
    );

    // Create a third component and re-check all qualifications.
    let component_three = create_component_for_unlocked_schema_name(ctx, &schema.name, "three")
        .await
        .expect("could not create component");
    let diagram = Diagram::assemble(ctx)
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
        3,                                  // expected
        component_one_qualifications.len()  // actual
    );
    assert_eq!(
        3,                                  // expected
        component_two_qualifications.len()  // actual
    );
    assert_eq!(
        3,                                    // expected
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

    // Re-check the prototype bags after the third component was created.
    {
        // Check the prototype bag for the first qualification.
        let attribute_prototype_ids =
            AttributePrototype::list_ids_for_func_id(ctx, created_func_one.id)
                .await
                .expect("could not list attribute prototype ids");
        let mut bags = Vec::new();
        for id in attribute_prototype_ids {
            bags.push(
                AttributePrototypeBag::assemble(ctx, id)
                    .await
                    .expect("could not assemble"),
            );
        }
        assert_eq!(
            2,          // expected
            bags.len(), // actual
        );
        let acutal: HashSet<SchemaVariantId> = HashSet::from_iter(
            bags.iter()
                .map(|b| b.schema_variant_id.expect("schema variant id not found")),
        );
        let expected = HashSet::from_iter([first_update_variant_id, second_update_variant_id]);
        assert_eq!(
            expected, // expected
            acutal    // actual
        );

        // Check the prototype bag for the second qualification.
        let attribute_prototype_ids =
            AttributePrototype::list_ids_for_func_id(ctx, created_func_two.id)
                .await
                .expect("could not list attribute prototype ids");
        let mut bags = Vec::new();
        for id in attribute_prototype_ids {
            bags.push(
                AttributePrototypeBag::assemble(ctx, id)
                    .await
                    .expect("could not assemble"),
            );
        }
        assert_eq!(
            1,          // expected
            bags.len(), // actual
        );
        let bag = bags.first().expect("no bags found");
        let bag_schema_variant_id = bag.schema_variant_id.expect("schema variant id not found");
        assert_eq!(
            second_update_variant_id, // expected
            bag_schema_variant_id     // actual
        );
    }
}
