use dal::component::frame::{Frame, FrameError, FrameResult};
use dal::diagram::Diagram;
use dal::{AttributeValue, Component, DalContext, Schema, SchemaVariant};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn convert_component_to_frame_and_attach(ctx: &mut DalContext) {
    // Collect the test exclusive schemas that we need.
    let mut starfield_schema = None;
    let mut fallout_schema = None;
    for schema in Schema::list(ctx).await.expect("list schemas") {
        match schema.name.as_str() {
            "starfield" => starfield_schema = Some(schema),
            "fallout" => fallout_schema = Some(schema),
            _ => {}
        }
    }
    let starfield_schema = starfield_schema.expect("could not find starfield schema");
    let fallout_schema = fallout_schema.expect("could not find fallout schema");

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
        &ctx,
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
        match component.schema_name() {
            "starfield" => starfield_parent_node_id = Some(component.parent_node_id()),
            "fallout" => fallout_parent_node_id = Some(component.parent_node_id()),
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
