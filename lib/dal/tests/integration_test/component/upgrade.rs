use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::Action;
use dal::diagram::Diagram;
use dal::func::authoring::FuncAuthoringClient;
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{AttributeValue, Component, ComponentType, DalContext, Prop, SchemaVariant};
use dal_test::expected::{ExpectComponent, ExpectSchema, ExpectSchemaVariant};
use dal_test::helpers::{create_component_for_default_schema_name, PropEditorTestView};
use dal_test::test;
use itertools::Itertools;
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use std::collections::VecDeque;
// TODO test that validates that components that exist on locked variants aren't auto upgraded, but can be upgraded manually

// Components that exist on the unlocked variant get auto upgraded when it gets regenerated
#[test]
async fn auto_upgrade_component(ctx: &mut DalContext) {
    // Let's create a new asset
    let variant_zero = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "paulsTestAsset",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
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

    let created_func = FuncAuthoringClient::create_new_action_func(
        ctx,
        Some("create Paul's test asset".to_owned()),
        ActionKind::Create,
        variant_zero.id(),
    )
    .await
    .expect("could not create action func");

    FuncAuthoringClient::save_code(ctx, created_func.id, create_action_code.to_owned())
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

    assert_eq!(my_upgraded_component.schema_variant_id, variant_one.into());

    let view_after_upgrade = Component::get_by_id(ctx, my_upgraded_component.component_id.into())
        .await
        .unwrap()
        .view(ctx)
        .await
        .expect("get component view");

    let root_id = Component::root_attribute_value_id(ctx, my_upgraded_component.id.into())
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
    let mut actions = Action::find_for_component_id(ctx, my_upgraded_component.id.into())
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

#[test]
async fn upgrade_component_type(ctx: &mut DalContext) {
    //
    // Create a new schema and variant set to component_type: ConfigurationFrameDown
    //
    let variant_zero = ExpectSchemaVariant(
        VariantAuthoringClient::create_schema_and_variant(
            ctx,
            "any variant",
            None,
            None,
            "Integration Tests",
            "#00b0b0",
        )
        .await
        .expect("Unable to create new asset")
        .id,
    );
    let updated = update_schema_variant_component_type(
        ctx,
        variant_zero,
        ComponentType::ConfigurationFrameDown,
    )
    .await;
    assert_eq!(variant_zero, updated);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        updated.get_type(ctx).await
    );

    //
    // Create a new component from the variant, with child
    //
    let component = variant_zero.create_component(ctx).await;
    let child = ExpectComponent::create(ctx, "Docker Image").await;
    child.upsert_parent(ctx, component).await;

    assert_eq!(variant_zero, component.schema_variant(ctx).await);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        component.get_type(ctx).await
    );

    //
    // Update the variant to be a Component that can't have parents
    //
    let variant_one =
        update_schema_variant_component_type(ctx, variant_zero, ComponentType::Component).await;
    assert_eq!(variant_zero, variant_one);
    assert_eq!(ComponentType::Component, variant_one.get_type(ctx).await);

    //
    // Check that the component is upgraded but is still set to ConfigurationFrameDown
    //
    assert_eq!(variant_one, component.schema_variant(ctx).await);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        component.get_type(ctx).await
    );
}

#[test]
async fn upgrade_component_type_after_explicit_set(ctx: &mut DalContext) {
    //
    // Create a new schema and variant set to component_type: ConfigurationFrameDown
    //
    let variant_zero = ExpectSchemaVariant(
        VariantAuthoringClient::create_schema_and_variant(
            ctx,
            "any variant",
            None,
            None,
            "Integration Tests",
            "#00b0b0",
        )
        .await
        .expect("Unable to create new asset")
        .id,
    );
    let updated = update_schema_variant_component_type(
        ctx,
        variant_zero,
        ComponentType::ConfigurationFrameDown,
    )
    .await;
    assert_eq!(variant_zero, updated);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        updated.get_type(ctx).await
    );

    //
    // Create a new component from the variant, and set its type to Component
    //
    let component = variant_zero.create_component(ctx).await;
    component.set_type(ctx, ComponentType::Component).await;

    assert_eq!(variant_zero, component.schema_variant(ctx).await);
    assert_eq!(ComponentType::Component, component.get_type(ctx).await);

    //
    // Update the variant (we add a new description)
    //
    let variant_one = update_schema_variant_description(ctx, variant_zero, "eek").await;
    assert_eq!(variant_zero, variant_one);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        variant_one.get_type(ctx).await
    );

    //
    // Check that the component is upgraded but is still set to ConfigurationFrameDown
    //
    assert_eq!(variant_one, component.schema_variant(ctx).await);
    assert_eq!(ComponentType::Component, component.get_type(ctx).await);
}

#[test]
async fn create_unlocked_schema_variant_after_component_changes_component_type(
    ctx: &mut DalContext,
) {
    let swifty = ExpectSchema::find(ctx, "swifty")
        .await
        .default_variant(ctx)
        .await;
    assert_eq!(
        ComponentType::ConfigurationFrameUp,
        swifty.get_type(ctx).await
    );

    //
    // Create a new component from the variant, and set its type to Component
    //
    let component = swifty.create_component(ctx).await;
    component.set_type(ctx, ComponentType::Component).await;

    assert_eq!(swifty, component.schema_variant(ctx).await);
    assert_eq!(ComponentType::Component, component.get_type(ctx).await);
    assert_eq!(
        ComponentType::ConfigurationFrameUp,
        swifty.get_type(ctx).await
    );

    //
    // Update the variant (we add a new description)
    //
    let copy = swifty.create_unlocked_copy(ctx).await;

    assert_ne!(swifty, copy);
    assert_eq!(
        ComponentType::ConfigurationFrameUp,
        swifty.get_type(ctx).await
    );
    assert_eq!(
        ComponentType::ConfigurationFrameUp,
        copy.get_type(ctx).await
    );
}

#[test]
async fn upgrade_array_of_objects(ctx: &mut DalContext) {
    let variant_code = r#"
    function main() {
        const networkInterfacesProp = new PropBuilder()
            .setKind("array")
            .setName("things")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("array").build())
            .setEntry(new PropBuilder()
                .setKind("object")
                .setName("thing")
                .addChild(new PropBuilder()
                    .setKind("string")
                    .setName("description")
                    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                    .build())
                .build())
            .build()
        return new AssetBuilder()
            .addProp(networkInterfacesProp)
            .build();
     }"#;

    let variant_zero = VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        "withArrayOfObjects",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
        variant_code,
    )
    .await
    .expect("Unable to create new asset");

    let my_asset_schema = variant_zero
        .schema(ctx)
        .await
        .expect("Unable to get the schema for the variant");

    // Create a component, add fields to array, set values
    let component = create_component_for_default_schema_name(
        ctx,
        my_asset_schema.name.clone(),
        "demo component",
    )
    .await
    .expect("could not create component");

    let things_path = &["root", "domain", "things"];
    let things_av_id = component
        .attribute_values_for_prop(ctx, things_path)
        .await
        .expect("find value ids for the prop things")
        .pop()
        .expect("there should only be one value id");

    let mut description_av_ids = vec![];
    let description_values = ["A", "B", "C"]
        .into_iter()
        .map(|str| json!(str))
        .collect_vec();
    for value in &description_values {
        let thing_av = AttributeValue::insert(ctx, things_av_id, None, None)
            .await
            .expect("unable to add field to array");

        let description_av_id = AttributeValue::all_object_children_to_leaves(ctx, thing_av)
            .await
            .expect("could not get child of thing")
            .pop()
            .expect("thing should have a child");

        AttributeValue::update(ctx, description_av_id, Some(value.to_owned()))
            .await
            .expect("could not update thing description");

        description_av_ids.push(description_av_id);
    }

    // Check that values were set successfully
    for index in 0..2 {
        let index_str = index.to_string();
        let value_path = ["root", "domain", "things", &index_str, "description"];
        PropEditorTestView::for_component_id(ctx, component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(&value_path)
            .expect("could not get description value");

        let value = description_values
            .get(index)
            .expect("unable to get reference value")
            .to_owned();

        let av_id = description_av_ids
            .get(index)
            .expect("unable to get reference av id")
            .to_owned();

        let prop_id = AttributeValue::prop_id(ctx, av_id)
            .await
            .expect("get prop_id for attribute value");

        std::assert_eq!(
            json![{
                "id": av_id,
                "propId": prop_id,
                "key": null,
                "value": value,
                "validation": null,
                "canBeSetBySocket": false,
                "isFromExternalSource": false,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": true
            }], // expected
            PropEditorTestView::for_component_id(ctx, component.id())
                .await
                .expect("could not get property editor test view")
                .get_value(&value_path)
                .expect("could not get value")
        );
    }

    // Regenerate, check that the fields in the component are still there and with the correct values
    VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id())
        .await
        .expect("unable to update asset");

    // Since the variant was regenerated, we need to gather the description av ids back
    let upgraded_component = Component::get_by_id(ctx, component.id())
        .await
        .expect("unable to find component after regeneration");

    let things_av_id = upgraded_component
        .attribute_values_for_prop(ctx, things_path)
        .await
        .expect("find value ids for the prop things")
        .pop()
        .expect("there should only be one value id");

    let mut regenerated_description_av_ids = vec![];
    for thing_av in AttributeValue::get_child_av_ids_in_order(ctx, things_av_id)
        .await
        .expect("unable to get thing av ids")
    {
        let description_av_id = AttributeValue::all_object_children_to_leaves(ctx, thing_av)
            .await
            .expect("could not get child of thing")
            .pop()
            .expect("thing should have a child");

        regenerated_description_av_ids.push(description_av_id);
    }

    // Check that the values are there on the regenerated component
    for index in 0..3 {
        let index_str = index.to_string();
        let value_path = ["root", "domain", "things", &index_str, "description"];
        PropEditorTestView::for_component_id(ctx, upgraded_component.id())
            .await
            .expect("could not get property editor test view")
            .get_value(&value_path)
            .expect("could not get description value");

        let value = description_values
            .get(index)
            .expect("unable to get reference value")
            .to_owned();

        let av_id = regenerated_description_av_ids
            .get(index)
            .expect("unable to get reference av id")
            .to_owned();

        let prop_id = AttributeValue::prop_id(ctx, av_id)
            .await
            .expect("get prop_id for attribute value");

        std::assert_eq!(
            json![{
                "id": av_id,
                "propId": prop_id,
                "key": null,
                "value": value,
                "validation": null,
                "canBeSetBySocket": false,
                "isFromExternalSource": false,
                "isControlledByAncestor": false,
                "isControlledByDynamicFunc": false,
                "overridden": true
            }], // expected
            PropEditorTestView::for_component_id(ctx, upgraded_component.id())
                .await
                .expect("could not get property editor test view")
                .get_value(&value_path)
                .expect("could not get value")
        );
    }
}

async fn update_schema_variant_component_type(
    ctx: &mut DalContext,
    variant: ExpectSchemaVariant,
    component_type: ComponentType,
) -> ExpectSchemaVariant {
    let variant = variant.schema_variant(ctx).await;
    VariantAuthoringClient::save_variant_content(
        ctx,
        variant.id(),
        "test schema",
        variant.display_name(),
        variant.category(),
        variant.description(),
        variant.link(),
        variant.color(),
        component_type,
        None as Option<String>,
    )
    .await
    .expect("save variant contents");

    SchemaVariant::get_by_id_or_error(ctx, variant.id)
        .await
        .expect("could not get updated variant")
        .into()
}

async fn update_schema_variant_description(
    ctx: &mut DalContext,
    variant: ExpectSchemaVariant,
    description: impl Into<String>,
) -> ExpectSchemaVariant {
    let variant = variant.schema_variant(ctx).await;
    VariantAuthoringClient::save_variant_content(
        ctx,
        variant.id(),
        "test schema",
        variant.display_name(),
        variant.category(),
        Some(description.into()),
        variant.link(),
        variant.color(),
        variant.component_type(),
        None as Option<String>,
    )
    .await
    .expect("save variant contents");
    SchemaVariant::get_by_id_or_error(ctx, variant.id)
        .await
        .expect("could not get updated variant")
        .into()
}
