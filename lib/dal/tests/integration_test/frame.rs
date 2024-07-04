use dal::component::frame::{Frame, FrameError};
use dal::diagram::SummaryDiagramInferredEdge;
use dal::diagram::{Diagram, DiagramResult, SummaryDiagramComponent, SummaryDiagramEdge};
use dal::{AttributeValue, Component, DalContext, Schema, SchemaVariant};
use dal::{ComponentType, InputSocket, OutputSocket};
use dal_test::helpers::{
    connect_components_with_socket_names, create_component_for_default_schema_name,
    create_component_for_schema_name_with_type, get_component_input_socket_value,
    get_component_output_socket_value, update_attribute_value_for_component, ChangeSetTestHelpers,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use std::collections::HashMap;

mod omega_nesting;
mod with_actions;

#[test]
async fn frames_and_connections(ctx: &mut DalContext) {
    //create 1 components and draw edges to another 1
    let first_component =
        create_component_for_default_schema_name(ctx, "small even lego", "first_component")
            .await
            .expect("could not create component");
    let second_component =
        create_component_for_default_schema_name(ctx, "small odd lego", "second_component")
            .await
            .expect("could not create component");

    //connect them
    connect_components_with_socket_names(
        ctx,
        first_component.id(),
        "one",
        second_component.id(),
        "one",
    )
    .await
    .expect("could not connect components with socket names");
    update_attribute_value_for_component(
        ctx,
        first_component.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // make sure the value propagates
    let input_value = get_component_input_socket_value(ctx, second_component.id(), "one")
        .await
        .expect("could not get component input socket value")
        .expect("has value");
    assert_eq!(
        "1",         // expected
        input_value, // actual
    );
    //create 2 of the same schema variant, only use frames to connect them
    //create 1 components and draw edges to another 1
    let third_component =
        create_component_for_default_schema_name(ctx, "small even lego", "third_component")
            .await
            .expect("could not create component");
    let fourth_component =
        create_component_for_default_schema_name(ctx, "small odd lego", "fourth_component")
            .await
            .expect("could not create component");
    fourth_component
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("set type");
    Frame::upsert_parent(ctx, third_component.id(), fourth_component.id())
        .await
        .expect("upserted parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    // set a value
    update_attribute_value_for_component(
        ctx,
        third_component.id(),
        &["root", "domain", "one"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");
    // values should propagate
    let input_value = get_component_input_socket_value(ctx, fourth_component.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "2",         // expected
        input_value, // actual
    );
}

#[test]
async fn convert_component_to_frame_and_attach_no_nesting(ctx: &mut DalContext) {
    let starfield_schema = Schema::find_by_name(ctx, "starfield")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");
    let fallout_schema = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");

    // Create components using the test exclusive schemas. Neither of them should be frames.
    let starfield_schema_variant = SchemaVariant::list_for_schema(ctx, starfield_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found");
    let fallout_schema_variant = SchemaVariant::list_for_schema(ctx, fallout_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found");
    let starfield_component = Component::new(ctx, "parent", starfield_schema_variant.id())
        .await
        .expect("could not create component");
    let fallout_component = Component::new(ctx, "child", fallout_schema_variant.id())
        .await
        .expect("could not create component");

    // Attempt to attach a child to a parent that is a not a frame.
    match Frame::upsert_parent(ctx, fallout_component.id(), starfield_component.id()).await {
        Ok(()) => panic!("attaching child to parent should fail if parent is not a frame"),
        Err(FrameError::ParentIsNotAFrame(..)) => {}
        Err(other_error) => panic!("unexpected error: {0}", other_error),
    }

    // Change the parent to become a frame.
    let type_attribute_value_id = starfield_component
        .attribute_values_for_prop(ctx, &["root", "si", "type"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(
        ctx,
        type_attribute_value_id,
        Some(serde_json::json!["ConfigurationFrameDown"]),
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Now that the parent is a frame, attempt to attach the child.
    Frame::upsert_parent(ctx, fallout_component.id(), starfield_component.id())
        .await
        .expect("could not attach child to parent");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Assemble the diagram and ensure we see the right number of components.
    let diagram = Diagram::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(2, diagram.components.len());

    // Collect the parent ids for the components on the diagram.
    let mut starfield_parent_node_id = None;
    let mut fallout_parent_node_id = None;
    for component in diagram.components {
        match component.schema_name.as_str() {
            "starfield" => starfield_parent_node_id = Some(component.parent_id),
            "fallout" => fallout_parent_node_id = Some(component.parent_id),
            schema_name => panic!(
                "unexpected schema name for diagram component: {0}",
                schema_name
            ),
        }
    }
    let starfield_parent_node_id =
        starfield_parent_node_id.expect("could not find starfield parent node id");
    let fallout_parent_node_id =
        fallout_parent_node_id.expect("could not find fallout parent node id");

    // Ensure the frame does not have a parent and the child's parent is the frame.
    assert!(starfield_parent_node_id.is_none());
    assert_eq!(
        starfield_component.id(),
        fallout_parent_node_id.expect("no parent node id for fallout component")
    );
}

#[test]
async fn simple_frames(ctx: &mut DalContext) {
    let swifty_schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");
    let fallout_schema = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");

    // Collect schema variants.
    let swifty_schema_variant_id = SchemaVariant::list_for_schema(ctx, swifty_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();
    let fallout_schema_variant_id = SchemaVariant::list_for_schema(ctx, fallout_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();

    // Scenario 1: create an Swifty frame.
    let new_era_taylor_swift_name = "new age taylor swift";
    let new_era_taylor_swift =
        Component::new(ctx, new_era_taylor_swift_name, swifty_schema_variant_id)
            .await
            .expect("could not create component");
    // swifty frame is a down frame
    let new_component_type = ComponentType::Component;
    new_era_taylor_swift
        .set_type(ctx, new_component_type)
        .await
        .expect("could not set type");

    // Validate Scenario 1
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            1,                        // expected
            diagram.components.len()  // actual
        );
        assert!(diagram.edges.is_empty());

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
        let found_type = &new_era_taylor_swift_assembled.0.component_type;
        assert_eq!(
            ComponentType::Component,
            serde_json::from_value(serde_json::Value::String(found_type.to_string()))
                .expect("could not something something")
        );
        let input_sockets = Component::input_socket_attribute_values_for_component_id(
            ctx,
            new_era_taylor_swift.id(),
        )
        .await
        .expect("couldn't get input sockets");
        assert_eq!(1, input_sockets.keys().len());
    }

    // Scenario 2: create a kelce frame and attach to swifty component
    let travis_kelce_component_name = "travis kelce";
    let travis_kelce_component =
        Component::new(ctx, travis_kelce_component_name, fallout_schema_variant_id)
            .await
            .expect("could not create component");
    travis_kelce_component
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("couldn't set type");
    Frame::upsert_parent(ctx, new_era_taylor_swift.id(), travis_kelce_component.id())
        .await
        .expect("could not attach child to parent");

    // Validate Scenario 2
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            2,                        // expected
            diagram.components.len()  // actual
        );
        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );

        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_some());
        assert_eq!(
            travis_kelce_assembled.0.component_id, // expected
            new_era_taylor_swift_assembled
                .0
                .parent_id
                .expect("no parent node id")  // actual
        );

        let output_sockets = Component::output_socket_attribute_values_for_component_id(
            ctx,
            travis_kelce_component.id(),
        )
        .await
        .expect("couldn't get output sockets");
        assert_eq!(2, output_sockets.keys().len(),);

        // make sure Swifty component matches the travis kelsey frame output sockets
        let swifty_input = InputSocket::find_with_name(ctx, "fallout", swifty_schema_variant_id)
            .await
            .expect("could not find input socket by name")
            .expect("is some");
        //let mut maybe_travis_output_socket = None;
        for (input_socket_id, input_socket_match) in
            Component::input_socket_attribute_values_for_component_id(
                ctx,
                new_era_taylor_swift.id(),
            )
            .await
            .expect("couldn't get input sockets")
        {
            if input_socket_id == swifty_input.id() {
                let mut possible_match =
                    Component::find_available_inferred_connections_to_input_socket(
                        ctx,
                        input_socket_match,
                    )
                    .await
                    .expect("couldn't find implicit inputs");
                assert!(!possible_match.is_empty());
                let travis_output_match = possible_match.pop().expect("has a value");
                //maybe_travis_output_socket = Some(travis_output);
                assert_eq!(
                    travis_output_match.component_id,
                    travis_kelce_assembled.0.component_id
                );
            }
        }
        //make sure travis output socket can find swifty input socket
        let outputs = Component::output_socket_attribute_values_for_component_id(
            ctx,
            travis_kelce_component.id(),
        )
        .await
        .expect("could not get output socket avs");
        let output_id = OutputSocket::find_with_name(ctx, "fallout", fallout_schema_variant_id)
            .await
            .expect("could not get output socket by name")
            .expect("value exists");
        let real_id = outputs.get(&output_id.id()).expect("found a value");
        let maybe_ins = Component::find_inferred_values_using_this_output_socket(
            ctx,
            real_id.attribute_value_id,
        )
        .await
        .expect("found one");
        assert!(!maybe_ins.is_empty());
        assert_eq!(maybe_ins.len(), 1);
        assert_eq!(diagram.get_all_inferred_edges().len(), 1);
    }
    // scenario 3 - detach and make sure nothing implicit passes

    Frame::orphan_child(ctx, new_era_taylor_swift.id())
        .await
        .expect("could not orphan");
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            2,                        // expected
            diagram.components.len()  // actual
        );
        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );

        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());

        let output_sockets = Component::output_socket_attribute_values_for_component_id(
            ctx,
            travis_kelce_component.id(),
        )
        .await
        .expect("couldn't get output sockets");
        assert_eq!(2, output_sockets.keys().len(),);

        // make sure Swifty component matches the travis kelsey frame output sockets
        let swifty_input = InputSocket::find_with_name(ctx, "fallout", swifty_schema_variant_id)
            .await
            .expect("could not get input socket by name")
            .expect("value found");
        for (input_socket_id, input_socket_match) in
            Component::input_socket_attribute_values_for_component_id(
                ctx,
                new_era_taylor_swift.id(),
            )
            .await
            .expect("couldn't get input sockets")
        {
            if input_socket_id == swifty_input.id() {
                let possible_match =
                    Component::find_available_inferred_connections_to_input_socket(
                        ctx,
                        input_socket_match,
                    )
                    .await
                    .expect("couldn't find implicit inputs");
                assert!(possible_match.is_empty());
            }
        }
        //make sure travis output socket can find swifty input socket
        let outputs = Component::output_socket_attribute_values_for_component_id(
            ctx,
            travis_kelce_component.id(),
        )
        .await
        .expect("values");
        let output_id = OutputSocket::find_with_name(ctx, "fallout", fallout_schema_variant_id)
            .await
            .expect("could not get output socket by name")
            .expect("value exists");
        let real_id = outputs.get(&output_id.id()).expect("found a value");
        let maybe_ins = Component::find_inferred_values_using_this_output_socket(
            ctx,
            real_id.attribute_value_id,
        )
        .await
        .expect("could not find inferred values");
        assert!(maybe_ins.is_empty());
        assert_eq!(diagram.get_all_inferred_edges().len(), 0);
    }
}

#[test]
async fn output_sockets_can_have_both(ctx: &mut DalContext) {
    // create an even frame
    let even_frame = create_component_for_default_schema_name(ctx, "large even lego", "even")
        .await
        .expect("could not create component");

    even_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    let odd_component = create_component_for_default_schema_name(ctx, "large odd lego", "odd1")
        .await
        .expect("could not create component");
    odd_component
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");
    Frame::upsert_parent(ctx, odd_component.id(), even_frame.id())
        .await
        .expect("could not upsert parent");
    // Change attribute value for one
    let type_attribute_value_id = even_frame
        .attribute_values_for_prop(ctx, &["root", "domain", "one"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(ctx, type_attribute_value_id, Some(serde_json::json!["1"]))
        .await
        .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // create another odd component, but manually connect to the frame (not a child!)
    let odd_component_2 = create_component_for_default_schema_name(ctx, "large odd lego", "odd2")
        .await
        .expect("could not create component");
    odd_component_2
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");

    connect_components_with_socket_names(ctx, even_frame.id(), "one", odd_component_2.id(), "one")
        .await
        .expect("could not connect components with socket names");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let diagram = DiagramByKey::assemble(ctx)
        .await
        .expect("could not assemble diagram");
    assert_eq!(
        3,                        // expected
        diagram.components.len()  // actual
    );
    assert_eq!(
        1,                   // expected
        diagram.edges.len()  // actual
    );
    assert_eq!(
        3,                                      // expected
        diagram.get_all_inferred_edges().len()  // actual
    );
    let odd_component = Component::get_by_id(ctx, odd_component.id())
        .await
        .expect("got component");
    let odd_component_1_av = odd_component
        .attribute_values_for_prop(ctx, &["root", "domain", "one"])
        .await
        .expect("got avs")
        .into_iter()
        .next()
        .expect("got av");
    let odd_component_1_mat_view = AttributeValue::get_by_id_or_error(ctx, odd_component_1_av)
        .await
        .expect("got av")
        .view(ctx)
        .await
        .expect("got mat view")
        .expect("has value");
    assert_eq!(odd_component_1_mat_view, serde_json::json!("1"));
    let odd_component_2 = Component::get_by_id(ctx, odd_component_2.id())
        .await
        .expect("got component");
    let odd_component_2_av = odd_component_2
        .attribute_values_for_prop(ctx, &["root", "domain", "one"])
        .await
        .expect("got avs")
        .into_iter()
        .next()
        .expect("got av");
    let odd_component_2_mat_view = AttributeValue::get_by_id_or_error(ctx, odd_component_2_av)
        .await
        .expect("got av")
        .view(ctx)
        .await
        .expect("got mat view")
        .expect("has value");
    assert_eq!(odd_component_2_mat_view, serde_json::json!("1"));
}

#[test]
async fn up_frames_take_inputs_from_down_frames_too(ctx: &mut DalContext) {
    // create an odd down frame
    let level_one = create_component_for_schema_name_with_type(
        ctx,
        "medium even lego",
        "level one",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    // create an even up frame
    let level_two = create_component_for_schema_name_with_type(
        ctx,
        "large odd lego",
        "level two",
        ComponentType::ConfigurationFrameUp,
    )
    .await
    .expect("could not create component");

    // upsert even frame into odd frame
    Frame::upsert_parent(ctx, level_two.id(), level_one.id())
        .await
        .expect("could not upsert frame");

    // create odd component to go inside even up frame
    let level_three = create_component_for_schema_name_with_type(
        ctx,
        "small even lego",
        "level three",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");

    // upsert component into up frame
    Frame::upsert_parent(ctx, level_three.id(), level_two.id())
        .await
        .expect("could not upsert parent");

    update_attribute_value_for_component(
        ctx,
        level_one.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await
    .expect("could not update attribute value");

    update_attribute_value_for_component(
        ctx,
        level_three.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await
    .expect("could not update attribute value");
    update_attribute_value_for_component(
        ctx,
        level_one.id(),
        &["root", "domain", "one"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // make sure everything looks as expected

    let input_value = get_component_input_socket_value(ctx, level_two.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        serde_json::json![vec!["2", "1"]], // expected
        input_value,                       // actual
    );
    let input_value = get_component_input_socket_value(ctx, level_two.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "3",         // expected
        input_value, // actual
    );
}

#[test]
async fn orphan_frames_deeply_nested(ctx: &mut DalContext) {
    // create a large up frame
    let even_level_one = create_component_for_schema_name_with_type(
        ctx,
        "large even lego",
        "level one",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("created frame");
    // put another medium frame inside
    let even_level_two = create_component_for_schema_name_with_type(
        ctx,
        "medium even lego",
        "level two",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    Frame::upsert_parent(ctx, even_level_two.id(), even_level_one.id())
        .await
        .expect("could not upsert parent");

    // create an odd frame inside level 2 (that we will later detach)
    let odd_level_three = create_component_for_schema_name_with_type(
        ctx,
        "large odd lego",
        "level three",
        ComponentType::ConfigurationFrameDown,
    )
    .await
    .expect("could not create component");

    Frame::upsert_parent(ctx, odd_level_three.id(), even_level_two.id())
        .await
        .expect("could not create upsert frame");

    // create an odd component inside level 3 (that will move when level 3 is detached)
    let odd_level_four = create_component_for_schema_name_with_type(
        ctx,
        "large odd lego",
        "level four",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");

    Frame::upsert_parent(ctx, odd_level_four.id(), odd_level_three.id())
        .await
        .expect("could not upsert parent");

    // create an even component, also inside level 3 (that will move when level 3 is detached AND take a value from level 3)
    let even_level_four = create_component_for_schema_name_with_type(
        ctx,
        "large even lego",
        "level four even",
        ComponentType::Component,
    )
    .await
    .expect("could not create component");
    Frame::upsert_parent(ctx, even_level_four.id(), odd_level_three.id())
        .await
        .expect("could not upsert parent");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // now let's set some values
    // level one sets output socket 5 which should pass to the level 3 and 4 items
    update_attribute_value_for_component(
        ctx,
        even_level_one.id(),
        &["root", "domain", "five"],
        serde_json::json!["5"],
    )
    .await
    .expect("could not update attribute value");
    // level two sets output socket 3 which should pass to level 3 and 4 items
    update_attribute_value_for_component(
        ctx,
        even_level_two.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await
    .expect("could not update attribute value");
    // level 3 sets output socket 2 which should pass to level 4 even component
    update_attribute_value_for_component(
        ctx,
        odd_level_three.id(),
        &["root", "domain", "two"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // let's make sure everything is as we expect
    let input_value = get_component_input_socket_value(ctx, odd_level_three.id(), "five")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "5",         // expected
        input_value, // actual
    );
    let input_value = get_component_input_socket_value(ctx, odd_level_four.id(), "five")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "5",         // expected
        input_value, // actual
    );
    let input_value = get_component_input_socket_value(ctx, odd_level_three.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "3",         // expected
        input_value, // actual
    );
    let input_value = get_component_input_socket_value(ctx, odd_level_four.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "3",         // expected
        input_value, // actual
    );
    let input_value = get_component_input_socket_value(ctx, even_level_four.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "2",         // expected
        input_value, // actual
    );

    // now let's orphan level 3
    Frame::orphan_child(ctx, odd_level_three.id())
        .await
        .expect("could not orphan component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // let's make sure everything updated accordingly
    let input_value = get_component_input_socket_value(ctx, odd_level_three.id(), "five")
        .await
        .expect("could not get input socket value");
    assert!(input_value.is_none());
    let input_value = get_component_input_socket_value(ctx, odd_level_four.id(), "five")
        .await
        .expect("could not get input socket value");
    assert!(input_value.is_none());
    let input_value = get_component_input_socket_value(ctx, odd_level_three.id(), "three")
        .await
        .expect("could not get input socket value");

    assert!(input_value.is_none());
    let input_value = get_component_input_socket_value(ctx, odd_level_four.id(), "three")
        .await
        .expect("could not get input socket value");

    assert!(input_value.is_none());
    let input_value = get_component_input_socket_value(ctx, even_level_four.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(
        "2",         // expected
        input_value, // actual
    );
}

#[test]
async fn simple_down_frames_no_nesting(ctx: &mut DalContext) {
    let even_frame = create_component_for_default_schema_name(ctx, "large even lego", "even")
        .await
        .expect("could not create component");
    let even_frame_component_id = even_frame.id();
    even_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");

    let odd_component = create_component_for_default_schema_name(ctx, "large odd lego", "odd")
        .await
        .expect("could not create component");
    odd_component
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");
    Frame::upsert_parent(ctx, odd_component.id(), even_frame.id())
        .await
        .expect("could not upsert parent");
    // Change attribute value for one
    update_attribute_value_for_component(
        ctx,
        even_frame_component_id,
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the output socket value is updated with 1
    let output_value = get_component_output_socket_value(ctx, even_frame_component_id, "one")
        .await
        .expect("could not get output socket value")
        .expect("has value");
    assert_eq!(output_value, serde_json::json!("1"));

    let input_value = get_component_input_socket_value(ctx, odd_component.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!("1"));
}

#[test]
async fn down_frames_moving_deeply_nested_frames(ctx: &mut DalContext) {
    // here's the scenario:
    // Create two greatgrandparent frames of the same schema variant, each with a grandparent frame inside, with different values to propagate
    // Put a parent frame, with two different child components and ensure values propagate
    // (one child takes from parent + great grandparent, and the other child takes from just the grandparent)
    // move the parent to the other grandparent (which is inside the other great grandparent) (with a different value set)
    // ensure the children is updated with all the new values
    // This test is to ensure that when we move frames with children between frames, the frames AND the children update accordingly

    // create first greatgrandparent
    let first_greatgrandparent_frame =
        create_component_for_default_schema_name(ctx, "medium even lego", "greatgrandparent 1")
            .await
            .expect("could not create component");

    first_greatgrandparent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    // create grandparent frame
    let first_grand_parent_frame =
        create_component_for_default_schema_name(ctx, "medium odd lego", "grandparent")
            .await
            .expect("could not create component");
    first_grand_parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    // create parent frame
    let parent_frame = create_component_for_default_schema_name(ctx, "small even lego", "parent")
        .await
        .expect("could not create component");
    parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    // create child components
    let first_child_component = create_component_for_default_schema_name(ctx, "medium odd lego", "child 1")
        .await
        .expect("could not create component");
    first_child_component
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    let second_child_component =
        create_component_for_default_schema_name(ctx, "medium even lego", "child 2")
            .await
            .expect("could not create component");
    second_child_component
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");

    // upsert child into parent, parent into grandparent, grandparent into great grandparent and child into grandparent
    Frame::upsert_parent(ctx, first_child_component.id(), parent_frame.id())
        .await
        .expect("can upsert parent");
    Frame::upsert_parent(ctx, parent_frame.id(), first_grand_parent_frame.id())
        .await
        .expect("can upsert parent");
    Frame::upsert_parent(ctx, second_child_component.id(), parent_frame.id())
        .await
        .expect("can upsert parent");
    Frame::upsert_parent(
        ctx,
        first_grand_parent_frame.id(),
        first_greatgrandparent_frame.id(),
    )
    .await
    .expect("can upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // set values for Grandparent and Parent Frames

    // this value should pass to the grandparent
    update_attribute_value_for_component(
        ctx,
        first_greatgrandparent_frame.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await
    .expect("could not update attribute value");
    // this value should only pass to the grandparent, the first_child has a closer match with its parent
    update_attribute_value_for_component(
        ctx,
        first_greatgrandparent_frame.id(),
        &["root", "domain", "one"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");
    // this value should pass to the first_child
    update_attribute_value_for_component(
        ctx,
        parent_frame.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await
    .expect("could not update attribute value");
    // this value should pass to the second_child
    update_attribute_value_for_component(
        ctx,
        first_grand_parent_frame.id(),
        &["root", "domain", "two"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the first_component is updated with 3
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("3"));
    // the first_componenent is updated with 1
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("1"));
    // the second_component is updated with 2
    let input_value = get_component_input_socket_value(ctx, second_child_component.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("2"));
    // the parent is updated with 2
    let input_value = get_component_input_socket_value(ctx, parent_frame.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("2"));

    // now create the other great grandparent and grandparent frame and move the parent into it
    let second_greatgrandparent_frame =
        create_component_for_default_schema_name(ctx, "medium even lego", "grandparent 2")
            .await
            .expect("could not create component");

    second_greatgrandparent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    let second_grand_parent_frame =
        create_component_for_default_schema_name(ctx, "small odd lego", "grandparent")
            .await
            .expect("could not create component");
    second_grand_parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    Frame::upsert_parent(
        ctx,
        second_grand_parent_frame.id(),
        second_greatgrandparent_frame.id(),
    )
    .await
    .expect("can upsert parent");

    Frame::upsert_parent(ctx, parent_frame.id(), second_grand_parent_frame.id())
        .await
        .expect("can upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the value coming from the first great grandparent should be unset
    assert!(
        get_component_input_socket_value(ctx, first_child_component.id(), "three")
            .await
            .expect("could not get input socket value")
            .is_none()
    );
    // the value coming from the first grandparent should be unset
    assert!(
        get_component_input_socket_value(ctx, second_child_component.id(), "two")
            .await
            .expect("could not get input socket value")
            .is_none()
    );
    assert!(
        get_component_input_socket_value(ctx, parent_frame.id(), "two")
            .await
            .expect("could not get input socket value")
            .is_none()
    );

    // this value should pass as the parent frame doesn't have an output socket for "3"
    update_attribute_value_for_component(
        ctx,
        second_greatgrandparent_frame.id(),
        &["root", "domain", "three"],
        serde_json::json!["4"],
    )
    .await
    .expect("could not update attribute value");
    // this value should pass as the parent frame doesn't have an output socket for "3"
    update_attribute_value_for_component(
        ctx,
        second_grand_parent_frame.id(),
        &["root", "domain", "two"],
        serde_json::json!["5"],
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the first_componenent still has 1
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("1"));
    // the component is updated with 4
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("is some");

    assert_eq!(input_value, serde_json::json!("4"));
    // the second component is updated with 5
    let input_value = get_component_input_socket_value(ctx, second_child_component.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("is some");

    assert_eq!(input_value, serde_json::json!("5"));
    // the parent is updated with 5
    let input_value = get_component_input_socket_value(ctx, parent_frame.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("is some");

    assert_eq!(input_value, serde_json::json!("5"));
}

#[test]
async fn up_frames_moving_deeply_nested_frames(ctx: &mut DalContext) {
    // this is the inverse of the down_frame_moving_deeply_nested_frames test (using up frames vs. down frames)

    // create first greatgrandparent
    let first_greatgrandparent_frame =
        create_component_for_default_schema_name(ctx, "medium even lego", "greatgrandparent 1")
            .await
            .expect("could not create component");

    first_greatgrandparent_frame
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");
    // create grandparent frame
    let first_grand_parent_frame =
        create_component_for_default_schema_name(ctx, "medium odd lego", "grandparent")
            .await
            .expect("could not create component");
    first_grand_parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");
    // create parent frame
    let parent_frame = create_component_for_default_schema_name(ctx, "small even lego", "parent")
        .await
        .expect("could not create component");
    parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");
    // create child components
    let first_child_component = create_component_for_default_schema_name(ctx, "medium odd lego", "child 1")
        .await
        .expect("could not create component");
    first_child_component
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");

    // upsert child into parent, parent into grandparent, grandparent into great grandparent and child into grandparent
    Frame::upsert_parent(ctx, parent_frame.id(), first_child_component.id())
        .await
        .expect("can upsert parent");
    Frame::upsert_parent(ctx, first_grand_parent_frame.id(), parent_frame.id())
        .await
        .expect("can upsert parent");

    Frame::upsert_parent(
        ctx,
        first_greatgrandparent_frame.id(),
        first_grand_parent_frame.id(),
    )
    .await
    .expect("can upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // set values for Grandparent and Parent Frames

    // this value should pass to the grandparent
    update_attribute_value_for_component(
        ctx,
        first_greatgrandparent_frame.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await
    .expect("could not update attribute value");
    // this value should only pass to the grandparent, the first_child has a closer match with its parent
    update_attribute_value_for_component(
        ctx,
        first_greatgrandparent_frame.id(),
        &["root", "domain", "one"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");
    // this value should pass to the first_child
    update_attribute_value_for_component(
        ctx,
        parent_frame.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await
    .expect("could not update attribute value");
    // this value should pass to the second_child
    update_attribute_value_for_component(
        ctx,
        first_grand_parent_frame.id(),
        &["root", "domain", "two"],
        serde_json::json!["2"],
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the first_component is updated with 3
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("3"));
    // the first_componenent is updated with 1
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("1"));

    // the parent is updated with 2
    let input_value = get_component_input_socket_value(ctx, parent_frame.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("2"));

    // now create the other great grandparent and grandparent frame and move the parent into it
    let second_greatgrandparent_frame =
        create_component_for_default_schema_name(ctx, "medium even lego", "grandparent 2")
            .await
            .expect("could not create component");

    second_greatgrandparent_frame
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");
    let second_grand_parent_frame =
        create_component_for_default_schema_name(ctx, "small odd lego", "grandparent")
            .await
            .expect("could not create component");
    second_grand_parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");
    Frame::upsert_parent(
        ctx,
        second_greatgrandparent_frame.id(),
        second_grand_parent_frame.id(),
    )
    .await
    .expect("can upsert parent");
    // detach the first grand parent from the parent
    Frame::orphan_child(ctx, first_grand_parent_frame.id())
        .await
        .expect("can detach frame");

    Frame::upsert_parent(ctx, second_grand_parent_frame.id(), parent_frame.id())
        .await
        .expect("can upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the value coming from the first great grandparent should be unset
    assert!(
        get_component_input_socket_value(ctx, first_child_component.id(), "three")
            .await
            .expect("could not get input socket value")
            .is_none()
    );

    assert!(
        get_component_input_socket_value(ctx, parent_frame.id(), "two")
            .await
            .expect("could not get input socket value")
            .is_none()
    );

    // this value should pass as the parent frame doesn't have an output socket for "3"
    update_attribute_value_for_component(
        ctx,
        second_greatgrandparent_frame.id(),
        &["root", "domain", "three"],
        serde_json::json!["4"],
    )
    .await
    .expect("could not update attribute value");
    // this value should pass as the parent frame doesn't have an output socket for "3"
    update_attribute_value_for_component(
        ctx,
        second_grand_parent_frame.id(),
        &["root", "domain", "two"],
        serde_json::json!["5"],
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the first_componenent still has 1
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("1"));
    // the component is updated with 4
    let input_value = get_component_input_socket_value(ctx, first_child_component.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("is some");

    assert_eq!(input_value, serde_json::json!("4"));
    // the second component is updated with 5

    // the parent is updated with 5
    let input_value = get_component_input_socket_value(ctx, parent_frame.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("is some");

    assert_eq!(input_value, serde_json::json!("5"));
}

#[test]
async fn simple_down_frames_nesting(ctx: &mut DalContext) {
    // create parent frame
    let even_parent_frame = create_component_for_default_schema_name(ctx, "large even lego", "even parent")
        .await
        .expect("could not create component");

    even_parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    // create child frame
    let even_child_frame = create_component_for_default_schema_name(ctx, "medium even lego", "even child")
        .await
        .expect("could not create component");

    even_child_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await
        .expect("could not set type");
    // insert child frame into parent frame
    Frame::upsert_parent(ctx, even_child_frame.id(), even_parent_frame.id())
        .await
        .expect("can upsert parent");
    // create component
    let odd_component = create_component_for_default_schema_name(ctx, "large odd lego", "odd")
        .await
        .expect("could not create component");
    odd_component
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");
    // insert component into CHILD frame
    Frame::upsert_parent(ctx, odd_component.id(), even_child_frame.id())
        .await
        .expect("can upsert to child frame");
    update_attribute_value_for_component(
        ctx,
        even_parent_frame.id(),
        &["root", "domain", "five"],
        serde_json::json!["5"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // the output socket value is updated with 1
    let output_value = get_component_output_socket_value(ctx, even_parent_frame.id(), "five")
        .await
        .expect("could not get output socket value")
        .expect("is some");
    assert_eq!(output_value, serde_json::json!("5"));

    // the component is updated with 5
    let input_value = get_component_input_socket_value(ctx, odd_component.id(), "five")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("5"));

    // now let's update the parent frame to a value that the child also has
    update_attribute_value_for_component(
        ctx,
        even_parent_frame.id(),
        &["root", "domain", "three"],
        serde_json::json!["4"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the component doesn't get the update as the child frame is a closer match and overrides it
    assert!(
        get_component_input_socket_value(ctx, odd_component.id(), "three")
            .await
            .expect("could not get input socket value")
            .is_none()
    );

    // now let's update the child frame's same socket to a value the component should take
    update_attribute_value_for_component(
        ctx,
        even_child_frame.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the component gets the update as the child frame is a closer match
    let input_value = get_component_input_socket_value(ctx, odd_component.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("3"));

    // now let's pop the component to the parent frame and make sure it gets the new socket value
    Frame::upsert_parent(ctx, odd_component.id(), even_parent_frame.id())
        .await
        .expect("could not upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // make sure the parent is right...
    let odd_component = Component::get_by_id(ctx, odd_component.id())
        .await
        .expect("could not get component by id");
    let new_parent = odd_component
        .parent(ctx)
        .await
        .expect("could not get component's parent")
        .expect("is some");
    assert_eq!(new_parent, even_parent_frame.id());
    // make sure the input socket for the component is updated
    let input_value = get_component_input_socket_value(ctx, odd_component.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("is some");
    assert_eq!(input_value, serde_json::json!("4"));
}

#[test]
async fn simple_up_frames_some_nesting(ctx: &mut DalContext) {
    let even_component = create_component_for_default_schema_name(ctx, "small even lego", "even")
        .await
        .expect("could not create component");

    even_component
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");

    let odd_up_frame = create_component_for_default_schema_name(ctx, "large odd lego", "odd")
        .await
        .expect("could not create component");
    odd_up_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");
    Frame::upsert_parent(ctx, even_component.id(), odd_up_frame.id())
        .await
        .expect("could not upsert parent");
    // Change attribute value for one on the component
    update_attribute_value_for_component(
        ctx,
        even_component.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the output socket value is updated with "1"
    let output_value = get_component_output_socket_value(ctx, even_component.id(), "one")
        .await
        .expect("could not get output socket value")
        .expect("has value");
    assert_eq!(output_value, serde_json::json!("1"));

    // make sure component output socket matches on the up frames input socket
    let input_value = get_component_input_socket_value(ctx, odd_up_frame.id(), "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");

    assert_eq!(input_value, serde_json::json!("1"));

    //let's add another component to the frame, to drive the "3" input socket
    let another_even_component =
        create_component_for_default_schema_name(ctx, "medium even lego", "another even")
            .await
            .expect("could not create component");

    another_even_component
        .set_type(ctx, ComponentType::Component)
        .await
        .expect("could not set type");
    Frame::upsert_parent(ctx, another_even_component.id(), odd_up_frame.id())
        .await
        .expect("could not upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Change attribute value for three on the component
    update_attribute_value_for_component(
        ctx,
        another_even_component.id(),
        &["root", "domain", "three"],
        serde_json::json!("3"),
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the output socket value is updated with "3"
    let output_value = get_component_output_socket_value(ctx, another_even_component.id(), "three")
        .await
        .expect("could not get output socket value")
        .expect("has value");
    assert_eq!(output_value, serde_json::json!("3"));
    // make sure component output socket matches on the up frames input socket
    let input_value = get_component_input_socket_value(ctx, odd_up_frame.id(), "three")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!("3"));

    //now let's drop that up frame into an even up frame, driving the even values
    let even_up_frame = create_component_for_default_schema_name(ctx, "large even lego", "another even")
        .await
        .expect("could not create component");

    even_up_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("could not set type");
    Frame::upsert_parent(ctx, odd_up_frame.id(), even_up_frame.id())
        .await
        .expect("could not upsert parent frame");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Change attribute value for two on the odd up frame
    update_attribute_value_for_component(
        ctx,
        odd_up_frame.id(),
        &["root", "domain", "two"],
        serde_json::json!("2"),
    )
    .await
    .expect("could not update attribute value");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // the output socket value is updated with "2"
    let output_value = get_component_output_socket_value(ctx, odd_up_frame.id(), "two")
        .await
        .expect("could not get output socket value")
        .expect("has value");
    assert_eq!(output_value, serde_json::json!("2"));

    // even up frame input socket matches odd up frame output socket
    let input_value = get_component_input_socket_value(ctx, even_up_frame.id(), "two")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!("2"));
}

#[test]
async fn up_frames_multiple_children_moves_and_deletes(ctx: &mut DalContext) {
    // create two components to feed an up frame
    let first_component =
        create_component_for_default_schema_name(ctx, "medium even lego", "first_component")
            .await
            .expect("could not create component");

    let second_component =
        create_component_for_default_schema_name(ctx, "medium even lego", "second_component")
            .await
            .expect("could not create component");
    let first_up_frame = create_component_for_default_schema_name(ctx, "medium odd lego", "first_frame")
        .await
        .expect("could not create component");
    first_up_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("type is set");
    // cache ids for later
    let first_component_id = first_component.id();
    let second_component_id = second_component.id();
    let first_up_frame_id = first_up_frame.id();

    Frame::upsert_parent(ctx, first_component_id, first_up_frame_id)
        .await
        .expect("upserted");
    Frame::upsert_parent(ctx, second_component_id, first_up_frame_id)
        .await
        .expect("upserted");

    // set attribute value for each component
    update_attribute_value_for_component(
        ctx,
        first_component_id,
        &["root", "domain", "one"],
        serde_json::json!("1"),
    )
    .await
    .expect("could not update attribute value");
    update_attribute_value_for_component(
        ctx,
        second_component_id,
        &["root", "domain", "one"],
        serde_json::json!("2"),
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // make sure output socket values are updated for components
    let first_output = get_component_output_socket_value(ctx, first_component_id, "one")
        .await
        .expect("could not get output socket value")
        .expect("has some");
    let second_output = get_component_output_socket_value(ctx, second_component_id, "one")
        .await
        .expect("could not get output socket value")
        .expect("has value");
    assert_eq!(first_output, serde_json::json!("1"));
    assert_eq!(second_output, serde_json::json!("2"));

    //make sure input socket value is updated
    let input_value = get_component_input_socket_value(ctx, first_up_frame_id, "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!(["1", "2"]));
    // create two more components in another up frame
    let third_component =
        create_component_for_default_schema_name(ctx, "medium even lego", "first_component")
            .await
            .expect("could not create component");
    let fourth_component =
        create_component_for_default_schema_name(ctx, "medium even lego", "second_component")
            .await
            .expect("could not create component");
    let second_up_frame = create_component_for_default_schema_name(ctx, "medium odd lego", "first_frame")
        .await
        .expect("could not create component");
    //cache ids for later
    let third_component_id = third_component.id();
    let fourth_component_id = fourth_component.id();
    let second_up_frame_id = second_up_frame.id();
    second_up_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await
        .expect("type is set");
    Frame::upsert_parent(ctx, third_component_id, second_up_frame_id)
        .await
        .expect("upserted");
    Frame::upsert_parent(ctx, fourth_component_id, second_up_frame_id)
        .await
        .expect("upserted");

    // set attribute value for each component
    update_attribute_value_for_component(
        ctx,
        third_component_id,
        &["root", "domain", "one"],
        serde_json::json!("3"),
    )
    .await
    .expect("could not update attribute value");
    update_attribute_value_for_component(
        ctx,
        fourth_component.id(),
        &["root", "domain", "one"],
        serde_json::json!("4"),
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // make sure output socket values are updated for components
    let third_output = get_component_output_socket_value(ctx, third_component_id, "one")
        .await
        .expect("could not get output socket value")
        .expect("has some");
    let fourth_output = get_component_output_socket_value(ctx, fourth_component_id, "one")
        .await
        .expect("could not get output socket value")
        .expect("has value");
    assert_eq!(third_output, serde_json::json!("3"));
    assert_eq!(fourth_output, serde_json::json!("4"));

    //make sure input socket value is updated
    let input_value = get_component_input_socket_value(ctx, second_up_frame_id, "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!(["3", "4"]));
    // both up frames feed the final up frame

    let parent_up_frame = create_component_for_default_schema_name(ctx, "small even lego", "parent_frame")
        .await
        .expect("could not create component");
    let parent_up_frame_id = parent_up_frame.id();
    Frame::upsert_parent(ctx, first_up_frame_id, parent_up_frame_id)
        .await
        .expect("upserted");
    Frame::upsert_parent(ctx, second_up_frame_id, parent_up_frame_id)
        .await
        .expect("upserted");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // make sure parent frame doesn't have any values for the input sockets, but does find them
    assert_eq!(
        get_component_input_socket_value(ctx, parent_up_frame_id, "two")
            .await
            .expect("could not get input socket value")
            .expect("value exists"),
        serde_json::json!([null, null])
    );
    // set one frame's output socket value and make sure it flows through
    update_attribute_value_for_component(
        ctx,
        first_up_frame_id,
        &["root", "domain", "two"],
        serde_json::json!("5"),
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let input_value = get_component_input_socket_value(ctx, parent_up_frame_id, "two")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!(["5", null]));
    //set second frame's outptu socket value and make sure both are now flowing
    update_attribute_value_for_component(
        ctx,
        second_up_frame_id,
        &["root", "domain", "two"],
        serde_json::json!("6"),
    )
    .await
    .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let input_value = get_component_input_socket_value(ctx, parent_up_frame_id, "two")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!(["5", "6"]));

    // now let's delete one of the components, and move one to the other up frame and make sure everything is updated
    let first_component = Component::get_by_id(ctx, first_component_id)
        .await
        .expect("got component");
    first_component.delete(ctx).await.expect("deleted");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    Frame::upsert_parent(ctx, third_component_id, first_up_frame_id)
        .await
        .expect("upserted");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // first frame should have two components
    let input_value = get_component_input_socket_value(ctx, first_up_frame_id, "one")
        .await
        .expect("could not get input socket value")
        .expect("got value");
    assert_eq!(input_value, serde_json::json!(["2", "3"]));
    // second frame should have one component
    let input_value = get_component_input_socket_value(ctx, second_up_frame_id, "one")
        .await
        .expect("could not get input socket value")
        .expect("has value");
    assert_eq!(input_value, serde_json::json!("4"));
}

#[test]
async fn multiple_frames_with_complex_connections_no_nesting(ctx: &mut DalContext) {
    let swifty_schema = Schema::find_by_name(ctx, "swifty")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");
    let fallout_schema = Schema::find_by_name(ctx, "fallout")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");

    // Collect schema variants.
    let swifty_schema_variant_id = SchemaVariant::list_for_schema(ctx, swifty_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();
    let fallout_schema_variant_id = SchemaVariant::list_for_schema(ctx, fallout_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();

    // Scenario 1: create an Swifty frame.
    let new_era_taylor_swift_name = "new age taylor swift";
    let new_era_taylor_swift =
        Component::new(ctx, new_era_taylor_swift_name, swifty_schema_variant_id)
            .await
            .expect("could not create component");

    // Validate Scenario 1
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            1,                        // expected
            diagram.components.len()  // actual
        );
        assert!(diagram.edges.is_empty());

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
    }

    // Scenario 2: create a kelce component and attach to swifty frame
    let travis_kelce_component_name = "travis kelce";
    let travis_kelce_component =
        Component::new(ctx, travis_kelce_component_name, fallout_schema_variant_id)
            .await
            .expect("could not create component");
    Frame::upsert_parent(ctx, travis_kelce_component.id(), new_era_taylor_swift.id())
        .await
        .expect("could not attach child to parent");

    // Validate Scenario 2
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            2,                        // expected
            diagram.components.len()  // actual
        );

        assert_eq!(
            1,                                      // expected
            diagram.get_all_inferred_edges().len()  // actual
        );

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );

        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .0
                .parent_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenario 3: add a different era swifty frame on its own.
    let country_era_taylor_swift_name = "country taylor swift";
    let country_era_taylor_swift =
        Component::new(ctx, country_era_taylor_swift_name, swifty_schema_variant_id)
            .await
            .expect("could not create component");

    // Validate Scenario 3
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            3,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            1,                                      // expected
            diagram.get_all_inferred_edges().len()  // actual
        );

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");
        let country_era_taylor_swift_assembled = diagram
            .components
            .get(country_era_taylor_swift_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                     // expected
            country_era_taylor_swift_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
        assert!(country_era_taylor_swift_assembled.0.parent_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .0
                .parent_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenarios 4 and 5: create a mama kelce component, but place it outside of both frames. Then, drag it onto the second swifty
    // frame.
    let mama_kelce_name = "mama kelce";
    let mama_kelce = Component::new(ctx, mama_kelce_name, fallout_schema_variant_id)
        .await
        .expect("could not create component");
    Frame::upsert_parent(ctx, mama_kelce.id(), country_era_taylor_swift.id())
        .await
        .expect("could not attach child to parent");

    // Validate Scenarios 4 and 5
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            4,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            2,                                      // expected
            diagram.get_all_inferred_edges().len()  // actual
        );

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");
        let country_era_taylor_swift_assembled = diagram
            .components
            .get(country_era_taylor_swift_name)
            .expect("could not get component by name");
        let mama_kelce_assembled = diagram
            .components
            .get(mama_kelce_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                     // expected
            country_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            mama_kelce.id(),                     // expected
            mama_kelce_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
        assert!(country_era_taylor_swift_assembled.0.parent_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .0
                .parent_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(), // expected
            mama_kelce_assembled.0.parent_id.expect("no parent node id")  // actual
        );
    }

    // // Scenarios 6: Country Era taylor Swift within New Era Taylor Swift.
    Frame::upsert_parent(
        ctx,
        country_era_taylor_swift.id(),
        new_era_taylor_swift.id(),
    )
    .await
    .expect("could not attach child to parent");

    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            4,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            2,                                      // expected
            diagram.get_all_inferred_edges().len()  // actual
        );

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");
        let country_era_taylor_swift_assembled = diagram
            .components
            .get(country_era_taylor_swift_name)
            .expect("could not get component by name");
        let mama_kelce_assembled = diagram
            .components
            .get(mama_kelce_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                     // expected
            country_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            mama_kelce.id(),                     // expected
            mama_kelce_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(),
            country_era_taylor_swift_assembled
                .0
                .parent_id
                .expect("no parent node id")
        );
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .0
                .parent_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(), // expected
            mama_kelce_assembled.0.parent_id.expect("no parent node id")  // actual
        );
    }
    //Scenario 7?! No more Country Era Swift, she wants to break freeeeeee
    Frame::orphan_child(ctx, country_era_taylor_swift.id())
        .await
        .expect("could not detach child to parent");
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            4,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            2,                                      // expected
            diagram.get_all_inferred_edges().len()  // actual
        );

        let new_era_taylor_swift_assembled = diagram
            .components
            .get(new_era_taylor_swift_name)
            .expect("could not get component by name");
        let travis_kelce_assembled = diagram
            .components
            .get(travis_kelce_component_name)
            .expect("could not get component by name");
        let country_era_taylor_swift_assembled = diagram
            .components
            .get(country_era_taylor_swift_name)
            .expect("could not get component by name");
        let mama_kelce_assembled = diagram
            .components
            .get(mama_kelce_name)
            .expect("could not get component by name");

        assert_eq!(
            new_era_taylor_swift.id(),                     // expected
            new_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),           // expected
            travis_kelce_assembled.0.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                     // expected
            country_era_taylor_swift_assembled.0.component_id  // actual
        );
        assert_eq!(
            mama_kelce.id(),                     // expected
            mama_kelce_assembled.0.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.0.parent_id.is_none());
        assert!(country_era_taylor_swift_assembled.0.parent_id.is_none());

        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .0
                .parent_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(), // expected
            mama_kelce_assembled.0.parent_id.expect("no parent node id")  // actual
        );
    }
}

struct DiagramByKey {
    pub components: HashMap<String, (SummaryDiagramComponent, Vec<SummaryDiagramInferredEdge>)>,
    pub edges: HashMap<String, SummaryDiagramEdge>,
}

impl DiagramByKey {
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let diagram = Diagram::assemble(ctx).await?;

        let mut components = HashMap::new();
        for component in &diagram.components {
            let mut inferred_edges = vec![];
            for inferred_edge in &diagram.inferred_edges {
                if inferred_edge.to_component_id == component.id {
                    inferred_edges.push(inferred_edge.to_owned());
                }
            }
            components.insert(
                component.display_name.clone(),
                (component.to_owned(), inferred_edges.to_owned()),
            );
        }

        let mut edges = HashMap::new();
        for edge in &diagram.edges {
            edges.insert(
                "{edge.to_socket_id}_{edge.from_socket_id}".to_string(),
                edge.to_owned(),
            );
        }

        Ok(Self { components, edges })
    }

    pub fn get_all_inferred_edges(&self) -> Vec<SummaryDiagramInferredEdge> {
        let mut all = vec![];
        for component in self.components.values() {
            for edge in component.1.clone() {
                all.push(edge);
            }
        }
        all
    }
}
