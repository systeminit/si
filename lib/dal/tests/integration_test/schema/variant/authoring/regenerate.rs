use dal::func::argument::{FuncArgument, FuncArgumentId};
use dal::func::binding::attribute::AttributeBinding;
use dal::func::binding::{
    AttributeArgumentBinding, AttributeFuncArgumentSource, AttributeFuncDestination, FuncBinding,
};
use dal::func::intrinsics::IntrinsicFunc;
use dal::prop::PropPath;
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{
    ComponentType, DalContext, Func, FuncId, OutputSocket, OutputSocketId, Prop, PropId, Schema,
    SchemaVariant, SchemaVariantId,
};
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::test;

#[test]
async fn regenerate_variant(ctx: &mut DalContext) {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("could not fork head");
    // find the variant we know is default and attached to this func already
    let schema = Schema::find_by_name(ctx, "dummy-secret")
        .await
        .expect("unable to find by name")
        .expect("no schema found");

    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id())
        .await
        .expect("unable to get default schema variant");
    // Cache the total number of funcs before continuing.
    let funcs = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");

    // Get the Auth Func
    let fn_name = "test:setDummySecretString";
    let func_id = Func::find_id_by_name(ctx, fn_name)
        .await
        .expect("found auth func")
        .expect("has a func");

    // ensure the func is attached
    assert!(funcs.into_iter().any(|func| func.id == func_id));

    // unlock schema variant
    let unlocked_schema_variant =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await
            .expect("could not unlock variant");

    // ensure func is attached to new variant

    let funcs_for_unlocked = SchemaVariant::all_funcs(ctx, unlocked_schema_variant.id)
        .await
        .expect("could not list funcs for schema variant");

    // ensure the func is attached
    assert!(funcs_for_unlocked
        .into_iter()
        .any(|func| func.id == func_id));

    // get the existing default variant and ensure the auth func is still attached to it
    let funcs_for_default = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    // ensure the func is attached
    assert!(funcs_for_default.into_iter().any(|func| func.id == func_id));

    // regenerate variant
    VariantAuthoringClient::regenerate_variant(ctx, unlocked_schema_variant.id)
        .await
        .expect("could not regenerate variant");

    // ensure funcs are attached to regenerated AND the existing default
    // ensure func is attached to new variant
    let funcs_for_unlocked = SchemaVariant::all_funcs(ctx, unlocked_schema_variant.id)
        .await
        .expect("could not list funcs for schema variant");
    // ensure the func is attached
    assert!(funcs_for_unlocked
        .into_iter()
        .any(|func| func.id == func_id));

    // get the existing default variant and ensure the auth func is still attached to it
    let funcs_for_default = SchemaVariant::all_funcs(ctx, schema_variant_id)
        .await
        .expect("could not list funcs for schema variant");
    // ensure the func is attached
    assert!(funcs_for_default.into_iter().any(|func| func.id == func_id));
}

#[test]
async fn retain_bindings(ctx: &mut DalContext) {
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
        .await
        .expect("unable to create schema and variant");
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
    .await
    .expect("could not save content");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Once it's all ready, regenerate and commit.
    let schema_variant_id = VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id)
        .await
        .expect("could not regenerate variant");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Gather all arguments needed to create our bindings.
    let alpha_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_source_prop"]),
    )
    .await
    .expect("could not find prop id by path");
    let alpha_destination_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_destination_prop"]),
    )
    .await
    .expect("could not find prop id by path");
    let beta_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "beta_source_prop"]),
    )
    .await
    .expect("could not find prop id by path");
    let beta_destination_output_socket_id = {
        let beta_destination_output_socket =
            OutputSocket::find_with_name(ctx, "beta_destination_output_socket", schema_variant_id)
                .await
                .expect("could not find with name")
                .expect("no output socket found");
        beta_destination_output_socket.id()
    };
    let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
        .await
        .expect("could not find identity func");
    let identity_func_argument_id = {
        let identity_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "identity", identity_func_id)
                .await
                .expect("could not find by name for func")
                .expect("no func argument found");
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
    .await
    .expect("could not create binding");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

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
    .await
    .expect("could not create binding");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // Check that the bindings look as we expect.
    let bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, identity_func_id, schema_variant_id)
            .await
            .expect("could not get bindings");
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
    let schema_variant_id = VariantAuthoringClient::regenerate_variant(ctx, schema_variant_id)
        .await
        .expect("could not regenerate variant");
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit");

    // After regenerating again, we need to re-fetch our prop(s) and socket(s).
    let alpha_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_source_prop"]),
    )
    .await
    .expect("could not find prop id by path");
    let alpha_destination_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "alpha_destination_prop"]),
    )
    .await
    .expect("could not find prop id by path");
    let beta_source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "domain", "beta_source_prop"]),
    )
    .await
    .expect("could not find prop id by path");
    let beta_destination_output_socket_id = {
        let beta_destination_output_socket =
            OutputSocket::find_with_name(ctx, "beta_destination_output_socket", schema_variant_id)
                .await
                .expect("could not find with name")
                .expect("no output socket found");
        beta_destination_output_socket.id()
    };

    // Check that the bindings look as we expect after regenerating again.
    let bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, identity_func_id, schema_variant_id)
            .await
            .expect("could not get bindings");
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
) -> Result<(), Box<dyn std::error::Error>> {
    let eventual_parent =
        AttributeBinding::assemble_eventual_parent(ctx, None, Some(schema_variant_id)).await?;
    let attribute_output_location = AttributeBinding::assemble_attribute_output_location(
        maybe_destination_prop_id.map(Into::into),
        maybe_destination_output_socket_id.map(Into::into),
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
