use dal::{
    AttributeValue,
    Component,
    DalContext,
    InputSocket,
    OutputSocket,
    Schema,
    SchemaVariant,
    attribute::prototype::argument::AttributePrototypeArgument,
    diagram::Diagram,
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
        create_named_component_for_schema_variant_on_default_view,
    },
    test,
};
use serde::Deserialize;

#[test]
async fn make_multiple_trees(ctx: &mut DalContext) {
    // create 2 even legos
    let even_lego_1 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large even lego",
        "even lego 1",
    )
    .await
    .expect("could not create component");
    let even_lego_2 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large even lego",
        "even lego 2",
    )
    .await
    .expect("could not create component");

    let even_sv_id = even_lego_1
        .schema_variant(ctx)
        .await
        .expect("found schema variant");
    // get output socket id
    let output_socket_id = OutputSocket::find_with_name(ctx, "one", even_sv_id.id())
        .await
        .expect("found output socket")
        .expect("output socket exists")
        .id();
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    //create 2 odd legos
    let odd_lego_1 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large odd lego",
        "odd lego 1",
    )
    .await
    .expect("could not create component");
    let odd_lego_2 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large odd lego",
        "odd lego 2",
    )
    .await
    .expect("could not create component");
    let odd_sv = odd_lego_1
        .schema_variant(ctx)
        .await
        .expect("found schema variant");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    // get input socket id
    let input_socket_id = InputSocket::find_with_name(ctx, "one", odd_sv.id())
        .await
        .expect("found input socket")
        .expect("exists")
        .id();

    //connect each of them
    let _result = Component::connect(
        ctx,
        even_lego_1.id(),
        output_socket_id,
        odd_lego_1.id(),
        input_socket_id,
    )
    .await
    .expect("could connect")
    .expect("apa exists");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let _result_2 = Component::connect(
        ctx,
        even_lego_2.id(),
        output_socket_id,
        odd_lego_2.id(),
        input_socket_id,
    )
    .await
    .expect("could connect")
    .expect("apa exists");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("got diagram");
    assert_eq!(
        2,                   // expected
        diagram.edges.len()  // actual
    );
    assert_eq!(
        4,                        // expected
        diagram.components.len()  // actual
    );
}
#[test]
async fn make_chain_remove_middle(ctx: &mut DalContext) {
    // make chain of odd lego 1 -> even lego 1 -> odd lego 2
    let odd_component_1 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large odd lego",
        "odd lego 1",
    )
    .await
    .expect("could not create component");
    let even_component_1 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large even lego",
        "even lego 1",
    )
    .await
    .expect("could not create component");
    let odd_component_2 = create_component_for_default_schema_name_in_default_view(
        ctx,
        "large odd lego",
        "odd lego 2",
    )
    .await
    .expect("could not create component");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("got diagram");
    assert_eq!(
        0,                   // expected
        diagram.edges.len()  // actual
    );
    assert_eq!(
        3,                        // expected
        diagram.components.len()  // actual
    );

    // connect first two components
    let odd_sv_id = odd_component_1
        .schema_variant(ctx)
        .await
        .expect("got schema variant")
        .id();
    let odd_output_socket_id = OutputSocket::find_with_name(ctx, "two", odd_sv_id)
        .await
        .expect("got output socket")
        .expect("output socket exists")
        .id();

    let even_sv_id = even_component_1
        .schema_variant(ctx)
        .await
        .expect("got schema variant")
        .id();

    let even_input_socket_id = InputSocket::find_with_name(ctx, "two", even_sv_id)
        .await
        .expect("found input socket")
        .expect("input socket exists")
        .id();

    let result = Component::connect(
        ctx,
        odd_component_1.id(),
        odd_output_socket_id,
        even_component_1.id(),
        even_input_socket_id,
    )
    .await
    .expect("could connect")
    .expect("apa exists");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("got diagram");
    assert_eq!(
        1,                   // expected
        diagram.edges.len()  // actual
    );
    assert_eq!(
        3,                        // expected
        diagram.components.len()  // actual
    );
    let apa = AttributePrototypeArgument::get_by_id(ctx, result)
        .await
        .expect("found apa");
    let targets = apa.targets().expect("targets is some");
    assert!(targets.destination_component_id == even_component_1.id());
    assert!(targets.source_component_id == odd_component_1.id());
    // connect second two components

    let odd_2_sv_id = odd_component_2
        .schema_variant(ctx)
        .await
        .expect("found schema variant")
        .id();
    let even_output_socket_id = OutputSocket::find_with_name(ctx, "one", even_sv_id)
        .await
        .expect("found output socket")
        .expect("output socket exists")
        .id();
    let odd_input_socket_id = InputSocket::find_with_name(ctx, "one", odd_2_sv_id)
        .await
        .expect("found input socket")
        .expect("input socket exists")
        .id();
    let apa_id = Component::connect(
        ctx,
        even_component_1.id(),
        even_output_socket_id,
        odd_component_2.id(),
        odd_input_socket_id,
    )
    .await
    .expect("created connection")
    .expect("apa exists");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("got diagram");
    assert_eq!(
        2,                   // expected
        diagram.edges.len()  // actual
    );
    assert_eq!(
        3,                        // expected
        diagram.components.len()  // actual
    );
    let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id)
        .await
        .expect("found apa");
    let targets = apa.targets().expect("targets is some");
    assert!(targets.destination_component_id == odd_component_2.id());
    assert!(targets.source_component_id == even_component_1.id());
    // delete even lego 2 component
    let component = even_component_1.delete(ctx).await.expect("could delete");
    // no resources here so we shouldn't have a component on the graph still
    assert!(component.is_none());
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    //make sure everything is cleaned up
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("got diagram");
    assert_eq!(
        0,                   // expected
        diagram.edges.len()  // actual
    );
    assert_eq!(
        2,                        // expected
        diagram.components.len()  // actual
    );
}
#[test]
async fn connect_and_disconnect_components_explicit_connection(ctx: &mut DalContext) {
    // Get the source schema variant id.
    let docker_image_schema = Schema::get_by_name(ctx, "Docker Image")
        .await
        .expect("no schema found");
    let mut docker_image_schema_variants =
        SchemaVariant::list_for_schema(ctx, docker_image_schema.id())
            .await
            .expect("could not list schema variants for schema");
    let docker_image_schema_variant = docker_image_schema_variants
        .pop()
        .expect("schema variants are empty");
    let docker_image_schema_variant_id = docker_image_schema_variant.id();

    // Get the destination schema variant id.
    let butane_schema = Schema::get_by_name(ctx, "Butane")
        .await
        .expect("no schema found");
    let mut butane_schema_variants = SchemaVariant::list_for_schema(ctx, butane_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let butane_schema_variant = butane_schema_variants
        .pop()
        .expect("schema variants are empty");
    let butane_schema_variant_id = butane_schema_variant.id();

    // Find the sockets we want to use.
    let output_socket =
        OutputSocket::find_with_name(ctx, "Container Image", docker_image_schema_variant_id)
            .await
            .expect("could not perform find output socket")
            .expect("output socket not found");
    let input_socket =
        InputSocket::find_with_name(ctx, "Container Image", butane_schema_variant_id)
            .await
            .expect("could not perform find input socket")
            .expect("input socket not found");

    // Create a component for both the source and the destination
    let oysters_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "oysters in my pocket",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a second component for a second source
    let lunch_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "were saving for lunch",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let royel_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "royel otis",
        butane_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect the components!
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        oysters_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect component 2
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        lunch_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");

    assert!(matches!(view, serde_json::Value::Array(_)));

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    if let serde_json::Value::Array(units_array) = view {
        assert_eq!(2, units_array.len())
    }

    // Assemble the diagram and check the edges.
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble the diagram");
    assert_eq!(2, diagram.edges.len());

    // lunch, oysters - docker
    // royel - butane

    // disconnect oysters from butane
    Component::remove_connection(
        ctx,
        oysters_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("able to remove connection");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Unit {
        #[allow(unused)]
        contents: String,
        #[allow(unused)]
        enabled: bool,
        name: String,
    }

    let units: Vec<Unit> = serde_json::from_value(view).expect("able to deserialize");
    assert_eq!(1, units.len());
    assert_eq!(
        "were-saving-for-lunch.service",
        units
            .first()
            .map(|unit| unit.name.to_owned())
            .expect("has the first unit")
    );

    // Disconnect lunch from butane
    Component::remove_connection(
        ctx,
        lunch_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("able to remove connection");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");
    let units: Vec<Unit> = serde_json::from_value(view).expect("able to deserialize");
    let empty_vec: Vec<Unit> = vec![];
    assert_eq!(empty_vec, units, "units should now be empty");
}

#[test]
async fn connect_to_one_destination_with_multiple_candidates_of_same_schema_variant_on_diagram(
    ctx: &mut DalContext,
) {
    let source = create_component_for_default_schema_name_in_default_view(ctx, "fallout", "source")
        .await
        .expect("could not create component");
    let source_sv_id = Component::schema_variant_id(ctx, source.id())
        .await
        .expect("find variant id for component");

    let destination =
        create_component_for_default_schema_name_in_default_view(ctx, "starfield", "destination")
            .await
            .expect("could not create component");
    let destination_sv_id = Component::schema_variant_id(ctx, destination.id())
        .await
        .expect("find variant id for component");
    create_component_for_default_schema_name_in_default_view(ctx, "starfield", "not destination")
        .await
        .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let output_socket = OutputSocket::find_with_name(ctx, "bethesda", source_sv_id)
        .await
        .expect("could not perform find output socket")
        .expect("output socket not found");

    let input_socket = InputSocket::find_with_name(ctx, "bethesda", destination_sv_id)
        .await
        .expect("could not perform find input socket")
        .expect("input socket not found");

    Component::connect(
        ctx,
        source.id(),
        output_socket.id(),
        destination.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble the diagram");

    assert_eq!(diagram.components.len(), 3);
    assert_eq!(diagram.edges.len(), 1);
    let edge = &diagram.edges[0];
    assert_eq!(edge.from_component_id, source.id());
    assert_eq!(edge.to_component_id, destination.id());
}

#[test]
async fn remove_connection(ctx: &mut DalContext) {
    // Get the source schema variant id.
    let docker_image_schema = Schema::get_by_name(ctx, "Docker Image")
        .await
        .expect("no schema found");
    let mut docker_image_schema_variants =
        SchemaVariant::list_for_schema(ctx, docker_image_schema.id())
            .await
            .expect("could not list schema variants for schema");
    let docker_image_schema_variant = docker_image_schema_variants
        .pop()
        .expect("schema variants are empty");
    let docker_image_schema_variant_id = docker_image_schema_variant.id();

    // Get the destination schema variant id.
    let butane_schema = Schema::get_by_name(ctx, "Butane")
        .await
        .expect("no schema found");
    let mut butane_schema_variants = SchemaVariant::list_for_schema(ctx, butane_schema.id())
        .await
        .expect("could not list schema variants for schema");
    let butane_schema_variant = butane_schema_variants
        .pop()
        .expect("schema variants are empty");
    let butane_schema_variant_id = butane_schema_variant.id();

    // Find the sockets we want to use.
    let output_socket =
        OutputSocket::find_with_name(ctx, "Container Image", docker_image_schema_variant_id)
            .await
            .expect("could not perform find output socket")
            .expect("output socket not found");
    let input_socket =
        InputSocket::find_with_name(ctx, "Container Image", butane_schema_variant_id)
            .await
            .expect("could not perform find input socket")
            .expect("input socket not found");

    // Create a component for both the source and the destination
    let oysters_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "oysters in my pocket",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Create a second component for a second source
    let lunch_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "were saving for lunch",
        docker_image_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let royel_component = create_named_component_for_schema_variant_on_default_view(
        ctx,
        "royel otis",
        butane_schema_variant_id,
    )
    .await
    .expect("could not create component");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect the components!
    let inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        oysters_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components")
    .expect("duplicate connection");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    // Connect component 2
    let _inter_component_attribute_prototype_argument_id = Component::connect(
        ctx,
        lunch_component.id(),
        output_socket.id(),
        royel_component.id(),
        input_socket.id(),
    )
    .await
    .expect("could not connect components");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");

    assert!(matches!(view, serde_json::Value::Array(_)));

    if let serde_json::Value::Array(units_array) = view {
        assert_eq!(2, units_array.len())
    }

    // Assemble the diagram and check the edges.
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble the diagram");
    assert_eq!(2, diagram.edges.len());

    // Disconnect Component 1
    AttributePrototypeArgument::remove(ctx, inter_component_attribute_prototype_argument_id)
        .await
        .expect("Unable to remove inter component attribute prototype argument");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let units_value_id = royel_component
        .attribute_values_for_prop(ctx, &["root", "domain", "systemd", "units"])
        .await
        .expect("able to get values for units")
        .first()
        .copied()
        .expect("has a value");

    let view = AttributeValue::view(ctx, units_value_id)
        .await
        .expect("able to get units view")
        .expect("units has a view");

    assert!(matches!(view, serde_json::Value::Array(_)));

    if let serde_json::Value::Array(units_array) = view {
        assert_eq!(1, units_array.len())
    }

    // Assemble the diagram and check the edges.
    let diagram = Diagram::assemble_for_default_view(ctx)
        .await
        .expect("could not assemble the diagram");
    assert_eq!(1, diagram.edges.len());
}
