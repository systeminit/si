use dal::action::prototype::{ActionKind, ActionPrototype};
use dal::action::Action;
use dal::component::frame::Frame;
use dal::diagram::Diagram;
use dal::func::authoring::FuncAuthoringClient;
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{
    AttributeValue, Component, ComponentType, DalContext, Prop, Schema, SchemaVariant,
    SchemaVariantId,
};
use dal_test::helpers::{
    create_component_for_default_schema_name_in_default_view, ChangeSetTestHelpers,
    PropEditorTestView,
};
use dal_test::{test, Result};
use itertools::Itertools;
use pretty_assertions_sorted::{assert_eq, assert_ne};
use serde_json::json;
use std::collections::VecDeque;
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
        .await
        .unwrap()
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
    let variant_id = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "variant",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
    )
    .await?
    .id;
    update_schema_variant_component_type(ctx, variant_id, ComponentType::ConfigurationFrameDown)
        .await?;
    assert_eq!(
        Some(ComponentType::ConfigurationFrameDown),
        SchemaVariant::get_type(ctx, variant_id).await?
    );

    //
    // Create a new component from the variant, with child
    //
    let component_id =
        create_component_for_default_schema_name_in_default_view(ctx, "variant", "component")
            .await?
            .id();
    let child_id =
        create_component_for_default_schema_name_in_default_view(ctx, "Docker Image", "child")
            .await?
            .id();
    Frame::upsert_parent(ctx, child_id, component_id).await?;

    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        Component::get_type_by_id(ctx, component_id).await?
    );

    //
    // Update the variant to be a Component that can't have parents
    //
    update_schema_variant_component_type(ctx, variant_id, ComponentType::Component).await?;
    assert_eq!(
        Some(ComponentType::Component),
        SchemaVariant::get_type(ctx, variant_id).await?
    );

    //
    // Check that the component is upgraded but is still set to ConfigurationFrameDown
    //
    assert_eq!(
        ComponentType::ConfigurationFrameDown,
        Component::get_type_by_id(ctx, component_id).await?
    );

    Ok(())
}

#[test]
async fn upgrade_component_type_after_explicit_set(ctx: &mut DalContext) -> Result<()> {
    //
    // Create a new schema and variant set to component_type: ConfigurationFrameDown
    //
    let variant_id = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "variant",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
    )
    .await?
    .id;
    update_schema_variant_component_type(ctx, variant_id, ComponentType::ConfigurationFrameDown)
        .await?;
    assert_eq!(
        Some(ComponentType::ConfigurationFrameDown),
        SchemaVariant::get_type(ctx, variant_id).await?
    );

    //
    // Create a new component from the variant, and set its type to Component
    //
    let component_id =
        create_component_for_default_schema_name_in_default_view(ctx, "variant", "component")
            .await?
            .id();
    Component::set_type_by_id(ctx, component_id, ComponentType::Component).await?;

    assert_eq!(
        ComponentType::Component,
        Component::get_type_by_id(ctx, component_id).await?
    );

    //
    // Update the variant (we add a new description)
    //
    update_schema_variant_description(ctx, variant_id, "eek").await?;
    assert_eq!(
        Some(ComponentType::ConfigurationFrameDown),
        SchemaVariant::get_type(ctx, variant_id).await?
    );

    //
    // Check that the component is upgraded but is still set to ConfigurationFrameDown
    //
    assert_eq!(
        ComponentType::Component,
        Component::get_type_by_id(ctx, component_id).await?
    );

    Ok(())
}

#[test]
async fn create_unlocked_schema_variant_after_component_changes_component_type(
    ctx: &mut DalContext,
) -> Result<()> {
    let swifty_id = SchemaVariant::default_id_for_schema_name(ctx, "swifty").await?;
    assert_eq!(
        Some(ComponentType::ConfigurationFrameUp),
        SchemaVariant::get_type(ctx, swifty_id).await?
    );

    //
    // Create a new component from the variant, and set its type to Component
    //
    let component_id =
        create_component_for_default_schema_name_in_default_view(ctx, "swifty", "component")
            .await?
            .id();
    Component::set_type_by_id(ctx, component_id, ComponentType::Component).await?;

    assert_eq!(
        swifty_id,
        Component::schema_variant_id(ctx, component_id).await?
    );
    assert_eq!(
        ComponentType::Component,
        Component::get_type_by_id(ctx, component_id).await?
    );
    assert_eq!(
        Some(ComponentType::ConfigurationFrameUp),
        SchemaVariant::get_type(ctx, swifty_id).await?
    );

    //
    // Update the variant (we add a new description)
    //
    let copy_id = VariantAuthoringClient::create_unlocked_variant_copy(ctx, swifty_id)
        .await?
        .id;

    assert_ne!(swifty_id, copy_id);
    assert_eq!(
        Some(ComponentType::ConfigurationFrameUp),
        SchemaVariant::get_type(ctx, swifty_id).await?
    );
    assert_eq!(
        Some(ComponentType::ConfigurationFrameUp),
        SchemaVariant::get_type(ctx, copy_id).await?
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
            .get_value(&value_path)?;

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
async fn upgrade_frame_with_child(ctx: &mut DalContext) -> Result<()> {
    let frame_original_code_definition = r#"
        function main() {
            const theProp = new PropBuilder()
                .setName("The Prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();
            const outputSocket = new SocketDefinitionBuilder()
                .setName("Socket Data")
                .setArity("one")
                .setValueFrom(
                    new ValueFromBuilder()
                        .setKind("prop")
                        .setPropPath(["root", "domain", "The Prop"])
                        .build()
                )
                .build();

            return new AssetBuilder()
                .addProp(theProp)
                .addOutputSocket(outputSocket)
                .build()
        }
    "#;
    let original_frame_variant_id = VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        "Initial Variant",
        None,
        None,
        "Category name",
        "#0000ff",
        frame_original_code_definition,
    )
    .await?
    .id;
    update_schema_variant_component_type(
        ctx,
        original_frame_variant_id,
        ComponentType::ConfigurationFrameDown,
    )
    .await?;

    let component_code_definition = r#"
        function main() {
            const theProp = new PropBuilder()
                .setName("The Prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .setValueFrom(
                    new ValueFromBuilder()
                        .setKind("inputSocket")
                        .setSocketName("Socket Data")
                        .build()
                )
                .build();

            const inputSocket = new SocketDefinitionBuilder()
                .setName("Socket Data")
                .setArity("one")
                .build();

            return new AssetBuilder()
                .addProp(theProp)
                .addInputSocket(inputSocket)
                .build();
        }
    "#;

    VariantAuthoringClient::create_schema_and_variant_from_code(
        ctx,
        "Child Variant",
        None,
        None,
        "Another Category",
        "#0077cc",
        component_code_definition,
    )
    .await?;

    let frame_component_id = create_component_for_default_schema_name_in_default_view(
        ctx,
        "Initial Variant",
        "frame_component",
    )
    .await?
    .id();
    let child_component_id = create_component_for_default_schema_name_in_default_view(
        ctx,
        "Child Variant",
        "child_component",
    )
    .await?
    .id();
    Frame::upsert_parent(ctx, child_component_id, frame_component_id).await?;

    let inferred_connections =
        Component::inferred_incoming_connections(ctx, child_component_id).await?;

    assert_eq!(1, inferred_connections.len());
    let inferred_connection = inferred_connections
        .first()
        .expect("Unable to get first element of a single element Vec.");
    assert_eq!(frame_component_id, inferred_connection.from_component_id,);

    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    let change_set = ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
        .await?;

    let updated_frame_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, original_frame_variant_id)
            .await?
            .id;

    let updated_frame_code_definition = r#"
        function main() {
            const theProp = new PropBuilder()
                .setName("The Prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();
            const outputSocket = new SocketDefinitionBuilder()
                .setName("Socket Data")
                .setArity("one")
                .setValueFrom(
                    new ValueFromBuilder()
                        .setKind("prop")
                        .setPropPath(["root", "domain", "The Prop"])
                        .build()
                )
                .build();

            const anotherProp = new PropBuilder()
                .setName("Another Prop")
                .setKind("string")
                .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                .build();

            return new AssetBuilder()
                .addProp(theProp)
                .addProp(anotherProp)
                .addOutputSocket(outputSocket)
                .build()
        }

    "#;
    let frame_variant = SchemaVariant::get_by_id(ctx, original_frame_variant_id).await?;
    VariantAuthoringClient::save_variant_content(
        ctx,
        updated_frame_variant_id,
        frame_variant.schema(ctx).await?.name.clone(),
        frame_variant.display_name(),
        frame_variant.category(),
        frame_variant.description(),
        frame_variant.link(),
        frame_variant.get_color(ctx).await?,
        frame_variant.component_type(),
        Some(updated_frame_code_definition),
    )
    .await?;

    let regenerated_frame_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, updated_frame_variant_id).await?;

    assert_ne!(original_frame_variant_id, regenerated_frame_variant_id);
    assert_eq!(updated_frame_variant_id, regenerated_frame_variant_id);

    let upgraded_frame_component = Component::get_by_id(ctx, frame_component_id)
        .await?
        .upgrade_to_new_variant(ctx, regenerated_frame_variant_id)
        .await?;

    let inferred_connections =
        Component::inferred_incoming_connections(ctx, child_component_id).await?;

    assert_eq!(1, inferred_connections.len());
    let inferred_connection = inferred_connections
        .first()
        .expect("Unable to get first element of a single element Vec.");
    assert_eq!(
        upgraded_frame_component.id(),
        inferred_connection.from_component_id
    );

    Ok(())
}

async fn update_schema_variant_component_type(
    ctx: &DalContext,
    variant_id: SchemaVariantId,
    component_type: ComponentType,
) -> Result<()> {
    let variant = SchemaVariant::get_by_id(ctx, variant_id).await?;
    VariantAuthoringClient::save_variant_content(
        ctx,
        variant_id,
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
    Ok(())
}

async fn update_schema_variant_description(
    ctx: &DalContext,
    variant_id: SchemaVariantId,
    description: impl Into<String>,
) -> Result<()> {
    let variant = SchemaVariant::get_by_id(ctx, variant_id).await?;
    Ok(VariantAuthoringClient::save_variant_content(
        ctx,
        variant_id,
        "test schema",
        variant.display_name(),
        variant.category(),
        Some(description.into()),
        variant.link(),
        variant.color(),
        variant.component_type(),
        None as Option<String>,
    )
    .await?)
}
