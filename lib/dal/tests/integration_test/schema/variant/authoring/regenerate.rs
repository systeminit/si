use dal::{
    AttributeValue,
    Component,
    ComponentType,
    DalContext,
    Func,
    FuncId,
    InputSocket,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    SchemaVariant,
    SchemaVariantId,
    SocketArity,
    func::{
        argument::{
            FuncArgument,
            FuncArgumentId,
        },
        binding::{
            AttributeArgumentBinding,
            AttributeFuncArgumentSource,
            AttributeFuncDestination,
            FuncBinding,
            attribute::AttributeBinding,
        },
        intrinsics::IntrinsicFunc,
    },
    prop::PropPath,
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    color_eyre::Result,
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};

#[test]
async fn regenerate_variant(ctx: &mut DalContext) -> Result<()> {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    // find the variant we know is default and attached to this func already
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "dummy-secret").await?;

    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id).await?;

    // Get the Auth Func
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await?
        .expect("has a func");

    // ensure the func is attached
    assert!(funcs.into_iter().any(|func| func.id == func_id));

    // unlock schema variant
    let unlocked_schema_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id).await?;

    // ensure func is attached to new variant

    let funcs_for_unlocked = SchemaVariant::all_funcs(ctx, unlocked_schema_variant.id).await?;

    // ensure the func is attached
    assert!(
        funcs_for_unlocked
            .into_iter()
            .any(|func| func.id == func_id)
    );

    // get the existing default variant and ensure the auth func is still attached to it
    let funcs_for_default = SchemaVariant::all_funcs(ctx, schema_variant_id).await?;
    // ensure the func is attached
    assert!(funcs_for_default.into_iter().any(|func| func.id == func_id));

    // regenerate variant
    VariantAuthoringClient::regenerate_variant(ctx, unlocked_schema_variant.id).await?;

    // ensure funcs are attached to regenerated AND the existing default
    // ensure func is attached to new variant
    let funcs_for_unlocked = SchemaVariant::all_funcs(ctx, unlocked_schema_variant.id).await?;
    // ensure the func is attached
    assert!(
        funcs_for_unlocked
            .into_iter()
            .any(|func| func.id == func_id)
    );

    // get the existing default variant and ensure the auth func is still attached to it
    let funcs_for_default = SchemaVariant::all_funcs(ctx, schema_variant_id).await?;
    // ensure the func is attached
    assert!(funcs_for_default.into_iter().any(|func| func.id == func_id));
    Ok(())
}

#[test]
async fn update_socket_data_on_regenerate(ctx: &mut DalContext) -> Result<()> {
    let name = "Bandit";
    let description = None;
    let link = None;
    let category = "Blue Heelers";
    let color = "#00A19B";

    // Create an asset with a corresponding asset func. After that, commit.
    let schema_variant_id = {
        let schema_variant = VariantAuthoringClient::create_schema_and_variant(
            ctx,
            name,
            description.clone(),
            link.clone(),
            category,
            color,
        )
        .await?;
        schema_variant.id()
    };
    let asset_func = "function main() {
        const asset = new AssetBuilder();
        const beta_destination_input_socket = new SocketDefinitionBuilder()
            .setName(\"input_socket\")
            .setArity(\"one\")
            .setConnectionAnnotation(\"two\")
            .build();
        asset.addInputSocket(beta_destination_input_socket);
        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName(\"output_socket\")
            .setArity(\"one\")
            .setConnectionAnnotation(\"two\")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }";
    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant_id,
        name,
        name,
        category,
        description.clone(),
        link.clone(),
        color,
        ComponentType::Component,
        Some(asset_func),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Once it's all ready, regenerate and commit.
    let schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // create a component
    let component =
        create_component_for_default_schema_name_in_default_view(ctx, "Bandit", "Bluey").await?;
    let component_id = component.id();

    // create 2 more to connect it to things
    let _input_comp = create_component_for_default_schema_name_in_default_view(
        ctx,
        "small odd lego",
        "small odd",
    )
    .await?;

    let _output_comp =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let component = Component::get_by_id(ctx, component_id).await?;
    // check the input socket data
    let mut input_socket_avs = component.input_socket_attribute_values(ctx).await?;
    assert!(input_socket_avs.len() == 1);
    let input_socket_av = input_socket_avs.pop().expect("has one just checked");
    let input_socket_id = InputSocket::find_for_attribute_value_id(ctx, input_socket_av)
        .await?
        .expect("has one for the av");
    let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
    assert_eq!(input_socket.arity(), SocketArity::One);
    assert_eq!(
        format!("{:?}", input_socket.connection_annotations()),
        "[ConnectionAnnotation { tokens: [\"two\"] }, ConnectionAnnotation { tokens: [\"input_socket\"] }]"
    );

    // check output socket data
    let mut output_socket_avs = component.output_socket_attribute_values(ctx).await?;
    assert!(output_socket_avs.len() == 1);
    let output_socket_av = output_socket_avs.pop().expect("has one just checked");
    let output_socket_id = OutputSocket::find_for_attribute_value_id(ctx, output_socket_av)
        .await?
        .expect("has one for the av");
    let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
    assert_eq!(output_socket.arity(), SocketArity::One);
    assert_eq!(
        format!("{:?}", output_socket.connection_annotations()),
        "[ConnectionAnnotation { tokens: [\"two\"] }, ConnectionAnnotation { tokens: [\"output_socket\"] }]"
    );

    // Modify the sockets, changing their arity and adding a connection annotation
    let asset_func = "function main() {
        const asset = new AssetBuilder();
       const beta_destination_input_socket = new SocketDefinitionBuilder()
            .setName(\"input_socket\")
            .setArity(\"many\")
            .setConnectionAnnotation(\"two\")
            .setConnectionAnnotation(\"dog\")
            .build();
        asset.addInputSocket(beta_destination_input_socket);
        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName(\"output_socket\")
            .setArity(\"many\")
            .setConnectionAnnotation(\"one\")
            .setConnectionAnnotation(\"dog\")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }";
    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant_id,
        name,
        name,
        category,
        description,
        link,
        color,
        ComponentType::Component,
        Some(asset_func),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Once it's all ready, regenerate and commit.
    // Note that regenerate should auto-update the component
    // so this also ensures the component upgrade is successful
    VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // check the input socket data
    let mut input_socket_avs = component.input_socket_attribute_values(ctx).await?;
    assert!(input_socket_avs.len() == 1);
    let input_socket_av = input_socket_avs.pop().expect("has one just checked");
    let input_socket_id = InputSocket::find_for_attribute_value_id(ctx, input_socket_av)
        .await?
        .expect("couldn't find input socket");
    let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
    assert_eq!(input_socket.arity(), SocketArity::Many);
    assert_eq!(
        format!("{:?}", input_socket.connection_annotations()),
        "[ConnectionAnnotation { tokens: [\"two\"] }, ConnectionAnnotation { tokens: [\"dog\"] }, ConnectionAnnotation { tokens: [\"input_socket\"] }]"
    );

    // check output socket data
    let mut output_socket_avs = component.output_socket_attribute_values(ctx).await?;
    assert!(output_socket_avs.len() == 1);
    let output_socket_av = output_socket_avs.pop().expect("has one just checked");
    let output_socket_id = OutputSocket::find_for_attribute_value_id(ctx, output_socket_av)
        .await?
        .expect("couldn't find input socket");
    let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
    assert_eq!(output_socket.arity(), SocketArity::Many);
    assert_eq!(
        format!("{:?}", output_socket.connection_annotations()),
        "[ConnectionAnnotation { tokens: [\"one\"] }, ConnectionAnnotation { tokens: [\"dog\"] }, ConnectionAnnotation { tokens: [\"output_socket\"] }]"
    );
    Ok(())
}

#[test]
async fn retain_bindings(ctx: &mut DalContext) -> Result<()> {
    let name = "Toto Wolff";
    let description = None;
    let link = None;
    let category = "Mercedes AMG Petronas";
    let color = "#00A19B";

    // Create an asset with a corresponding asset func. After that, commit.
    let schema_variant_id = {
        let schema_variant = VariantAuthoringClient::create_schema_and_variant(
            ctx,
            name,
            description.clone(),
            link.clone(),
            category,
            color,
        )
        .await?;
        schema_variant.id()
    };
    let asset_func = "function main() {
        const asset = new AssetBuilder();

        const alpha_source_prop = new PropBuilder()
            .setName(\"alpha_source_prop\")
            .setKind(\"string\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(alpha_source_prop);

        const alpha_destination_prop = new PropBuilder()
            .setName(\"alpha_destination_prop\")
            .setKind(\"string\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(alpha_destination_prop);

        const beta_source_prop = new PropBuilder()
            .setName(\"beta_source_prop\")
            .setKind(\"string\")
            .setWidget(new PropWidgetDefinitionBuilder().setKind(\"text\").build())
            .build();
        asset.addProp(beta_source_prop);

        const beta_destination_output_socket = new SocketDefinitionBuilder()
            .setName(\"beta_destination_output_socket\")
            .setArity(\"one\")
            .build();
        asset.addOutputSocket(beta_destination_output_socket);

        return asset.build();
    }";
    VariantAuthoringClient::save_variant_content(
        ctx,
        schema_variant_id,
        name,
        name,
        category,
        description,
        link,
        color,
        ComponentType::Component,
        Some(asset_func),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Once it's all ready, regenerate and commit.
    let schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Gather all arguments needed to create our bindings.
    let alpha_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_source_prop"]),
    )
    .await?;
    let alpha_destination_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_destination_prop"]),
    )
    .await?;
    let beta_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "beta_source_prop"]),
    )
    .await?;
    let beta_destination_output_socket_id = {
        let beta_destination_output_socket =
            OutputSocket::find_with_name(ctx, "beta_destination_output_socket", schema_variant_id)
                .await?
                .expect("no output socket found");
        beta_destination_output_socket.id()
    };
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?;
    let identity_func_argument_id = {
        let identity_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
                .await?
                .expect("could not find by name for func");
        identity_func_argument.id
    };

    // Create the first binding and commit.
    create_binding_simple(
        ctx,
        schema_variant_id,
        alpha_source_prop_id,
        Some(alpha_destination_prop_id),
        None,
        identity_func_id,
        identity_func_argument_id,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Create the second binding and commit.
    create_binding_simple(
        ctx,
        schema_variant_id,
        beta_source_prop_id,
        None,
        Some(beta_destination_output_socket_id),
        identity_func_id,
        identity_func_argument_id,
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that the bindings look as we expect.
    let bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, identity_func_id, schema_variant_id)
            .await?;
    assert_eq!(
        2,              // expected
        bindings.len()  // actual
    );
    for binding in bindings {
        match binding {
            FuncBinding::Attribute(mut binding) => match binding.output_location {
                AttributeFuncDestination::Prop(prop_id) => {
                    assert_eq!(
                        alpha_destination_prop_id, // expected
                        prop_id                    // actual
                    );
                    let argument_binding = binding
                        .argument_bindings
                        .pop()
                        .expect("unexpected empty argument bindings");
                    assert!(binding.argument_bindings.is_empty());
                    assert_eq!(
                        identity_func_argument_id,         // expected
                        argument_binding.func_argument_id  // actual
                    );
                    assert_eq!(
                        AttributeFuncArgumentSource::Prop(alpha_source_prop_id), // expected
                        argument_binding.attribute_func_input_location           // actual
                    );
                }
                AttributeFuncDestination::OutputSocket(output_socket_id) => {
                    assert_eq!(
                        beta_destination_output_socket_id, // expected
                        output_socket_id                   // actual
                    );
                    let argument_binding = binding
                        .argument_bindings
                        .pop()
                        .expect("unexpected empty argument bindings");
                    assert!(binding.argument_bindings.is_empty());
                    assert_eq!(
                        identity_func_argument_id,         // expected
                        argument_binding.func_argument_id  // actual
                    );
                    assert_eq!(
                        AttributeFuncArgumentSource::Prop(beta_source_prop_id), // expected
                        argument_binding.attribute_func_input_location          // actual
                    );
                }
                output_location => panic!("unexpected output location: {output_location:?}"),
            },
            inner_binding => panic!("unexpected binding kind: {inner_binding:?}"),
        }
    }

    // Regenerate the variant again to ensure that we have retained our bindings.
    let schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // After regenerating again, we need to re-fetch our prop(s) and socket(s).
    let alpha_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_source_prop"]),
    )
    .await?;
    let alpha_destination_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_destination_prop"]),
    )
    .await?;
    let beta_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "beta_source_prop"]),
    )
    .await?;
    let beta_destination_output_socket_id = {
        let beta_destination_output_socket =
            OutputSocket::find_with_name(ctx, "beta_destination_output_socket", schema_variant_id)
                .await?
                .expect("no output socket found");
        beta_destination_output_socket.id()
    };

    // Check that the bindings look as we expect after regenerating again.
    let bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, identity_func_id, schema_variant_id)
            .await?;
    assert_eq!(
        2,              // expected
        bindings.len()  // actual
    );
    for binding in bindings {
        match binding {
            FuncBinding::Attribute(mut binding) => match binding.output_location {
                AttributeFuncDestination::Prop(prop_id) => {
                    assert_eq!(
                        alpha_destination_prop_id, // expected
                        prop_id                    // actual
                    );
                    let argument_binding = binding
                        .argument_bindings
                        .pop()
                        .expect("unexpected empty argument bindings");
                    assert!(binding.argument_bindings.is_empty());
                    assert_eq!(
                        identity_func_argument_id,         // expected
                        argument_binding.func_argument_id  // actual
                    );
                    assert_eq!(
                        AttributeFuncArgumentSource::Prop(alpha_source_prop_id), // expected
                        argument_binding.attribute_func_input_location           // actual
                    );
                }
                AttributeFuncDestination::OutputSocket(output_socket_id) => {
                    assert_eq!(
                        beta_destination_output_socket_id, // expected
                        output_socket_id                   // actual
                    );
                    let argument_binding = binding
                        .argument_bindings
                        .pop()
                        .expect("unexpected empty argument bindings");
                    assert!(binding.argument_bindings.is_empty());
                    assert_eq!(
                        identity_func_argument_id,         // expected
                        argument_binding.func_argument_id  // actual
                    );
                    assert_eq!(
                        AttributeFuncArgumentSource::Prop(beta_source_prop_id), // expected
                        argument_binding.attribute_func_input_location          // actual
                    );
                }
                output_location => panic!("unexpected output location: {output_location:?}"),
            },
            inner_binding => panic!("unexpected binding kind: {inner_binding:?}"),
        }
    }
    Ok(())
}

#[test]
async fn dynamic_functions_on_new_attribute_values_are_rerun(ctx: &mut DalContext) -> Result<()> {
    let destination_schema_variant_name = "SchemaVariant Name";
    let category = "Schema Variant Category";
    let description = None;
    let link = None;
    let color = "#0586F1";
    let destination_schema_variant_id = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        destination_schema_variant_name,
        description.clone(),
        link.clone(),
        category,
        color,
    )
    .await?
    .id();
    let destination_asset_func = r##"function main() {
        const asset = new AssetBuilder();

        const inputNumbers = new PropBuilder()
            .setName("inputNumbers")
            .setKind("array")
            .setHidden(false)
            .setWidget(new PropWidgetDefinitionBuilder()
                .setKind("array")
                .build())
            .setEntry(new PropBuilder()
                .setName("number")
                .setKind("string")
                .setHidden(false)
                .setWidget(new PropWidgetDefinitionBuilder()
                   .setKind("header")
                   .build())
                .build()
            )
            .build();
        const outputNumber = new PropBuilder()
            .setName("outputNumber")
            .setKind("string")
            .build();

        asset.addProp(inputNumbers)
            .addProp(outputNumber)
            .build();
        return asset.build();
    }"##;
    VariantAuthoringClient::save_variant_content(
        ctx,
        destination_schema_variant_id,
        destination_schema_variant_name,
        destination_schema_variant_name,
        category,
        description.clone(),
        link.clone(),
        color,
        ComponentType::Component,
        Some(destination_asset_func),
    )
    .await?;
    let source_schema_variant_name = "Source SchemaVariant";
    let source_category = "Source Schema Category";
    let source_schema_variant_id = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        source_schema_variant_name,
        description.clone(),
        link.clone(),
        source_category,
        color,
    )
    .await?
    .id();
    let source_asset_func = r##"function main() {
        const asset = new AssetBuilder();
        return asset.build();
    }"##;
    VariantAuthoringClient::save_variant_content(
        ctx,
        source_schema_variant_id,
        source_schema_variant_name,
        source_schema_variant_name,
        source_category,
        description,
        link,
        color,
        ComponentType::Component,
        Some(source_asset_func),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    let destination_schema_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, destination_schema_variant_id).await?;
    VariantAuthoringClient::regenerate_variant(ctx, source_schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let source_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        source_schema_variant_name,
        "Source Component",
    )
    .await?;
    let destination_component = create_component_for_default_schema_name_in_default_view(
        ctx,
        destination_schema_variant_name,
        "Destination Component",
    )
    .await?;
    let array_av_id = Component::attribute_value_for_prop(
        ctx,
        destination_component.id(),
        &["root", "domain", "inputNumbers"],
    )
    .await?;
    AttributeValue::insert(ctx, array_av_id, Some(serde_json::json!("")), None).await?;
    dal_test::helpers::attribute::value::subscribe(
        ctx,
        ("Destination Component", "/domain/inputNumbers/0"),
        ("Source Component", "/si/name"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // Make sure we set up the subscription correctly.
    assert_eq!(
        Some(serde_json::json!(["Source Component"])),
        AttributeValue::view(ctx, array_av_id).await?,
    );

    // Regenerate the variant to see if we still have a value for the array element.
    VariantAuthoringClient::regenerate_variant(ctx, destination_schema_variant_id).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let array_av_id = Component::attribute_value_for_prop(
        ctx,
        destination_component.id(),
        &["root", "domain", "inputNumbers"],
    )
    .await?;
    assert_eq!(
        Some(serde_json::json!(["Source Component"])),
        AttributeValue::view(ctx, array_av_id).await?,
    );

    let source_name_av_id =
        Component::attribute_value_for_prop(ctx, source_component.id(), &["root", "si", "name"])
            .await?;
    AttributeValue::update(
        ctx,
        source_name_av_id,
        Some(serde_json::json!("New Source Name")),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        Some(serde_json::json!(["New Source Name"])),
        AttributeValue::view(ctx, array_av_id).await?,
    );

    Ok(())
}

// Mimics the behavior in "v2/func/binding/create_binding" for output sockets.
async fn create_binding_simple(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
    source_prop_id: PropId,
    maybe_destination_prop_id: Option<PropId>,
    maybe_destination_output_socket_id: Option<OutputSocketId>,
    identity_func_id: FuncId,
    identity_func_argument_id: FuncArgumentId,
) -> Result<()> {
    let eventual_parent =
        AttributeBinding::assemble_eventual_parent(ctx, None, Some(schema_variant_id)).await?;
    let attribute_output_location = AttributeBinding::assemble_attribute_output_location(
        maybe_destination_prop_id,
        maybe_destination_output_socket_id,
    )?;

    AttributeBinding::upsert_attribute_binding(
        ctx,
        identity_func_id,
        eventual_parent,
        attribute_output_location,
        vec![AttributeArgumentBinding {
            attribute_prototype_argument_id: None,
            func_argument_id: identity_func_argument_id,
            attribute_func_input_location: AttributeBinding::assemble_attribute_input_location(
                Some(source_prop_id),
                None,
                None,
            )?,
        }],
    )
    .await?;

    Ok(())
}
