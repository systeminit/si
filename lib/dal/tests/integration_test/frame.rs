use dal::component::frame::{Frame, FrameError};
use dal::diagram::SummaryDiagramInferredEdge;
use dal::diagram::{Diagram, DiagramResult, SummaryDiagramComponent, SummaryDiagramEdge};
use dal::{AttributeValue, Component, DalContext, Schema, SchemaVariant};
use dal::{ComponentType, InputSocket, OutputSocket};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::helpers::{connect_components_with_socket_names, create_component_for_schema_name};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use std::collections::HashMap;

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
                let possible_match = Component::find_potential_inferred_connection_to_input_socket(
                    ctx,
                    input_socket_match,
                )
                .await
                .expect("couldn't find implicit inputs");
                assert!(possible_match.is_some());
                let travis_output_match = possible_match.expect("has a value");
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
                let possible_match = Component::find_potential_inferred_connection_to_input_socket(
                    ctx,
                    input_socket_match,
                )
                .await
                .expect("couldn't find implicit inputs");
                assert!(possible_match.is_none());
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
    let even_frame = create_component_for_schema_name(ctx, "large even lego", "even").await;

    let _ = even_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await;
    let odd_component = create_component_for_schema_name(ctx, "large odd lego", "odd1").await;
    let _ = odd_component.set_type(ctx, ComponentType::Component).await;
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
    let odd_component_2 = create_component_for_schema_name(ctx, "large odd lego", "odd2").await;
    let _ = odd_component_2
        .set_type(ctx, ComponentType::Component)
        .await;

    connect_components_with_socket_names(ctx, even_frame.id(), "one", odd_component_2.id(), "one")
        .await;
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
    let odd_component_1_mat_view = AttributeValue::get_by_id(ctx, odd_component_1_av)
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
    let odd_component_2_mat_view = AttributeValue::get_by_id(ctx, odd_component_2_av)
        .await
        .expect("got av")
        .view(ctx)
        .await
        .expect("got mat view")
        .expect("has value");
    assert_eq!(odd_component_2_mat_view, serde_json::json!("1"));
}

#[test]
async fn simple_down_frames_no_nesting(ctx: &mut DalContext) {
    let even_frame = create_component_for_schema_name(ctx, "large even lego", "even").await;

    let _ = even_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await;

    let odd_component = create_component_for_schema_name(ctx, "large odd lego", "odd").await;
    let _ = odd_component.set_type(ctx, ComponentType::Component).await;
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
    // the output socket value is updated with Jason Mraz
    let even_frame_sv_id = Component::schema_variant_id(ctx, even_frame.id())
        .await
        .expect("could not get sv id");
    let even_frame_output_sockets = even_frame
        .output_socket_attribute_values(ctx)
        .await
        .expect("could not get output socket avs");
    let output_id = OutputSocket::find_with_name(ctx, "one", even_frame_sv_id)
        .await
        .expect("could not get output socket by name")
        .expect("is some");
    let one_av_id = even_frame_output_sockets
        .get(&output_id.id())
        .expect("found a value");
    let one_av = AttributeValue::get_by_id(ctx, one_av_id.attribute_value_id)
        .await
        .expect("could not get attribute value");
    let one_mat_view = one_av
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");

    assert_eq!(one_mat_view, serde_json::json!("1"));
    let odd_sv_id = Component::schema_variant_id(ctx, odd_component.id())
        .await
        .expect("could not get sv id");
    let one_input_id = InputSocket::find_with_name(ctx, "one", odd_sv_id)
        .await
        .expect("could not get input socket by name")
        .expect("is some");

    let odd_inputs = odd_component
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get attribute values");

    let one_input_match = odd_inputs.get(&one_input_id.id()).expect("found a value");

    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");
    assert_eq!(one_input_mat_view, serde_json::json!("1"));

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");

    assert_eq!(one_input_mat_view, serde_json::json!("1"));
}
#[test]
async fn simple_down_frames_nesting(ctx: &mut DalContext) {
    // create parent frame
    let even_parent_frame =
        create_component_for_schema_name(ctx, "large even lego", "even parent").await;

    let _ = even_parent_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await;
    // create child frame
    let even_child_frame =
        create_component_for_schema_name(ctx, "medium even lego", "even child").await;

    let _ = even_child_frame
        .set_type(ctx, ComponentType::ConfigurationFrameDown)
        .await;
    // insert child frame into parent frame
    Frame::upsert_parent(ctx, even_child_frame.id(), even_parent_frame.id())
        .await
        .expect("can upsert parent");
    // create component
    let odd_component = create_component_for_schema_name(ctx, "large odd lego", "odd").await;
    let _ = odd_component.set_type(ctx, ComponentType::Component).await;
    // insert component into CHILD frame
    Frame::upsert_parent(ctx, odd_component.id(), even_child_frame.id())
        .await
        .expect("can upsert to child frame");
    let one_attribute_value_id = even_parent_frame
        .attribute_values_for_prop(ctx, &["root", "domain", "five"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(ctx, one_attribute_value_id, Some(serde_json::json!["5"]))
        .await
        .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // the output socket value is updated with 1
    let even_parent_frame_sv_id = Component::schema_variant_id(ctx, even_parent_frame.id())
        .await
        .expect("works");
    let even_frame_output_sockets = even_parent_frame
        .output_socket_attribute_values(ctx)
        .await
        .expect("could not get output socket avs");
    let output_id = OutputSocket::find_with_name(ctx, "five", even_parent_frame_sv_id)
        .await
        .expect("could not find output socket by name")
        .expect("is some");
    let one_av_id = even_frame_output_sockets
        .get(&output_id.id())
        .expect("could not get output socket av");
    let one_av = AttributeValue::get_by_id(ctx, one_av_id.attribute_value_id)
        .await
        .expect("could not get attribute value");
    let one_mat_view = one_av
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");

    assert_eq!(one_mat_view, serde_json::json!("5"));

    // the component is updated with 5
    let odd_component_sv_id = Component::schema_variant_id(ctx, odd_component.id())
        .await
        .expect("could not get sv id");
    let one_component_input_socket = InputSocket::find_with_name(ctx, "five", odd_component_sv_id)
        .await
        .expect("could not find input socket by name")
        .expect("is some");

    let odd_inputs = odd_component
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get input socket avs");

    let one_input_match = odd_inputs
        .get(&one_component_input_socket.id())
        .expect("could not get input socket");

    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");
    assert_eq!(one_input_mat_view, serde_json::json!("5"));
    // now let's update the parent frame to a value that the child also has
    let one_attribute_value_id = even_parent_frame
        .attribute_values_for_prop(ctx, &["root", "domain", "three"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(ctx, one_attribute_value_id, Some(serde_json::json!["4"]))
        .await
        .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the component doesn't get the update as the child frame is a closer match and overrides it
    // the component is updated with 5
    let odd_component_sv_id = Component::schema_variant_id(ctx, odd_component.id())
        .await
        .expect("could not get sv id");
    let one_component_input_socket = InputSocket::find_with_name(ctx, "three", odd_component_sv_id)
        .await
        .expect("could not find input socket by name")
        .expect("is some");

    let odd_inputs = odd_component
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get input socket avs");

    let one_input_match = odd_inputs
        .get(&one_component_input_socket.id())
        .expect("could not get input value");

    assert!(
        AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
            .await
            .expect("could not get attribute value by id")
            .view(ctx)
            .await
            .expect("could not get materialized view")
            .is_none()
    );
    // now let's update the child frame's same socket to a value the component should take
    let one_attribute_value_id = even_child_frame
        .attribute_values_for_prop(ctx, &["root", "domain", "three"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(ctx, one_attribute_value_id, Some(serde_json::json!["3"]))
        .await
        .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the component gets the update as the child frame is a closer match
    let odd_component_sv_id = Component::schema_variant_id(ctx, odd_component.id())
        .await
        .expect("could not get sv id");
    let one_component_input_socket = InputSocket::find_with_name(ctx, "three", odd_component_sv_id)
        .await
        .expect("could not find input socket by name")
        .expect("is some");

    let odd_inputs = odd_component
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get input socket attribute values");

    let one_input_match = odd_inputs
        .get(&one_component_input_socket.id())
        .expect("could not get input socket");

    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get av by id")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");
    assert_eq!(one_input_mat_view, serde_json::json!("3"));
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
    let one_component_input_socket = InputSocket::find_with_name(ctx, "three", odd_component_sv_id)
        .await
        .expect("could not find input socket by name")
        .expect("is some");

    let one_input_match =
        Component::input_socket_match(ctx, odd_component.id(), one_component_input_socket.id())
            .await
            .expect("could not find input socket match")
            .expect("is some");

    let three_socket_id = OutputSocket::find_with_name(ctx, "three", even_parent_frame_sv_id)
        .await
        .expect("could not find output socket by name")
        .expect("is some");
    let three_av_id =
        Component::output_socket_match(ctx, even_parent_frame.id(), three_socket_id.id())
            .await
            .expect("could not get output socket av")
            .expect("is some");

    let _one_attribute_value_id = even_parent_frame
        .attribute_values_for_prop(ctx, &["root", "domain", "three"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");
    let one_output_mat_view = AttributeValue::get_by_id(ctx, three_av_id.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not find materialized view")
        .expect("is some");
    assert_eq!(one_output_mat_view, serde_json::json!("4"));
    let maybe_match =
        Component::find_potential_inferred_connection_to_input_socket(ctx, one_input_match)
            .await
            .expect("could not find inferred input socket")
            .expect("is some");
    assert_eq!(
        maybe_match.attribute_value_id,
        three_av_id.attribute_value_id
    );
    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");
    assert_eq!(one_input_mat_view, serde_json::json!("4"));
}
#[test]
async fn simple_up_frames_some_nesting(ctx: &mut DalContext) {
    let even_component = create_component_for_schema_name(ctx, "small even lego", "even").await;

    let _ = even_component.set_type(ctx, ComponentType::Component).await;

    let odd_up_frame = create_component_for_schema_name(ctx, "large odd lego", "odd").await;
    let _ = odd_up_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await;
    Frame::upsert_parent(ctx, even_component.id(), odd_up_frame.id())
        .await
        .expect("could not upsert parent");
    // Change attribute value for one on the component
    let type_attribute_value_id = even_component
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

    // the output socket value is updated with "1"
    let even_component_sv_id = Component::schema_variant_id(ctx, even_component.id())
        .await
        .expect("could not get sv id");
    let even_component_output_sockets = even_component
        .output_socket_attribute_values(ctx)
        .await
        .expect("could not get output avs");
    let output_id = OutputSocket::find_with_name(ctx, "one", even_component_sv_id)
        .await
        .expect("could not find output socket by name")
        .expect("is some");
    let even_component_output_av = even_component_output_sockets
        .get(&output_id.id())
        .expect("found a value")
        .attribute_value_id;
    let one_av = AttributeValue::get_by_id(ctx, even_component_output_av)
        .await
        .expect("could not get attribute value");
    let one_mat_view = one_av
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");

    assert_eq!(one_mat_view, serde_json::json!("1"));

    // // make sure component output socket matches on the up frames input socket
    let odd_up_frame_sv_id = Component::schema_variant_id(ctx, odd_up_frame.id())
        .await
        .expect("could not get sv id");
    let odd_up_frame_input_socket = InputSocket::find_with_name(ctx, "one", odd_up_frame_sv_id)
        .await
        .expect("could not get input socket by name")
        .expect("is some");

    let odd_inputs = odd_up_frame
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get input socket avs");

    let one_input_match = odd_inputs
        .get(&odd_up_frame_input_socket.id())
        .expect("could not find input by id");

    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");
    assert_eq!(one_input_mat_view, serde_json::json!("1"));

    //let's add another component to the frame, to drive the "3" input socket

    let another_even_component =
        create_component_for_schema_name(ctx, "medium even lego", "another even").await;

    let _ = another_even_component
        .set_type(ctx, ComponentType::Component)
        .await;
    Frame::upsert_parent(ctx, another_even_component.id(), odd_up_frame.id())
        .await
        .expect("could not upsert parent");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // Change attribute value for three on the component
    let type_attribute_value_id = another_even_component
        .attribute_values_for_prop(ctx, &["root", "domain", "three"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(ctx, type_attribute_value_id, Some(serde_json::json!("3")))
        .await
        .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the output socket value is updated with "3"
    let another_even_component_sv_id =
        Component::schema_variant_id(ctx, another_even_component.id())
            .await
            .expect("could not get sv id");
    let another_even_component_output_sockets = another_even_component
        .output_socket_attribute_values(ctx)
        .await
        .expect("could not get output socket avs");
    let another_even_output_socket =
        OutputSocket::find_with_name(ctx, "three", another_even_component_sv_id)
            .await
            .expect("found a name")
            .expect("value exists");
    let even_component_output_av = another_even_component_output_sockets
        .get(&another_even_output_socket.id())
        .expect("could not find output socket by id")
        .attribute_value_id;
    let another_three_av = AttributeValue::get_by_id(ctx, even_component_output_av)
        .await
        .expect("could not get attribute value");
    let one_mat_view = another_three_av
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");

    assert_eq!(one_mat_view, serde_json::json!("3"));
    // // make sure component output socket matches on the up frames input socket
    let odd_up_frame_sv_id = Component::schema_variant_id(ctx, odd_up_frame.id())
        .await
        .expect("could not get sv id");
    let odd_up_frame_input_socket = InputSocket::find_with_name(ctx, "three", odd_up_frame_sv_id)
        .await
        .expect("could not find input socket by name")
        .expect("is some");

    let odd_inputs = odd_up_frame
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get input socket avs");

    let one_input_match = odd_inputs
        .get(&odd_up_frame_input_socket.id())
        .expect("could not get input socket");

    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");
    assert_eq!(one_input_mat_view, serde_json::json!("3"));

    //now let's drop that up frame into an even up frame, driving the even values
    let even_up_frame =
        create_component_for_schema_name(ctx, "large even lego", "another even").await;

    let _ = even_up_frame
        .set_type(ctx, ComponentType::ConfigurationFrameUp)
        .await;
    Frame::upsert_parent(ctx, odd_up_frame.id(), even_up_frame.id())
        .await
        .expect("could not upsert parent frame");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // Change attribute value for two on the odd up frame
    let odd_up_frame = Component::get_by_id(ctx, odd_up_frame.id())
        .await
        .expect("could not get component");
    let odd_up_frame_two_av_id = odd_up_frame
        .attribute_values_for_prop(ctx, &["root", "domain", "two"])
        .await
        .expect("could not find attribute values for prop")
        .into_iter()
        .next()
        .expect("could not get type attribute value id");

    AttributeValue::update(ctx, odd_up_frame_two_av_id, Some(serde_json::json!("2")))
        .await
        .expect("could not update attribute value");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // the output socket value is updated with "2"
    let odd_up_frame_sv_id = Component::schema_variant_id(ctx, odd_up_frame.id())
        .await
        .expect("could not get sv id");
    let odd_up_frame_output_sockets = odd_up_frame
        .output_socket_attribute_values(ctx)
        .await
        .expect("could not get output sockets");
    let odd_up_frame_output_socket = OutputSocket::find_with_name(ctx, "two", odd_up_frame_sv_id)
        .await
        .expect("could not find output socket by name")
        .expect("output socket is some");
    let odd_up_frame_output_av = odd_up_frame_output_sockets
        .get(&odd_up_frame_output_socket.id())
        .expect("found a value")
        .attribute_value_id;
    let odd_two_av = AttributeValue::get_by_id(ctx, odd_up_frame_output_av)
        .await
        .expect("could not get attribute value");
    let odd_two_mat_view = odd_two_av
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("is some");

    assert_eq!(odd_two_mat_view, serde_json::json!("2"));
    // even up frame input socket matches odd up frame output socket

    let even_up_frame = Component::get_by_id(ctx, even_up_frame.id())
        .await
        .expect("could not get component");
    let even_up_frame_sv_id = Component::schema_variant_id(ctx, even_up_frame.id())
        .await
        .expect("could not get sv id");
    let even_up_frame_input_socket = InputSocket::find_with_name(ctx, "two", even_up_frame_sv_id)
        .await
        .expect("could not get input socket by name")
        .expect("input socket is some");

    let even_inputs = even_up_frame
        .input_socket_attribute_values(ctx)
        .await
        .expect("could not get input socket avs");

    let one_input_match = even_inputs
        .get(&even_up_frame_input_socket.id())
        .expect("could not get input socket");

    let output_match =
        Component::find_potential_inferred_connection_to_input_socket(ctx, *one_input_match)
            .await
            .expect("could not get inferred connection")
            .expect("inferred connection is some");
    assert_eq!(odd_up_frame_output_av, output_match.attribute_value_id);

    let one_input_mat_view = AttributeValue::get_by_id(ctx, one_input_match.attribute_value_id)
        .await
        .expect("could not get attribute value")
        .view(ctx)
        .await
        .expect("could not get materialized view")
        .expect("materialized view is some");
    assert_eq!(one_input_mat_view, serde_json::json!("2"));
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
