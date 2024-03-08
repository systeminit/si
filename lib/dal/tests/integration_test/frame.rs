use dal::component::frame::{Frame, FrameError};
use dal::diagram::{Diagram, DiagramResult, EdgeId, SummaryDiagramComponent, SummaryDiagramEdge};
use dal::{AttributeValue, Component, DalContext, Schema, SchemaVariant};
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
            new_era_taylor_swift.id(),                   // expected
            new_era_taylor_swift_assembled.component_id  // actual
        );
        assert!(new_era_taylor_swift_assembled.parent_node_id.is_none());
    }

    // Scenario 2: create a kelce component and attach to swifty frame
    let travis_kelce_component_name = "travis kelce";
    let travis_kelce_component =
        Component::new(ctx, travis_kelce_component_name, fallout_schema_variant_id)
            .await
            .expect("could not create component");
    Frame::attach_child_to_parent(ctx, new_era_taylor_swift.id(), travis_kelce_component.id())
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
        for (name, SummaryDiagramComponent { sockets, .. }) in &diagram.components {
            println!("{name}, {}", sockets)
        }
        assert_eq!(
            1,                   // expected
            diagram.edges.len()  // actual
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
            new_era_taylor_swift.id(),                   // expected
            new_era_taylor_swift_assembled.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),         // expected
            travis_kelce_assembled.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.parent_node_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .parent_node_id
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
            1,                   // expected
            diagram.edges.len()  // actual
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
            new_era_taylor_swift.id(),                   // expected
            new_era_taylor_swift_assembled.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),         // expected
            travis_kelce_assembled.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                   // expected
            country_era_taylor_swift_assembled.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.parent_node_id.is_none());
        assert!(country_era_taylor_swift_assembled.parent_node_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
    }

    // Scenarios 4 and 5: create a mama kelce component, but place it outside of both frames. Then, drag it onto the second swifty
    // frame.
    let mama_kelce_name = "mama kelce";
    let mama_kelce = Component::new(ctx, mama_kelce_name, fallout_schema_variant_id)
        .await
        .expect("could not create component");
    Frame::attach_child_to_parent(ctx, country_era_taylor_swift.id(), mama_kelce.id())
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
            new_era_taylor_swift.id(),                   // expected
            new_era_taylor_swift_assembled.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),         // expected
            travis_kelce_assembled.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                   // expected
            country_era_taylor_swift_assembled.component_id  // actual
        );
        assert_eq!(
            mama_kelce.id(),                   // expected
            mama_kelce_assembled.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.parent_node_id.is_none());
        assert!(country_era_taylor_swift_assembled.parent_node_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(), // expected
            mama_kelce_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
    }

    // // Scenarios 6: Country Era taylor Swift within New Era Taylor Swift.
    Frame::attach_child_to_parent(
        ctx,
        new_era_taylor_swift.id(),
        country_era_taylor_swift.id(),
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
            2,                   // expected
            diagram.edges.len()  // actual
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
            new_era_taylor_swift.id(),                   // expected
            new_era_taylor_swift_assembled.component_id  // actual
        );
        assert_eq!(
            travis_kelce_component.id(),         // expected
            travis_kelce_assembled.component_id  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(),                   // expected
            country_era_taylor_swift_assembled.component_id  // actual
        );
        assert_eq!(
            mama_kelce.id(),                   // expected
            mama_kelce_assembled.component_id  // actual
        );

        assert!(new_era_taylor_swift_assembled.parent_node_id.is_none());
        assert_eq!(
            new_era_taylor_swift.id(),
            country_era_taylor_swift_assembled
                .parent_node_id
                .expect("no parent node id")
        );
        assert_eq!(
            new_era_taylor_swift.id(), // expected
            travis_kelce_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
        );
        assert_eq!(
            country_era_taylor_swift.id(), // expected
            mama_kelce_assembled
                .parent_node_id
                .expect("no parent node id")  // actual
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
