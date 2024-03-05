use dal::component::frame::{Frame, FrameError};
use dal::diagram::{Diagram, DiagramResult, EdgeId, SummaryDiagramComponent, SummaryDiagramEdge};
use dal::{
    AttributeValue, Component, DalContext, InputSocket, OutputSocket, Schema, SchemaVariant,
};
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
    let starfield_component = Component::new(ctx, "parent", starfield_schema_variant.id(), None)
        .await
        .expect("could not create component");
    let fallout_component = Component::new(ctx, "child", fallout_schema_variant.id(), None)
        .await
        .expect("could not create component");

    // Attempt to attach a child to a parent that is a not a frame.
    match Frame::attach_child_to_parent(ctx, starfield_component.id(), fallout_component.id()).await
    {
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

    ctx.blocking_commit()
        .await
        .expect("could not perform blocking commit");

    // Now that the parent is a frame, attempt to attach the child.
    Frame::attach_child_to_parent(ctx, starfield_component.id(), fallout_component.id())
        .await
        .expect("could not attach child to parent");

    ctx.blocking_commit()
        .await
        .expect("could not perform blocking commit");

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
            "starfield" => starfield_parent_node_id = Some(component.parent_node_id),
            "fallout" => fallout_parent_node_id = Some(component.parent_node_id),
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
async fn multiple_frames_with_complex_connections_no_nesting(ctx: &mut DalContext) {
    let region_schema = Schema::find_by_name(ctx, "Region")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");
    let ec2_schema = Schema::find_by_name(ctx, "EC2 Instance")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");
    let ami_schema = Schema::find_by_name(ctx, "AMI")
        .await
        .expect("could not perform find by name")
        .expect("schema not found by name");

    // Collect schema variants.
    let region_schema_variant_id = SchemaVariant::list_for_schema(ctx, region_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();
    let ec2_schema_variant_id = SchemaVariant::list_for_schema(ctx, ec2_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();
    let ami_schema_variant_id = SchemaVariant::list_for_schema(ctx, ami_schema.id())
        .await
        .expect("could not list schema variants")
        .pop()
        .expect("no schema variants found")
        .id();

    // Scenario 1: create an AWS region frame.
    let first_region_frame_name = "first region frame";
    let first_region_frame =
        Component::new(ctx, first_region_frame_name, region_schema_variant_id, None)
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

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert!(first_region_frame_assembled.parent_node_id.is_none());
    }

    // Scenario 2: create an AMI and attach to region frame
    let first_ami_component_name = "first ami component";
    let first_ami_component =
        Component::new(ctx, first_ami_component_name, ami_schema_variant_id, None)
            .await
            .expect("could not create component");
    Frame::attach_child_to_parent(ctx, first_region_frame.id(), first_ami_component.id())
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
            1,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenario 3: add another aws region frame on its own.
    let second_region_frame_name = "second region frame";
    let second_region_frame = Component::new(
        ctx,
        second_region_frame_name,
        region_schema_variant_id,
        None,
    )
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
            1,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");
        let second_region_frame_assembled = diagram
            .components
            .get(second_region_frame_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            second_region_frame.id(),                   // expected
            second_region_frame_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert!(second_region_frame_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenarios 4 and 5: create another ami, but place it outside of both frames. Then, drag it onto the second region
    // frame. Since we are working with dal integration tests and not sdf routes, we combine these two scenarios.
    let second_ami_component_name = "second ami component";
    let second_ami_component =
        Component::new(ctx, second_ami_component_name, ami_schema_variant_id, None)
            .await
            .expect("could not create component");
    Frame::attach_child_to_parent(ctx, second_region_frame.id(), second_ami_component.id())
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
            2,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");
        let second_region_frame_assembled = diagram
            .components
            .get(second_region_frame_name)
            .expect("could not get component by name");
        let second_ami_component_assembled = diagram
            .components
            .get(second_ami_component_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            second_region_frame.id(),                   // expected
            second_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            second_ami_component.id(),                   // expected
            second_ami_component_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert!(second_region_frame_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            second_region_frame.id(), // expected
            second_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenarios 6 and 7: create an ec2 instance, but place it outside of both frames. Then, drag it onto the first
    // region frame. Since we are working with dal integration tests and not sdf routes, we combine these two scenarios.
    let first_ec2_instance_component_name = "first ec2 instance component";
    let first_ec2_instance_component = Component::new(
        ctx,
        first_ec2_instance_component_name,
        ec2_schema_variant_id,
        None,
    )
    .await
    .expect("could not create component");
    Frame::attach_child_to_parent(
        ctx,
        first_region_frame.id(),
        first_ec2_instance_component.id(),
    )
    .await
    .expect("could not attach child to parent");

    // Validate Scenarios 6 and 7
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            5,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            3,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");
        let second_region_frame_assembled = diagram
            .components
            .get(second_region_frame_name)
            .expect("could not get component by name");
        let second_ami_component_assembled = diagram
            .components
            .get(second_ami_component_name)
            .expect("could not get component by name");
        let first_ec2_instance_component_assembled = diagram
            .components
            .get(first_ec2_instance_component_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            second_region_frame.id(),                   // expected
            second_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            second_ami_component.id(),                   // expected
            second_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                   // expected
            first_ec2_instance_component_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert!(second_region_frame_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            second_region_frame.id(), // expected
            second_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            first_region_frame.id(), // expected
            first_ec2_instance_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenario 8: draw an edge between the first ami and the first ec2 using the "Image ID" sockets. Both should exist
    // within the first region frame.
    let image_id_socket_name = "Image ID";
    let image_id_ami_output_socket_id =
        OutputSocket::find_with_name(ctx, image_id_socket_name, ami_schema_variant_id)
            .await
            .expect("could not perform output socket find by name")
            .expect("no output socket found")
            .id();
    let image_id_ec2_instance_input_socket_id =
        InputSocket::find_with_name(ctx, image_id_socket_name, ec2_schema_variant_id)
            .await
            .expect("could not perform input socket find by name")
            .expect("no input socket found")
            .id();
    let image_id_ami_to_ec2_instance_attribute_prototype_argument_id = Component::connect(
        ctx,
        first_ami_component.id(),
        image_id_ami_output_socket_id,
        first_ec2_instance_component.id(),
        image_id_ec2_instance_input_socket_id,
    )
    .await
    .expect("could not perform connection");

    // Validate Scenario 8
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            5,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            4,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");
        let second_region_frame_assembled = diagram
            .components
            .get(second_region_frame_name)
            .expect("could not get component by name");
        let second_ami_component_assembled = diagram
            .components
            .get(second_ami_component_name)
            .expect("could not get component by name");
        let first_ec2_instance_component_assembled = diagram
            .components
            .get(first_ec2_instance_component_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            second_region_frame.id(),                   // expected
            second_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            second_ami_component.id(),                   // expected
            second_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                   // expected
            first_ec2_instance_component_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert!(second_region_frame_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            second_region_frame.id(), // expected
            second_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            first_region_frame.id(), // expected
            first_ec2_instance_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );

        let image_id_ami_to_ec2_instance_edge_assembled = diagram
            .edges
            .get(&image_id_ami_to_ec2_instance_attribute_prototype_argument_id)
            .expect("could not get edge by id");
        assert_eq!(
            first_ami_component.id(),                                 // expected
            image_id_ami_to_ec2_instance_edge_assembled.from_node_id  // actual
        );
        assert_eq!(
            image_id_ami_output_socket_id,                              // expected
            image_id_ami_to_ec2_instance_edge_assembled.from_socket_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                      // expected
            image_id_ami_to_ec2_instance_edge_assembled.to_node_id  // actual
        );
        assert_eq!(
            image_id_ec2_instance_input_socket_id, // expected
            image_id_ami_to_ec2_instance_edge_assembled.to_socket_id  // actual
        );
    }

    // Scenario 9: create a third AMI outside of both frames.
    let third_ami_component_name = "third ami component";
    let third_ami_component =
        Component::new(ctx, third_ami_component_name, ami_schema_variant_id, None)
            .await
            .expect("could not create component");

    // Validate Scenario 9
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            6,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            4,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");
        let second_region_frame_assembled = diagram
            .components
            .get(second_region_frame_name)
            .expect("could not get component by name");
        let second_ami_component_assembled = diagram
            .components
            .get(second_ami_component_name)
            .expect("could not get component by name");
        let first_ec2_instance_component_assembled = diagram
            .components
            .get(first_ec2_instance_component_name)
            .expect("could not get component by name");
        let third_ami_component_assembled = diagram
            .components
            .get(third_ami_component_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            second_region_frame.id(),                   // expected
            second_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            second_ami_component.id(),                   // expected
            second_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                   // expected
            first_ec2_instance_component_assembled.component_id  // actual
        );
        assert_eq!(
            third_ami_component.id(),                   // expected
            third_ami_component_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert!(second_region_frame_assembled.parent_node_id.is_none());
        assert!(third_ami_component_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            second_region_frame.id(), // expected
            second_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            first_region_frame.id(), // expected
            first_ec2_instance_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );

        let image_id_ami_to_ec2_instance_edge_assembled = diagram
            .edges
            .get(&image_id_ami_to_ec2_instance_attribute_prototype_argument_id)
            .expect("could not get edge by id");
        assert_eq!(
            first_ami_component.id(),                                 // expected
            image_id_ami_to_ec2_instance_edge_assembled.from_node_id  // actual
        );
        assert_eq!(
            image_id_ami_output_socket_id,                              // expected
            image_id_ami_to_ec2_instance_edge_assembled.from_socket_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                      // expected
            image_id_ami_to_ec2_instance_edge_assembled.to_node_id  // actual
        );
        assert_eq!(
            image_id_ec2_instance_input_socket_id, // expected
            image_id_ami_to_ec2_instance_edge_assembled.to_socket_id  // actual
        );
    }

    // Scenario 10: draw an edge (do not drag the component or place it onto a frame) between the "Region" socket of the
    // second region frame and the "Region" socket of the third ami.
    let region_socket_name = "Region";
    let region_region_output_socket_id =
        OutputSocket::find_with_name(ctx, region_socket_name, region_schema_variant_id)
            .await
            .expect("could not perform output socket find by name")
            .expect("no output socket found")
            .id();
    let region_ami_input_socket_id =
        InputSocket::find_with_name(ctx, region_socket_name, ami_schema_variant_id)
            .await
            .expect("could not perform input socket find by name")
            .expect("no input socket found")
            .id();
    let region_region_to_ami_attribute_prototype_argument_id = Component::connect(
        ctx,
        second_region_frame.id(),
        region_region_output_socket_id,
        third_ami_component.id(),
        region_ami_input_socket_id,
    )
    .await
    .expect("could not perform connection");

    // Validate Scenario 10
    {
        let diagram = DiagramByKey::assemble(ctx)
            .await
            .expect("could not assemble diagram");
        assert_eq!(
            6,                        // expected
            diagram.components.len()  // actual
        );
        assert_eq!(
            5,                   // expected
            diagram.edges.len()  // actual
        );

        let first_region_frame_assembled = diagram
            .components
            .get(first_region_frame_name)
            .expect("could not get component by name");
        let first_ami_component_assembled = diagram
            .components
            .get(first_ami_component_name)
            .expect("could not get component by name");
        let second_region_frame_assembled = diagram
            .components
            .get(second_region_frame_name)
            .expect("could not get component by name");
        let second_ami_component_assembled = diagram
            .components
            .get(second_ami_component_name)
            .expect("could not get component by name");
        let first_ec2_instance_component_assembled = diagram
            .components
            .get(first_ec2_instance_component_name)
            .expect("could not get component by name");
        let third_ami_component_assembled = diagram
            .components
            .get(third_ami_component_name)
            .expect("could not get component by name");

        assert_eq!(
            first_region_frame.id(),                   // expected
            first_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            first_ami_component.id(),                   // expected
            first_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            second_region_frame.id(),                   // expected
            second_region_frame_assembled.component_id  // actual
        );
        assert_eq!(
            second_ami_component.id(),                   // expected
            second_ami_component_assembled.component_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                   // expected
            first_ec2_instance_component_assembled.component_id  // actual
        );
        assert_eq!(
            third_ami_component.id(),                   // expected
            third_ami_component_assembled.component_id  // actual
        );

        assert!(first_region_frame_assembled.parent_node_id.is_none());
        assert!(second_region_frame_assembled.parent_node_id.is_none());
        assert!(third_ami_component_assembled.parent_node_id.is_none());
        assert_eq!(
            first_region_frame.id(), // expected
            first_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            second_region_frame.id(), // expected
            second_ami_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            first_region_frame.id(), // expected
            first_ec2_instance_component_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );

        let image_id_ami_to_ec2_instance_edge_assembled = diagram
            .edges
            .get(&image_id_ami_to_ec2_instance_attribute_prototype_argument_id)
            .expect("could not get edge by id");
        assert_eq!(
            first_ami_component.id(),                                 // expected
            image_id_ami_to_ec2_instance_edge_assembled.from_node_id  // actual
        );
        assert_eq!(
            image_id_ami_output_socket_id,                              // expected
            image_id_ami_to_ec2_instance_edge_assembled.from_socket_id  // actual
        );
        assert_eq!(
            first_ec2_instance_component.id(),                      // expected
            image_id_ami_to_ec2_instance_edge_assembled.to_node_id  // actual
        );
        assert_eq!(
            image_id_ec2_instance_input_socket_id, // expected
            image_id_ami_to_ec2_instance_edge_assembled.to_socket_id  // actual
        );

        let region_region_to_ami_edge_assembled = diagram
            .edges
            .get(&region_region_to_ami_attribute_prototype_argument_id)
            .expect("could not get edge by id");
        assert_eq!(
            second_region_frame.id(),                         // expected
            region_region_to_ami_edge_assembled.from_node_id  // actual
        );
        assert_eq!(
            region_region_output_socket_id,                     // expected
            region_region_to_ami_edge_assembled.from_socket_id  // actual
        );
        assert_eq!(
            third_ami_component.id(),                       // expected
            region_region_to_ami_edge_assembled.to_node_id  // actual
        );
        assert_eq!(
            region_ami_input_socket_id,                       // expected
            region_region_to_ami_edge_assembled.to_socket_id  // actual
        );
    }
}

struct DiagramByKey {
    pub components: HashMap<String, SummaryDiagramComponent>,
    pub edges: HashMap<EdgeId, SummaryDiagramEdge>,
}

impl DiagramByKey {
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let diagram = Diagram::assemble(ctx).await?;

        let mut components = HashMap::new();
        for component in &diagram.components {
            components.insert(component.display_name.clone(), component.to_owned());
        }

        let mut edges = HashMap::new();
        for edge in &diagram.edges {
            edges.insert(edge.edge_id, edge.to_owned());
        }

        Ok(Self { components, edges })
    }
}
