use dal::attribute::value::DependentValueGraph;
use dal::component::{ComponentGeometry, DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::diagram::Diagram;
use dal::prop::{Prop, PropPath};
use dal::property_editor::values::PropertyEditorValues;
use dal::{AttributeValue, AttributeValueId};
use dal::{Component, DalContext, Schema, SchemaVariant};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::helpers::{
    connect_components_with_socket_names, create_component_for_default_schema_name,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

mod debug;
mod delete;
mod get_code;
mod get_diff;
mod set_type;
mod upgrade;

#[test]
async fn update_and_insert_and_update(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name(ctx, "Docker Image", "a tulip in a cup")
            .await
            .expect("could not create component");
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
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    component.view(ctx).await.expect("view for component");

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
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

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
    let mut component = Component::new(ctx, name, schema_variant_id)
        .await
        .expect("could not create component");
    component
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
    let post_creation_schema = post_creation_schema_variant
        .schema(ctx)
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
async fn through_the_wormholes_simple(ctx: &mut DalContext) {
    let name = "across the universe";
    let component = create_component_for_default_schema_name(ctx, "starfield", name)
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

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

    let update_graph = DependentValueGraph::new(ctx, vec![rigid_designator_value_id])
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

    let view = AttributeValue::get_by_id_or_error(ctx, rigid_designator_value_id)
        .await
        .expect("get av")
        .view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(rigid_designation, view);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let naming_and_necessity_view =
        AttributeValue::get_by_id_or_error(ctx, naming_and_necessity_value_id)
            .await
            .expect("able to get attribute value for `naming_and_necessity_value_id`")
            .view(ctx)
            .await
            .expect("able to get view for `naming_and_necessity_value_id`")
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

    let root_value = AttributeValue::get_by_id_or_error(ctx, root_value_id)
        .await
        .expect("able to get the value for the root prop attriburte value id");

    let root_view = root_value
        .view(ctx)
        .await
        .expect("able to fetch view for root value")
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
}

#[test]
async fn through_the_wormholes_child_value_reactivity(ctx: &mut DalContext) {
    let name = "across the universe";
    let component = create_component_for_default_schema_name(ctx, "starfield", name)
        .await
        .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let possible_world_a_prop_id = Prop::find_prop_id_by_path(
        ctx,
        variant_id,
        &PropPath::new(["root", "domain", "possible_world_a"]),
    )
    .await
    .expect("able to find 'possible_world' prop");

    let possible_world_values = Prop::attribute_values_for_prop_id(ctx, possible_world_a_prop_id)
        .await
        .expect("able to get attribute value for universe prop");

    assert_eq!(1, possible_world_values.len());

    let possible_world_a_value_id = possible_world_values
        .first()
        .copied()
        .expect("get first value id");

    assert_eq!(
        component.id(),
        AttributeValue::component_id(ctx, possible_world_a_value_id)
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

    let update_graph = DependentValueGraph::new(ctx, vec![possible_world_a_value_id])
        .await
        .expect("able to generate update graph");

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
    .await
    .expect("able to set universe value");

    let view = AttributeValue::get_by_id_or_error(ctx, possible_world_a_value_id)
        .await
        .expect("get av")
        .view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(possible_world_a, view);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let naming_and_necessity_view =
        AttributeValue::get_by_id_or_error(ctx, naming_and_necessity_value_id)
            .await
            .expect("able to get attribute value for `naming_and_necessity_value_id`")
            .view(ctx)
            .await
            .expect("able to get view for `naming_and_necessity_value_id`")
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

    let root_value = AttributeValue::get_by_id_or_error(ctx, root_value_id)
        .await
        .expect("able to get the value for the root prop attriburte value id");

    let root_view = root_value
        .view(ctx)
        .await
        .expect("able to fetch view for root value")
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
}

#[test]
async fn through_the_wormholes_dynamic_child_value_reactivity(ctx: &mut DalContext) {
    let etoiles_name = "À la belle étoile";
    let etoiles_component = create_component_for_default_schema_name(ctx, "etoiles", etoiles_name)
        .await
        .expect("could not create component");
    let etoiles_variant_id = Component::schema_variant_id(ctx, etoiles_component.id())
        .await
        .expect("find variant id for etoiles component");
    let morningstar_name = "hesperus is phosphorus";
    let morningstar_component =
        create_component_for_default_schema_name(ctx, "morningstar", morningstar_name)
            .await
            .expect("could not create component");
    let morningstar_variant_id = Component::schema_variant_id(ctx, morningstar_component.id())
        .await
        .expect("find variant id for morningstar component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let possible_world_a_prop_id = Prop::find_prop_id_by_path(
        ctx,
        etoiles_variant_id,
        &PropPath::new(["root", "domain", "possible_world_a"]),
    )
    .await
    .expect("able to find 'possible_world_a' prop");

    let possible_world_values = Prop::attribute_values_for_prop_id(ctx, possible_world_a_prop_id)
        .await
        .expect("able to get attribute value for universe prop");

    let possible_world_a_value_id = possible_world_values
        .first()
        .copied()
        .expect("get first value id");

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
    .await
    .expect("able to set universe value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let possible_world_b_prop_id = Prop::find_prop_id_by_path(
        ctx,
        etoiles_variant_id,
        &PropPath::new(["root", "domain", "possible_world_b"]),
    )
    .await
    .expect("able to find 'possible_world_b' prop");

    let possible_world_values = Prop::attribute_values_for_prop_id(ctx, possible_world_b_prop_id)
        .await
        .expect("able to get attribute value for possible world prop");

    let possible_world_b_value_id = possible_world_values
        .first()
        .copied()
        .expect("get first value id");

    let value = AttributeValue::get_by_id_or_error(ctx, possible_world_b_value_id)
        .await
        .expect("able to get av by id");

    let possible_world_b = serde_json::json!({
        "wormhole_1": {
            "wormhole_2": {
                "wormhole_3": {
                    "rigid_designator": "phosphorus"
                }
            }
        }
    });

    assert_eq!(
        Some(possible_world_b),
        value.view(ctx).await.expect("able to get view")
    );

    connect_components_with_socket_names(
        ctx,
        etoiles_component.id(),
        "naming_and_necessity",
        morningstar_component.id(),
        "naming_and_necessity",
    )
    .await
    .expect("could not connect components with socket names");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let stars_prop_id = Prop::find_prop_id_by_path(
        ctx,
        morningstar_variant_id,
        &PropPath::new(["root", "domain", "stars"]),
    )
    .await
    .expect("able to find 'stars' prop");

    let stars_value_id = Prop::attribute_values_for_prop_id(ctx, stars_prop_id)
        .await
        .expect("able to get attribute value for possible world prop")
        .first()
        .copied()
        .expect("get first value id");

    let stars_value = AttributeValue::get_by_id_or_error(ctx, stars_value_id)
        .await
        .expect("able to get av by id");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    assert_eq!(
        Some(serde_json::to_value("phosphorus").expect("able to make phosphorus value")),
        stars_value.view(ctx).await.expect("get stars value")
    );
}

#[test]
async fn set_the_universe(ctx: &mut DalContext) {
    let component =
        create_component_for_default_schema_name(ctx, "starfield", "across the universe")
            .await
            .expect("could not create component");
    let variant_id = Component::schema_variant_id(ctx, component.id())
        .await
        .expect("find variant id for component");

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

    let view = AttributeValue::get_by_id_or_error(ctx, universe_value_id)
        .await
        .expect("get av")
        .view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(universe_json, view);

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let view = AttributeValue::get_by_id_or_error(ctx, universe_value_id)
        .await
        .expect("get av")
        .view(ctx)
        .await
        .expect("get view")
        .expect("has a view");

    assert_eq!(universe_json, view);
}

#[test]
async fn paste_component(ctx: &mut DalContext) {
    let pirate_name = "Long John Silver";
    let parrot_name = "Captain Flint";
    let pirate_component = create_component_for_default_schema_name(ctx, "pirate", pirate_name)
        .await
        .expect("could not create component");

    let parrots_path = &["root", "domain", "parrot_names"];

    let pet_shop_component = create_component_for_default_schema_name(ctx, "pet_shop", "Petopia")
        .await
        .expect("could not create component");

    // set value on source component
    {
        let pet_shop_parrot_av_id = pet_shop_component
            .attribute_values_for_prop(ctx, parrots_path)
            .await
            .expect("find value ids for prop parrot_names")
            .pop()
            .expect("there should only be one value id");

        AttributeValue::insert(ctx, pet_shop_parrot_av_id, Some(parrot_name.into()), None)
            .await
            .expect("insert value in pet_shop parrot_names array");
    }

    connect_components_with_socket_names(
        ctx,
        pet_shop_component.id(),
        "parrot_names",
        pirate_component.id(),
        "parrot_names",
    )
    .await
    .expect("could not connect components with socket names");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let pasted_pirate_component = pirate_component
        .copy_paste(
            ctx,
            ComponentGeometry {
                x: pirate_component.x().to_string(),
                y: pirate_component.y().to_string(),
                width: None,
                height: None,
            },
        )
        .await
        .expect("unable to paste component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let view = pasted_pirate_component
        .view(ctx)
        .await
        .expect("unable to get materialized view of component")
        .expect("no view found");

    assert_eq!(
        view,
        serde_json::json!({
            "domain": {
                // Propagated from /si/name, which means the attribute prototype has been copied
                // from the copied component (since we manually set all values, which removes the
                // default attribute prototype for the slot
                "name": "Long John Silver - Copy",

                // The connection is not copied
                // "parrot_names": [
                //     "Captain Flint",
                // ],
            },
            "resource_value": {},
            "resource": {},
            "si": {
                "color": "#ff00ff",
                "name": "Long John Silver - Copy",
                "type": "component",
            },
        })
    );
}
