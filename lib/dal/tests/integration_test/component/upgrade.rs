use std::collections::VecDeque;

use dal::{
    AttributeValue,
    ChangeSet,
    Component,
    ComponentId,
    ComponentType,
    DalContext,
    InputSocket,
    OutputSocket,
    Prop,
    Schema,
    SchemaVariant,
    action::{
        Action,
        prototype::{
            ActionKind,
            ActionPrototype,
        },
    },
    component::frame::Frame,
    diagram::Diagram,
    func::authoring::FuncAuthoringClient,
    prop::PropPath,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    expected::{
        ExpectComponent,
        ExpectSchema,
        ExpectSchemaVariant,
    },
    helpers::{
        ChangeSetTestHelpers,
        PropEditorTestView,
        attribute::value,
        change_set,
        create_component_for_default_schema_name_in_default_view,
        schema::variant::{
            self,
            SchemaVariantKey,
        },
    },
    test,
};
use itertools::Itertools;
use pretty_assertions_sorted::{
    assert_eq,
    assert_ne,
};
use serde_json::json;

use crate::integration_test::component::connectable_test::ConnectableTest;
// TODO test that validates that components that exist on locked variants aren't auto upgraded, but can be upgraded manually

// Components that exist on the unlocked variant get auto upgraded when it gets regenerated
#[test]
async fn auto_upgrade_component(ctx: &mut DalContext) -> Result<()> {
    // Let's create a new asset
    let variant_zero = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "paulsTestAsset",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
    )
    .await?;

    let my_asset_schema = variant_zero.schema(ctx).await?;

    let default_schema_variant = Schema::default_variant_id(ctx, my_asset_schema.id()).await?;
    assert_eq!(default_schema_variant, variant_zero.id());

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
    .await?;

    FuncAuthoringClient::save_code(ctx, created_func.id, create_action_code.to_owned()).await?;

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
        variant_zero.get_color(ctx).await?,
        variant_zero.component_type(),
        Some(first_code_update),
    )
    .await?;

    let updated_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id()).await?;

    // We should still see that the schema variant we updated is the same as we have no components on the graph
    assert_eq!(variant_zero.id(), updated_variant_id);

    // Add a component to the diagram
    let initial_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        my_asset_schema.name.clone(),
        "demo component",
    )
    .await?;
    let initial_diagram = Diagram::assemble_for_default_view(ctx).await?;
    assert_eq!(1, initial_diagram.components.len());

    let domain_prop_av_id = initial_component.domain_prop_attribute_value(ctx).await?;

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
    .await?;

    // see that there's one action enqueued
    let mut actions = Action::find_for_component_id(ctx, initial_component.id()).await?;
    assert!(actions.len() == 1);

    // and the one action is a create
    let create_action_id = actions.pop().expect("has an action");

    let create_prototype_id = Action::prototype_id(ctx, create_action_id).await?;
    let create_prototype = ActionPrototype::get_by_id(ctx, create_prototype_id).await?;
    assert_eq!(create_prototype.kind, ActionKind::Create);

    // Let's ensure that our prop is visible in the component
    Prop::find_prop_id_by_path(
        ctx,
        updated_variant_id,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await?;

    let initial_component_schema_variant = initial_component.schema_variant(ctx).await?;
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
        variant_zero.get_color(ctx).await?,
        variant_zero.component_type(),
        Some(second_code_update),
    )
    .await?;

    let variant_one = VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id()).await?;

    // We should have a NEW schema variant id as there is a component on the graph
    assert_ne!(variant_one, variant_zero.id());

    // Check that the props exist for the new variant
    Prop::find_prop_id_by_path(
        ctx,
        variant_one,
        &PropPath::new(["root", "domain", "testProp"]),
    )
    .await?;

    Prop::find_prop_id_by_path(
        ctx,
        variant_one,
        &PropPath::new(["root", "domain", "anotherProp"]),
    )
    .await?;

    // Check that the component has been auto upgraded
    let one_component_on_the_graph = Diagram::assemble_for_default_view(ctx).await?;
    assert_eq!(one_component_on_the_graph.components.len(), 1);
    let my_upgraded_component = one_component_on_the_graph
        .components
        .first()
        .expect("unable to get the upgradable component on the graph");

    assert_eq!(my_upgraded_component.schema_variant_id, variant_one);

    let view_after_upgrade = Component::get_by_id(ctx, my_upgraded_component.component_id)
        .await?
        .view(ctx)
        .await?;

    let root_id = Component::root_attribute_value_id(ctx, my_upgraded_component.id).await?;

    let mut value_q = VecDeque::from([root_id]);

    while let Some(current_av_id) = value_q.pop_front() {
        let is_for = AttributeValue::is_for(ctx, current_av_id).await?;
        if let Some(prop_id) = is_for.prop_id() {
            let prop_variant_id = Prop::schema_variant_id(ctx, prop_id)
                .await?
                .expect("should have a sv");

            assert_eq!(variant_one, prop_variant_id);
        }
        for child_av_id in AttributeValue::get_child_av_ids_in_order(ctx, current_av_id).await? {
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
    let mut actions = Action::find_for_component_id(ctx, my_upgraded_component.id).await?;
    assert_eq!(actions.len(), 1);

    // and the one action is a create
    let create_action_id = actions.pop().expect("has an action");

    let create_prototype_id = Action::prototype_id(ctx, create_action_id).await?;
    let create_prototype = ActionPrototype::get_by_id(ctx, create_prototype_id).await?;
    assert_eq!(create_prototype.kind, ActionKind::Create);

    let upgraded_graph = Diagram::assemble_for_default_view(ctx).await?;
    let upgraded_component = upgraded_graph
        .components
        .first()
        .expect("unable to get the upgraded component on the graph");
    assert_eq!(
        upgraded_component.can_be_upgraded, false,
        "the old asset should not be on the graph anymore, and the current one should be upgraded"
    );

    Ok(())
}

#[test]
async fn upgrade_component_type(ctx: &mut DalContext) -> Result<()> {
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
        .await?
        .id,
    );
    let updated = update_schema_variant_component_type(
        ctx,
        variant_zero,
        ComponentType::ConfigurationFrameDown,
    )
    .await?;
    assert_eq!(variant_zero, updated);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        updated.get_type(ctx).await
    );

    //
    // Create a new component from the variant, with child
    //
    let component = variant_zero.create_component_on_default_view(ctx).await;
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
        update_schema_variant_component_type(ctx, variant_zero, ComponentType::Component).await?;
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

    Ok(())
}

#[test]
async fn upgrade_component_type_after_explicit_set(ctx: &mut DalContext) -> Result<()> {
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
        .await?
        .id,
    );
    let updated = update_schema_variant_component_type(
        ctx,
        variant_zero,
        ComponentType::ConfigurationFrameDown,
    )
    .await?;
    assert_eq!(variant_zero, updated);
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        updated.get_type(ctx).await
    );

    //
    // Create a new component from the variant, and set its type to Component
    //
    let component = variant_zero.create_component_on_default_view(ctx).await;
    component.set_type(ctx, ComponentType::Component).await;

    assert_eq!(variant_zero, component.schema_variant(ctx).await);
    assert_eq!(ComponentType::Component, component.get_type(ctx).await);

    //
    // Update the variant (we add a new description)
    //
    let variant_one = update_schema_variant_description(ctx, variant_zero, "eek").await?;
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

    Ok(())
}

#[test]
async fn create_unlocked_schema_variant_after_component_changes_component_type(
    ctx: &mut DalContext,
) -> Result<()> {
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
    let component = swifty.create_component_on_default_view(ctx).await;
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

    Ok(())
}

#[test]
async fn upgrade_array_of_objects(ctx: &mut DalContext) -> Result<()> {
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
    .await?;

    let my_asset_schema = variant_zero.schema(ctx).await?;

    // Create a component, add fields to array, set values
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        my_asset_schema.name.clone(),
        "demo component",
    )
    .await?;

    let things_path = &["root", "domain", "things"];
    let things_av_id = component
        .attribute_values_for_prop(ctx, things_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    let mut description_av_ids = vec![];
    let description_values = ["A", "B", "C"]
        .into_iter()
        .map(|str| json!(str))
        .collect_vec();
    for value in &description_values {
        let thing_av = AttributeValue::insert(ctx, things_av_id, None, None).await?;

        let description_av_id = AttributeValue::all_object_children_to_leaves(ctx, thing_av)
            .await?
            .pop()
            .expect("thing should have a child");

        AttributeValue::update(ctx, description_av_id, Some(value.to_owned())).await?;

        description_av_ids.push(description_av_id);
    }

    // Check that values were set successfully
    for index in 0..2 {
        let index_str = index.to_string();
        let value_path = ["root", "domain", "things", &index_str, "description"];
        PropEditorTestView::for_component_id(ctx, component.id())
            .await?
            .get_value(&value_path)?;

        let value = description_values
            .get(index)
            .expect("unable to get reference value")
            .to_owned();

        let av_id = description_av_ids
            .get(index)
            .expect("unable to get reference av id")
            .to_owned();

        let prop_id = AttributeValue::prop_id(ctx, av_id).await?;

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
                .await?
                .get_value(&value_path)?
        );
    }

    // Regenerate, check that the fields in the component are still there and with the correct values
    VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id()).await?;

    // Since the variant was regenerated, we need to gather the description av ids back
    let upgraded_component = Component::get_by_id(ctx, component.id()).await?;

    let things_av_id = upgraded_component
        .attribute_values_for_prop(ctx, things_path)
        .await?
        .pop()
        .expect("there should only be one value id");

    let mut regenerated_description_av_ids = vec![];
    for thing_av in AttributeValue::get_child_av_ids_in_order(ctx, things_av_id).await? {
        let description_av_id = AttributeValue::all_object_children_to_leaves(ctx, thing_av)
            .await?
            .pop()
            .expect("thing should have a child");

        regenerated_description_av_ids.push(description_av_id);
    }

    // Check that the values are there on the regenerated component
    for index in 0..3 {
        let index_str = index.to_string();
        let value_path = ["root", "domain", "things", &index_str, "description"];
        PropEditorTestView::for_component_id(ctx, upgraded_component.id())
            .await?
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

        let prop_id = AttributeValue::prop_id(ctx, av_id).await?;

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
                .await?
                .get_value(&value_path)?
        );
    }

    Ok(())
}

#[test]
async fn upgrade_component_multiplayer_new_and_removed_sockets(ctx: &mut DalContext) -> Result<()> {
    // Set up connectables with multiple inputs
    let test = ConnectableTest::setup(ctx).await?;
    let parent = test.create_parent(ctx, "parent").await?;
    let manager = test.create_manager(ctx, "manager").await?;
    let input1 = test.create_connectable(ctx, "input1", None, []).await?;
    let input2 = test.create_connectable(ctx, "input2", None, []).await?;
    let component = test
        .create_connectable(ctx, "component", Some(input1), [input1, input2])
        .await?;
    let output1 = test
        .create_connectable(ctx, "output1", Some(component), [])
        .await?;
    let output2 = test
        .create_connectable(ctx, "output2", Some(component), [])
        .await?;
    let managed = test.create_connectable(ctx, "managed", None, []).await?;
    Frame::upsert_parent_for_tests(ctx, component.id, parent.id).await?;
    Component::manage_component(ctx, manager.id, component.id).await?;
    Component::manage_component(ctx, component.id, managed.id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let expect_domain = json!({
        "Value": "component",
        "One": "input1",
        "Many": [ "input1", "input2" ],
        "Inferred": "parent",
    });
    let expect_conns = vec![
        "[in] Many <- input1.Value",
        "[in] Many <- input2.Value",
        "[in] One <- input1.Value",
        "[inferred] Inferred <- parent.Inferred",
        "[managed] managed",
        "[manager] manager",
        "[out] Value -> output1.One",
        "[out] Value -> output2.One",
    ];
    assert_eq!(expect_domain, component.domain(ctx).await?);
    assert_eq!(expect_conns, connections(ctx, component.id).await?);
    assert_eq!(
        json!({
            "Value": "output1",
            "One": "component",
        }),
        output1.domain(ctx).await?
    );
    assert_eq!(
        json!({
            "Value": "output2",
            "One": "component",
        }),
        output2.domain(ctx).await?
    );

    // Create an unlocked, updated component with all existing sockets removed and new ones added
    SchemaVariant::get_by_id(ctx, test.connectable_variant_id)
        .await?
        .lock(ctx)
        .await?;
    let updated_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, test.connectable_variant_id)
            .await?
            .id();
    assert_ne!(updated_variant_id, test.connectable_variant_id);
    variant::update_asset_func(ctx,
        updated_variant_id,
        r#"
            function main() {
                return {
                    props: [
                        { name: "NewValue", kind: "string" },
                        { name: "NewOne", kind: "string", valueFrom: { kind: "inputSocket", socket_name: "NewOne" } },
                        { name: "NewMany", kind: "array",
                            entry: { name: "ManyItem", kind: "string" },
                            valueFrom: { kind: "inputSocket", socket_name: "NewMany" },
                        },
                    ],
                    inputSockets: [
                        { name: "NewOne", arity: "one", connectionAnnotations: "[\"Value\"]" },
                        { name: "NewMany", arity: "many", connectionAnnotations: "[\"Value\"]" },
                    ],
                    outputSockets: [
                        { name: "NewValue", arity: "one", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "NewValue" ] }, connectionAnnotations: "[\"Value\"]" },
                    ],
                };
            }
        "#,
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Update the component simultaneously in two multiplayer sessions
    let (player1, player2) = (ctx.clone(), ctx.clone());
    {
        Component::upgrade_to_new_variant(&player1, component.id, updated_variant_id).await?;
        player1.commit().await?;
    }
    {
        Component::upgrade_to_new_variant(&player2, component.id, updated_variant_id).await?;
        player2.commit().await?;
    }

    // Find out how the changeset ended up
    ChangeSet::wait_for_dvu(ctx, false).await?;
    // Make absolutely sure the socket AVs were not duplicated
    assert_eq!(
        3,
        Component::attribute_values_for_all_sockets(ctx, component.id)
            .await?
            .len()
    );
    assert_eq!(
        vec!["[managed] managed", "[manager] manager",],
        connections(ctx, component.id).await?
    );
    assert_eq!(json!({}), component.domain(ctx).await?);

    // Make sure output connections are still there
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    assert!(connections(ctx, output1.id).await?.is_empty());
    assert!(connections(ctx, output2.id).await?.is_empty());
    assert_eq!(json!({ "Value": "output1" }), output1.domain(ctx).await?);
    assert_eq!(json!({ "Value": "output2" }), output2.domain(ctx).await?);

    Ok(())
}

#[test]
async fn upgrade_component_with_subscriptions(ctx: &mut DalContext) -> Result<()> {
    let test = ConnectableTest::setup(ctx).await?;
    test.create_connectable(ctx, "in", None, []).await?;
    let out = test.create_connectable(ctx, "out", None, []).await?;
    value::subscribe(ctx, ("out", "/domain/Value"), ("in", "/domain/Value")).await?;
    value::set(ctx, ("in", "/domain/Value"), "old").await?;
    change_set::commit(ctx).await?;
    assert_eq!("old", value::get(ctx, ("out", "/domain/Value")).await?);

    // Upgrade and make sure value still flows through
    let updated = update_schema_variant_description(ctx, "connectable", "new").await?;
    Component::upgrade_to_new_variant(ctx, out.id, updated.id()).await?;
    value::set(ctx, ("in", "/domain/Value"), "new").await?;
    change_set::commit(ctx).await?;
    assert_eq!("new", value::get(ctx, ("out", "/domain/Value")).await?);

    Ok(())
}

async fn update_schema_variant_component_type(
    ctx: &mut DalContext,
    variant: impl SchemaVariantKey,
    component_type: ComponentType,
) -> Result<ExpectSchemaVariant> {
    let variant = SchemaVariant::get_by_id(ctx, variant::id(ctx, variant).await?).await?;
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
    .await?;

    Ok(SchemaVariant::get_by_id(ctx, variant.id).await?.into())
}

async fn update_schema_variant_description(
    ctx: &mut DalContext,
    variant: impl SchemaVariantKey,
    description: impl Into<String>,
) -> Result<ExpectSchemaVariant> {
    let variant = SchemaVariant::get_by_id(ctx, variant::id(ctx, variant).await?).await?;
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
    .await?;
    Ok(SchemaVariant::get_by_id(ctx, variant.id).await?.into())
}

async fn connections(ctx: &DalContext, component_id: ComponentId) -> Result<Vec<String>> {
    let mut result = vec![];
    for connection in Component::incoming_connections_for_id(ctx, component_id).await? {
        let from_component = Component::name_by_id(ctx, connection.from_component_id).await?;
        let from = OutputSocket::get_by_id(ctx, connection.from_output_socket_id).await?;
        let to = InputSocket::get_by_id(ctx, connection.to_input_socket_id).await?;
        result.push(format!(
            "[in] {} <- {}.{}",
            to.name(),
            from_component,
            from.name(),
        ));
    }
    for connection in Component::inferred_incoming_connections(ctx, component_id).await? {
        let from_component = Component::name_by_id(ctx, connection.from_component_id).await?;
        let from = OutputSocket::get_by_id(ctx, connection.from_output_socket_id).await?;
        let to = InputSocket::get_by_id(ctx, connection.to_input_socket_id).await?;
        result.push(format!(
            "[inferred] {} <- {}.{}",
            to.name(),
            from_component,
            from.name(),
        ));
    }
    for connection in Component::outgoing_connections_for_id(ctx, component_id).await? {
        let from = OutputSocket::get_by_id(ctx, connection.from_output_socket_id).await?;
        let to_component = Component::name_by_id(ctx, connection.to_component_id).await?;
        let to = InputSocket::get_by_id(ctx, connection.to_input_socket_id).await?;
        result.push(format!(
            "[out] {} -> {}.{}",
            from.name(),
            to_component,
            to.name(),
        ));
    }
    for manager_id in Component::managers_by_id(ctx, component_id).await? {
        let manager = Component::name_by_id(ctx, manager_id).await?;
        result.push(format!("[manager] {manager}",));
    }
    let component = Component::get_by_id(ctx, component_id).await?;
    for managed_id in component.get_managed(ctx).await? {
        let managed = Component::name_by_id(ctx, managed_id).await?;
        result.push(format!("[managed] {managed}"));
    }
    result.sort();
    Ok(result)
}
