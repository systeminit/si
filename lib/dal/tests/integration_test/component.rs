use dal::{
    AttributeValue,
    AttributeValueId,
    Component,
    DalContext,
    Schema,
    SchemaVariant,
    Ulid,
    attribute::value::DependentValueGraph,
    diagram::{
        Diagram,
        view::View,
    },
    prop::{
        Prop,
        PropPath,
    },
    property_editor::values::PropertyEditorValues,
    workspace_snapshot::DependentValueRoot,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        component,
        create_component_for_default_schema_name_in_default_view,
        create_component_for_schema_variant_on_default_view,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

mod autosubscribe;
mod connectable_test;
mod debug;
mod delete;
mod duplicate;
mod get_code;
mod get_diff;
mod paste;
mod property_order;
mod set_type;
mod upgrade;

#[test]
async fn update_and_insert_and_update(ctx: &mut DalContext) -> Result<()> {
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "Docker Image",
        "a tulip in a cup",
    )
    .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;

    let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;

    let image_prop_id =
        Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(["root", "domain", "image"]))
            .await?;

    let exposed_ports_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "ExposedPorts"]),
    )
    .await?;

    let exposed_ports_elem_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "ExposedPorts", "ExposedPort"]),
    )
    .await?;

    // Update image
    let image_av_id = property_values
        .find_by_prop_id(image_prop_id)
        .expect("can't find default attribute value for ExposedPorts");

    let image_value = serde_json::json!("fiona/apple");
    AttributeValue::update(ctx, image_av_id, Some(image_value.clone())).await?;

    let exposed_port_attribute_value_id = property_values
        .find_by_prop_id(exposed_ports_prop_id)
        .expect("can't find default attribute value for ExposedPorts");

    // Insert it unset first (to mimick frontend)
    let inserted_av_id =
        AttributeValue::insert(ctx, exposed_port_attribute_value_id, None, None).await?;

    // Before sending to the rebaser, confirm the value is there and it's the only one for the
    // ExposedPorts prop
    let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;

    let (fetched_image_value, image_av_id_again) = property_values
        .find_with_value_by_prop_id(image_prop_id)
        .expect("able to get image av id from pvalues");

    assert_eq!(image_av_id, image_av_id_again);
    assert_eq!(image_value, fetched_image_value);

    let mut inserted_attribute_values: Vec<AttributeValueId> =
        property_values.list_by_prop_id(exposed_ports_elem_prop_id);

    assert_eq!(1, inserted_attribute_values.len());
    let pvalues_inserted_attribute_value_id =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);

    // Rebase!
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    component.view(ctx).await?;

    // Confirm after rebase
    let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;

    let (fetched_image_value, image_av_id_again) = property_values
        .find_with_value_by_prop_id(image_prop_id)
        .expect("able to get image av id from pvalues");

    assert_eq!(image_av_id, image_av_id_again);
    assert_eq!(image_value, fetched_image_value);

    let mut inserted_attribute_values =
        property_values.list_with_values_by_prop_id(exposed_ports_elem_prop_id);
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, serde_json::Value::Null);

    let value = serde_json::json!("i ran out of white doves feathers");

    // Update the value we inserted
    AttributeValue::update(ctx, inserted_av_id, Some(value.clone())).await?;

    // Confirm again before rebase
    let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;

    let mut inserted_attribute_values =
        property_values.list_with_values_by_prop_id(exposed_ports_elem_prop_id);
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, value.clone());

    // Rebase again!
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let property_values = PropertyEditorValues::assemble(ctx, component.id()).await?;

    let mut inserted_attribute_values =
        property_values.list_with_values_by_prop_id(exposed_ports_elem_prop_id);
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, value.clone());

    Ok(())
}

#[test]
async fn create_and_determine_lineage(ctx: &DalContext) -> Result<()> {
    // List all schemas in the workspace. Pick the first one alphabetically.
    let mut schemas: Vec<Schema> = Schema::list(ctx).await?;
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    let schema = schemas.pop().expect("schemas are empty");

    // Ensure we can get it by id.
    let found_schema = Schema::get_by_id(ctx, schema.id()).await?;
    assert_eq!(
        schema.id(),       // expected
        found_schema.id()  // actual
    );

    // Pick a schema variant.
    let mut schema_variants = SchemaVariant::list_for_schema(ctx, found_schema.id()).await?;
    let schema_variant = schema_variants.pop().expect("schemas are empty");
    let schema_variant_id = schema_variant.id();

    // Create a component and set geometry.
    let mut component =
        create_component_for_schema_variant_on_default_view(ctx, schema_variant_id).await?;

    let default_view_id = View::get_id_for_default(ctx).await?;
    component
        .set_geometry(
            ctx,
            default_view_id,
            1isize,
            1isize,
            Some(500isize),
            Some(500isize),
        )
        .await?;

    // Determine the schema variant from the component. Ensure it is the same as before.
    let post_creation_schema_variant = component.schema_variant(ctx).await?;
    assert_eq!(
        schema_variant_id,                 // expected
        post_creation_schema_variant.id()  // actual
    );

    // Determine the schema from the schema variant. Ensure it is the same as before.
    let post_creation_schema = post_creation_schema_variant.schema(ctx).await?;
    assert_eq!(
        schema.id(),               // expected
        post_creation_schema.id()  // actual
    );

    // Assemble the diagram just to make sure it works.
    let _diagram = Diagram::assemble_for_default_view(ctx).await?;

    Ok(())
}

#[test]
async fn through_the_wormholes_simple(ctx: &mut DalContext) -> Result<()> {
    let name = "across the universe";
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", name).await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let rigid_designator_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new([
            "root",
            "domain",
            "possible_world_a",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "rigid_designator",
        ]),
    )
    .await?;

    let rigid_designator_values =
        Component::attribute_values_for_prop_id(ctx, component.id(), rigid_designator_prop_id)
            .await?;

    assert_eq!(1, rigid_designator_values.len());

    let rigid_designator_value_id = rigid_designator_values
        .first()
        .copied()
        .expect("get first value id");

    assert_eq!(
        component.id(),
        AttributeValue::component_id(ctx, rigid_designator_value_id).await?
    );

    let naming_and_necessity_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new([
            "root",
            "domain",
            "possible_world_b",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "naming_and_necessity",
        ]),
    )
    .await?;

    let naming_and_necessity_value_id =
        Component::attribute_values_for_prop_id(ctx, component.id(), naming_and_necessity_prop_id)
            .await?
            .first()
            .copied()
            .expect("get first value id");

    let update_graph = DependentValueGraph::new(
        ctx,
        vec![DependentValueRoot::Unfinished(
            rigid_designator_value_id.into(),
        )],
    )
    .await?;

    assert!(
        update_graph.contains_value(naming_and_necessity_value_id),
        "update graph has the value we aren't setting but which depends on the value we are setting"
    );

    assert!(
        update_graph
            .direct_dependencies_of(naming_and_necessity_value_id)
            .contains(&rigid_designator_value_id),
        "update graph declares that `naming_and_necessity` value depends on `rigid_designator` value"
    );

    let rigid_designation = serde_json::json!("hesperus");

    AttributeValue::update(
        ctx,
        rigid_designator_value_id,
        Some(rigid_designation.to_owned()),
    )
    .await?;

    let view = AttributeValue::view(ctx, rigid_designator_value_id)
        .await?
        .expect("has a view");

    assert_eq!(rigid_designation, view);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let naming_and_necessity_view = AttributeValue::view(ctx, naming_and_necessity_value_id)
        .await?
        .expect("naming and necessity has a value");

    // hesperus is phosphorus (the attr func on naming_and_necessity_value_id will return
    // phosphorus if it receives hesperus)
    assert_eq!("phosphorus", naming_and_necessity_view);

    let root_prop_id =
        Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(["root"])).await?;

    let root_value_id = Component::attribute_values_for_prop_id(ctx, component.id(), root_prop_id)
        .await?
        .first()
        .copied()
        .expect("a value exists for the root prop");

    let root_view = AttributeValue::view(ctx, root_value_id)
        .await?
        .expect("there is a value for the root value view");

    assert_eq!(
        serde_json::json!({
                "si": { "name": name, "color": "#ffffff", "type": "component" },
                "resource_value": {},
                "domain": {
                    "name": name,
                    "possible_world_a": {
                        "wormhole_1": {
                            "wormhole_2": {
                                "wormhole_3": {
                                    "rigid_designator": rigid_designation
                                }
                            }
                        }
                    },
                    "possible_world_b": {
                        "wormhole_1": {
                            "wormhole_2": {
                                "wormhole_3": {
                                    "naming_and_necessity": "phosphorus"
                                }
                            }
                        }
                    },
                    "universe": { "galaxies": [] },
                }
            }
        ),
        root_view
    );

    Ok(())
}

#[test]
async fn through_the_wormholes_child_value_reactivity(ctx: &mut DalContext) -> Result<()> {
    let name = "across the universe";
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", name).await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let possible_world_a_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "possible_world_a"]),
    )
    .await?;

    let possible_world_values =
        Component::attribute_values_for_prop_id(ctx, component.id(), possible_world_a_prop_id)
            .await
            .expect("able to get attribute value for universe prop");

    assert_eq!(1, possible_world_values.len());

    let possible_world_a_value_id = possible_world_values
        .first()
        .copied()
        .expect("get first value id");

    assert_eq!(
        component.id(),
        AttributeValue::component_id(ctx, possible_world_a_value_id).await?
    );

    let naming_and_necessity_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new([
            "root",
            "domain",
            "possible_world_b",
            "wormhole_1",
            "wormhole_2",
            "wormhole_3",
            "naming_and_necessity",
        ]),
    )
    .await?;

    let naming_and_necessity_value_id =
        Component::attribute_values_for_prop_id(ctx, component.id(), naming_and_necessity_prop_id)
            .await?
            .first()
            .copied()
            .expect("get first value id");

    let update_graph = DependentValueGraph::new(
        ctx,
        vec![DependentValueRoot::Unfinished(
            possible_world_a_value_id.into(),
        )],
    )
    .await?;

    assert!(
        update_graph.contains_value(naming_and_necessity_value_id),
        "update graph has the value we aren't setting but which depends on the value we are setting"
    );

    let possible_world_a = serde_json::json!({
        "wormhole_1": {
            "wormhole_2": {
                "wormhole_3": {
                    "rigid_designator": "hesperus"
                }
            }
        }
    });

    AttributeValue::update(
        ctx,
        possible_world_a_value_id,
        Some(possible_world_a.clone()),
    )
    .await?;

    let view = AttributeValue::view(ctx, possible_world_a_value_id)
        .await?
        .expect("has a view");

    assert_eq!(possible_world_a, view);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let naming_and_necessity_view = AttributeValue::view(ctx, naming_and_necessity_value_id)
        .await?
        .expect("naming and necessity has a value");

    // hesperus is phosphorus (the attr func on naming_and_necessity_value_id will return
    // phosphorus if it receives hesperus)
    assert_eq!("phosphorus", naming_and_necessity_view);

    let root_prop_id =
        Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(["root"])).await?;

    let root_value_id = Component::attribute_values_for_prop_id(ctx, component.id(), root_prop_id)
        .await?
        .first()
        .copied()
        .expect("a value exists for the root prop");

    let root_view = AttributeValue::view(ctx, root_value_id)
        .await?
        .expect("there is a value for the root value view");

    assert_eq!(
        serde_json::json!({
                "si": { "name": name, "color": "#ffffff", "type": "component" },
                "resource_value": {},
                "domain": {
                    "name": name,
                    "possible_world_a": possible_world_a,
                    "possible_world_b": {
                        "wormhole_1": {
                            "wormhole_2": {
                                "wormhole_3": {
                                    "naming_and_necessity": "phosphorus"
                                }
                            }
                        }
                    },
                    "universe": { "galaxies": [] },
                }
            }
        ),
        root_view
    );

    Ok(())
}

#[test]
async fn set_the_universe(ctx: &mut DalContext) -> Result<()> {
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "starfield",
        "across the universe",
    )
    .await?;
    let variant_id = Component::schema_variant_id(ctx, component.id()).await?;

    let universe_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "universe"]),
    )
    .await?;

    let universe_values =
        Component::attribute_values_for_prop_id(ctx, component.id(), universe_prop_id).await?;

    assert_eq!(1, universe_values.len());

    let universe_value_id = universe_values
        .first()
        .copied()
        .expect("get first value id");

    assert_eq!(
        component.id(),
        AttributeValue::component_id(ctx, universe_value_id).await?
    );

    let universe_json = serde_json::json!({
        "galaxies": [
            { "sun": "sol", "planets": 9 },
            { "sun": "champagne supernova", "planets": 9000 },
            { "sun": "black hole", "planets": 0 }
        ]
    });

    AttributeValue::update(ctx, universe_value_id, Some(universe_json.to_owned())).await?;

    let view = AttributeValue::view(ctx, universe_value_id)
        .await?
        .expect("has a view");

    assert_eq!(universe_json, view);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let view = AttributeValue::view(ctx, universe_value_id)
        .await?
        .expect("has a view");

    assert_eq!(universe_json, view);

    Ok(())
}

#[test]
#[ignore = "this test does not pass on the legacy snapshot because id is not included in the node hash"]
async fn change_component_id(ctx: &mut DalContext) -> Result<()> {
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "starfield",
        "across the universe",
    )
    .await?;

    let current_component_id = component.id();
    let current_lineage_id = ctx
        .workspace_snapshot()?
        .get_node_weight(current_component_id)
        .await
        .expect("node weight exists")
        .lineage_id();

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let new_component_id = Ulid::new();
    ctx.workspace_snapshot()?
        .update_node_id(current_component_id, new_component_id, current_lineage_id)
        .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert!(
        Component::try_get_by_id(ctx, current_component_id)
            .await?
            .is_none()
    );
    assert!(
        Component::try_get_by_id(ctx, new_component_id.into())
            .await?
            .is_some()
    );

    Ok(())
}

#[test]
async fn default_values_appear_in_views(ctx: &mut DalContext) -> Result<()> {
    let _subscription_tracker = variant::create(
        ctx,
        "default_values",
        r#"
            function main() {
    // Props
    const labelsProp = new PropBuilder()
        .setName("labels")
        .setKind("string")
        .setHidden(false)
        .setWidget(new PropWidgetDefinitionBuilder()
            .setKind("text")
            .build())
        .setValidationFormat(Joi.string().required())
        .setDocumentation("User-defined labels (`key/value` pairs) for the Resource.\nFor more information, see \"[Labels](#labels)\".\n")
        .build();

    const nameProp = new PropBuilder()
        .setName("name")
        .setKind("string")
        .setHidden(false)
        .setDefaultValue("default-name")
        .setWidget(new PropWidgetDefinitionBuilder()
            .setKind("text")
            .build())
        .setValidationFormat(Joi.string().required())
        .setDocumentation("Name of the Resource. Must be unique per Project.")
        .build();

    const extraProp = new PropBuilder()
        .setName("extra")
        .setKind("object")
        .setHidden(false)
        .setWidget(new PropWidgetDefinitionBuilder()
            .setKind("header")
            .build())
        .addChild(
            new PropBuilder()
            .setName("endpoint")
            .setKind("string")
            .setHidden(false)
            .setWidget(new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build())
            .setDefaultValue("ssh_keys")
            .build()
        )
        .addChild(
            new PropBuilder()
            .setName("PropUsageMap")
            .setKind("string")
            .setHidden(true)
            .setWidget(new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build())
            .setDefaultValue("{\"createOnly\":[\"public_key\"],\"updatable\":[\"labels\",\"name\"]}")
            .build()
        )
        .build();

    return new AssetBuilder()
        .addProp(labelsProp)
        .addProp(nameProp)
        .addProp(extraProp)
        .build();
}
        "#,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let component_id = component::create(ctx, "default_values", "A").await?;
    let component = Component::get_by_id(ctx, component_id).await?;
    let view = component.view(ctx).await?.expect("has a view");
    assert_eq!(
        serde_json::json!({
            "si": { "name": "A", "type": "component","color": "FFFFFF"  },
            "domain": {
                "name": "default-name",
                "extra": {
                    "endpoint": "ssh_keys",
                    "PropUsageMap": "{\"createOnly\":[\"public_key\"],\"updatable\":[\"labels\",\"name\"]}"
                },
            }
        }),
        view
    );
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let component = Component::get_by_id(ctx, component_id).await?;
    let view = component.view(ctx).await?.expect("has a view");
    // after dvu, the attribute tree is vivified which changes the view
    assert_eq!(
        serde_json::json!({
            "si": { "name": "A", "type": "component","color": "FFFFFF"  },
            "domain": {
                "name": "default-name",
                "extra": {
                    "endpoint": "ssh_keys",
                    "PropUsageMap": "{\"createOnly\":[\"public_key\"],\"updatable\":[\"labels\",\"name\"]}"
                },
            },
            "resource_value": serde_json::json!({}),
        }),
        view
    );
    Ok(())
}
