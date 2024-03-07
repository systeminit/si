use dal::attribute::value::DependentValueGraph;
use dal::component::{DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::diagram::Diagram;
use dal::prop::{Prop, PropPath};
use dal::property_editor::values::PropertyEditorValues;
use dal::{AttributeValue, AttributeValueId, ComponentType};
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::test;
use dal_test::test_harness::create_component_for_schema_name;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn update_and_insert_and_update(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "Docker Image", "a tulip in a cup").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");

    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let image_prop_id =
        Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(["root", "domain", "image"]))
            .await
            .expect("able to find image prop");

    let exposed_ports_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "ExposedPorts"]),
    )
    .await
    .expect("able to find exposed ports prop");

    let exposed_ports_elem_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "ExposedPorts", "ExposedPort"]),
    )
    .await
    .expect("able to find exposed ports element prop");

    // Update image
    let image_av_id = property_values
        .find_by_prop_id(image_prop_id)
        .expect("can't find default attribute value for ExposedPorts");

    let image_value = serde_json::json!("fiona/apple");
    AttributeValue::update(ctx, image_av_id, Some(image_value.clone()))
        .await
        .expect("able to update image prop with 'fiona/apple'");

    let exposed_port_attribute_value_id = property_values
        .find_by_prop_id(exposed_ports_prop_id)
        .expect("can't find default attribute value for ExposedPorts");

    // Insert it unset first (to mimick frontend)
    let inserted_av_id = AttributeValue::insert(ctx, exposed_port_attribute_value_id, None, None)
        .await
        .expect("able to insert");

    // Before sending to the rebaser, confirm the value is there and it's the only one for the
    // ExposedPorts prop
    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

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
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    dbg!(component
        .materialized_view(ctx)
        .await
        .expect("materialized_view for component"));

    // Confirm after rebase
    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

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
    AttributeValue::update(ctx, inserted_av_id, Some(value.clone()))
        .await
        .expect("able to update");

    // Confirm again before rebase
    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let mut inserted_attribute_values =
        property_values.list_with_values_by_prop_id(exposed_ports_elem_prop_id);
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, value.clone());

    // Rebase again!
    let conflicts = ctx.commit().await.expect("unable to commit");
    assert!(conflicts.is_none());

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let property_values = PropertyEditorValues::assemble(ctx, component.id())
        .await
        .expect("able to list prop values");

    let mut inserted_attribute_values =
        property_values.list_with_values_by_prop_id(exposed_ports_elem_prop_id);
    assert_eq!(1, inserted_attribute_values.len());
    let (inserted_value, pvalues_inserted_attribute_value_id) =
        inserted_attribute_values.pop().expect("get our av id");
    assert_eq!(inserted_av_id, pvalues_inserted_attribute_value_id);
    assert_eq!(inserted_value, value.clone());
}

#[test]
async fn create_and_determine_lineage(ctx: &DalContext) {
    // List all schemas in the workspace. Pick the first one alphabetically.
    let mut schemas: Vec<Schema> = Schema::list(ctx).await.expect("could not list schemas");
    schemas.sort_by(|a, b| a.name.cmp(&b.name));
    let schema = schemas.pop().expect("schemas are empty");

    // Ensure we can get it by id.
    let found_schema = Schema::get_by_id(ctx, schema.id())
        .await
        .expect("could not get schema by id");
    assert_eq!(
        schema.id(),       // expected
        found_schema.id()  // actual
    );

    // Pick a schema variant.
    let mut schema_variants = SchemaVariant::list_for_schema(ctx, found_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let schema_variant = schema_variants.pop().expect("schemas are empty");
    let schema_variant_id = schema_variant.id();

    // Create a component and set geometry.
    let name = "fsu not top four";
    let component = Component::new(ctx, name, schema_variant_id)
        .await
        .expect("could not create component");
    let component = component
        .set_geometry(
            ctx,
            "1",
            "-1",
            Some(DEFAULT_COMPONENT_WIDTH),
            Some(DEFAULT_COMPONENT_HEIGHT),
        )
        .await
        .expect("could not set geometry");

    // Determine the schema variant from the component. Ensure it is the same as before.
    let post_creation_schema_variant = component
        .schema_variant(ctx)
        .await
        .expect("could not get schema variant for component");
    assert_eq!(
        schema_variant_id,                 // expected
        post_creation_schema_variant.id()  // actual
    );

    // Determine the schema from the schema variant. Ensure it is the same as before.
    let post_creation_schema = SchemaVariant::schema(ctx, post_creation_schema_variant.id())
        .await
        .expect("could not get schema for schema variant");
    assert_eq!(
        schema.id(),               // expected
        post_creation_schema.id()  // actual
    );

    // Assemble the diagram just to make sure it works.
    let _diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
}

#[test]
async fn through_the_wormholes(ctx: &mut DalContext) {
    let name = "across the universe";
    let component = create_component_for_schema_name(ctx, "starfield", name).await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

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
    .await
    .expect("able to find 'rigid_designator' prop");

    let rigid_designator_values = Prop::attribute_values_for_prop_id(ctx, rigid_designator_prop_id)
        .await
        .expect("able to get attribute value for universe prop");

    assert_eq!(1, rigid_designator_values.len());

    let rigid_designator_value_id = rigid_designator_values
        .first()
        .copied()
        .expect("get first value id");

    assert_eq!(
        component.id(),
        AttributeValue::component_id(ctx, rigid_designator_value_id)
            .await
            .expect("able to get component id for universe value")
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
    .await
    .expect("able to find 'naming_and_necessity' prop");

    let naming_and_necessity_value_id =
        Prop::attribute_values_for_prop_id(ctx, naming_and_necessity_prop_id)
            .await
            .expect("able to get values for naming_and_necessity")
            .first()
            .copied()
            .expect("get first value id");

    let update_graph = DependentValueGraph::for_values(ctx, vec![rigid_designator_value_id])
        .await
        .expect("able to generate update graph");

    assert!(
        update_graph.contains_value(naming_and_necessity_value_id),
        "update graph has the value we aren't setting but which depends on the value we are setting"
    );

    assert!(update_graph
        .direct_dependencies_of(naming_and_necessity_value_id)
        .iter()
        .any(|&id| id == rigid_designator_value_id),
        "update graph declares that `naming_and_necessity` value depends on `rigid_designator` value"
    );

    let rigid_designation = serde_json::json!("hesperus");

    AttributeValue::update(
        ctx,
        rigid_designator_value_id,
        Some(rigid_designation.to_owned()),
    )
    .await
    .expect("able to set universe value");

    let materialized_view = AttributeValue::get_by_id(ctx, rigid_designator_value_id)
        .await
        .expect("get av")
        .materialized_view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(rigid_designation, materialized_view);

    ctx.blocking_commit().await.expect("commit");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let naming_and_necessity_view = AttributeValue::get_by_id(ctx, naming_and_necessity_value_id)
        .await
        .expect("able to get attribute value for `naming_and_necessity_value_id`")
        .materialized_view(ctx)
        .await
        .expect("able to get materialized_view for `naming_and_necessity_value_id`")
        .expect("naming and necessity has a value");

    // hesperus is phosphorus (the attr func on naming_and_necessity_value_id will return
    // phosphorus if it receives hesperus)
    assert_eq!("phosphorus", naming_and_necessity_view);

    let root_prop_id = Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(["root"]))
        .await
        .expect("able to find root prop");

    let root_value_id = Prop::attribute_values_for_prop_id(ctx, root_prop_id)
        .await
        .expect("get root prop value id")
        .first()
        .copied()
        .expect("a value exists for the root prop");

    let root_value = AttributeValue::get_by_id(ctx, root_value_id)
        .await
        .expect("able to get the value for the root prop attriburte value id");

    let root_view = root_value
        .materialized_view(ctx)
        .await
        .expect("able to fetch materialized_view for root value")
        .expect("there is a value for the root value materialized_view");

    assert_eq!(
        serde_json::json!({
                "si": { "name": name, "color": "#ffffff", "type": "component" },
                "resource": {},
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
}
#[test]
async fn set_the_universe(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "starfield", "across the universe").await;
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");

    ctx.blocking_commit()
        .await
        .expect("blocking commit after component creation");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("update_snapshot_to_visibility");

    let universe_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "universe"]),
    )
    .await
    .expect("able to find 'root/domain/universe' prop");

    let universe_values = Prop::attribute_values_for_prop_id(ctx, universe_prop_id)
        .await
        .expect("able to get attribute value for universe prop");

    assert_eq!(1, universe_values.len());

    let universe_value_id = universe_values
        .first()
        .copied()
        .expect("get first value id");

    assert_eq!(
        component.id(),
        AttributeValue::component_id(ctx, universe_value_id)
            .await
            .expect("able to get component id for universe value")
    );

    let universe_json = serde_json::json!({
        "galaxies": [
            { "sun": "sol", "planets": 9 },
            { "sun": "champagne supernova", "planets": 9000 },
            { "sun": "black hole", "planets": 0 }
        ]
    });

    AttributeValue::update(ctx, universe_value_id, Some(universe_json.to_owned()))
        .await
        .expect("able to set universe value");

    let materialized_view = AttributeValue::get_by_id(ctx, universe_value_id)
        .await
        .expect("get av")
        .materialized_view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(universe_json, materialized_view);

    ctx.blocking_commit().await.expect("commit");

    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visiblity");

    let materialized_view = AttributeValue::get_by_id(ctx, universe_value_id)
        .await
        .expect("get av")
        .materialized_view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(universe_json, materialized_view);
}

#[test]
async fn set_type(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(ctx, "starfield", "black star").await;

    assert_eq!(
        component.get_type(ctx).await.expect("could not get type"),
        ComponentType::Component
    );

    component
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");

    assert_eq!(
        component.get_type(ctx).await.expect("could not get type"),
        ComponentType::ConfigurationFrameUp
    );
}
