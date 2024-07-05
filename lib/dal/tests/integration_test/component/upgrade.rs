use std::collections::VecDeque;

use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::Action;
use dal::diagram::Diagram;
use dal::func::authoring::{CreateFuncOptions, FuncAuthoringClient};
use dal::func::FuncAssociations;
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{AttributeValue, Component, DalContext, Prop};
use dal_test::helpers::create_component_for_default_schema_name;
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

// TODO test that validates that components that exist on locked variants aren't auto upgraded, but can be upgraded manually

// Components that exist on the unlocked variant get auto upgraded when it gets regenerated
#[test]
async fn auto_upgrade_component(ctx: &mut DalContext) {
    // Let's create a new asset
    let asset_name = "paulsTestAsset".to_string();
    let description = None;
    let link = None;
    let category = "Integration Tests".to_string();
    let color = "#00b0b0".to_string();
    let variant_zero = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        asset_name.clone(),
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

    // Build Create Action Func
    let create_action_code = "async function main() {
        return { payload: { \"poop\": true }, status: \"ok\" };
    }";
    let create_action_opts = CreateFuncOptions::ActionOptions {
        schema_variant_id: variant_zero.id(),
        action_kind: ActionKind::Create,
    };

    let created_func = FuncAuthoringClient::create_func(
        ctx,
        dal::func::FuncKind::Action,
        Some("create Paul's test asset".to_owned()),
        Some(create_action_opts),
    )
    .await
    .expect("could create action func");
    let create_func_associations = FuncAssociations::Action {
        kind: ActionKind::Create,
        schema_variant_ids: vec![variant_zero.id()],
    };

    FuncAuthoringClient::save_func(
        ctx,
        created_func.id,
        Some("Test Create Action".to_owned()),
        "create Paul's test asset".to_owned(),
        None,
        Some(create_action_code.to_owned()),
        Some(create_func_associations),
    )
    .await
    .expect("could save func");

    // Now let's update the variant
    let first_code_update = "function main() {\n
         const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build()
         const myProp2 = new PropBuilder().setName(\"testPropWillRemove\").setKind(\"string\").build()
         const arrayProp = new PropBuilder().setName(\"arrayProp\").setKind(\"array\").setEntry(\n
            new PropBuilder().setName(\"arrayElem\").setKind(\"string\").build()\n
        ).build();\n
         return new AssetBuilder().addProp(myProp).addProp(arrayProp).build()\n}"
        .to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        variant_zero.id(),
        my_asset_schema.name.clone(),
        variant_zero.display_name(),
        variant_zero.category(),
        variant_zero.description(),
        variant_zero.link(),
        variant_zero
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        variant_zero.component_type(),
        Some(first_code_update),
    )
    .await
    .expect("save variant contents");

    let updated_variant_id = VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id())
        .await
        .expect("unable to update asset");

    // We should still see that the schema variant we updated is the same as we have no components on the graph
    assert_eq!(variant_zero.id(), updated_variant_id);

    // Add a component to the diagram
    let initial_component = create_component_for_default_schema_name(
        ctx,
        my_asset_schema.name.clone(),
        "demo component",
    )
    .await
    .expect("could not create component");
    let initial_diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(1, initial_diagram.components.len());

    let domain_prop_av_id = initial_component
        .domain_prop_attribute_value(ctx)
        .await
        .expect("able to get domain prop");

    // Set the domain so we get some array elements
    AttributeValue::update(
        ctx,
        domain_prop_av_id,
        Some(serde_json::json!({
            "testProp": "test",
            "testPropWillRemove": "testToBeRemoved",
            "arrayProp": [
                "first",
                "second"
            ]
        })),
    )
    .await
    .expect("update failed");

    // see that there's one action enqueued
    let mut actions = Action::find_for_component_id(ctx, initial_component.id())
        .await
        .expect("got the actions");
    assert!(actions.len() == 1);

    // and the one action is a create
    let create_action_id = actions.pop().expect("has an action");

    let create_prototype_id = Action::prototype_id(ctx, create_action_id)
        .await
        .expect("found action prototype");
    let create_prototype = ActionPrototype::get_by_id(ctx, create_prototype_id)
        .await
        .expect("got prototype");
    assert_eq!(create_prototype.kind, ActionKind::Create);

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
    let second_code_update = "function main() {\n
        const myProp = new PropBuilder().setName(\"testProp\").setKind(\"string\").build();\n
        const anotherProp = new PropBuilder().setName(\"anotherProp\").setKind(\"integer\").build();\n
        const arrayProp = new PropBuilder().setName(\"arrayProp\").setKind(\"array\").setEntry(\n
            new PropBuilder().setName(\"arrayElem\").setKind(\"string\").build()\n
        ).build();\n

         return new AssetBuilder().addProp(myProp).addProp(arrayProp).addProp(anotherProp).build()\n
        }".to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        variant_zero.id(),
        my_asset_schema.name,
        variant_zero.display_name(),
        variant_zero.category(),
        variant_zero.description(),
        variant_zero.link(),
        variant_zero
            .get_color(ctx)
            .await
            .expect("get color from schema variant"),
        variant_zero.component_type(),
        Some(second_code_update),
    )
    .await
    .expect("save variant contents");

    let variant_one = VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id())
        .await
        .expect("upgrade variant");

    // We should have a NEW schema variant id as there is a component on the graph
    assert_ne!(variant_one, variant_zero.id());

    // Check that the props exist for the new variant
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

    // Check that the component has been auto upgraded
    let one_component_on_the_graph = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(one_component_on_the_graph.components.len(), 1);
    let my_upgraded_component = one_component_on_the_graph
        .components
        .first()
        .expect("unable to get the upgradable component on the graph");
    assert_eq!(my_upgraded_component.schema_variant_id, variant_one);

    let view_after_upgrade = Component::get_by_id(ctx, my_upgraded_component.component_id)
        .await
        .unwrap()
        .view(ctx)
        .await
        .expect("get component view");

    let root_id = Component::root_attribute_value_id(ctx, my_upgraded_component.id)
        .await
        .expect("unable to get root av id");

    let mut value_q = VecDeque::from([root_id]);

    while let Some(current_av_id) = value_q.pop_front() {
        let is_for = AttributeValue::is_for(ctx, current_av_id)
            .await
            .expect("get is for");
        if let Some(prop_id) = is_for.prop_id() {
            let prop_variant_id = Prop::schema_variant_id(ctx, prop_id)
                .await
                .expect("get sv for prop")
                .expect("should have a sv");

            assert_eq!(variant_one, prop_variant_id);
        }
        for child_av_id in AttributeValue::get_child_av_ids_in_order(ctx, current_av_id)
            .await
            .expect("unable to get child av ids")
        {
            value_q.push_back(child_av_id);
        }
    }
    // This test confirms we handle deleted props, and we copy over the values of arrays
    assert_eq!(
        Some(serde_json::json!({
            "si": {
                "name": "demo component",
                "type": "component",
                "color": "#00b0b0",
            },
            "domain": {
                "testProp": "test",
                "arrayProp": [
                    "first",
                    "second",
                ]
            }
        })),
        view_after_upgrade
    );

    // see that there's still only one action enqueued
    let mut actions = Action::find_for_component_id(ctx, my_upgraded_component.id)
        .await
        .expect("got the actions");
    assert_eq!(actions.len(), 1);

    // and the one action is a create
    let create_action_id = actions.pop().expect("has an action");

    let create_prototype_id = Action::prototype_id(ctx, create_action_id)
        .await
        .expect("found action prototype");
    let create_prototype = ActionPrototype::get_by_id(ctx, create_prototype_id)
        .await
        .expect("got prototype");
    assert_eq!(create_prototype.kind, ActionKind::Create);

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
}
