use dal::func::argument::FuncArgument;
use dal::func::binding::attribute::AttributeBinding;
use dal::func::binding::{
    AttributeArgumentBinding, AttributeFuncArgumentSource, AttributeFuncDestination, EventualParent,
};
use dal::prop::{Prop, PropPath};
use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{AttributeValue, Func, InputSocket};
use dal::{Component, DalContext, Schema};
use dal_test::helpers::{
    create_component_for_default_schema_name_in_default_view,
    create_component_for_schema_variant_on_default_view, update_attribute_value_for_component,
    ChangeSetTestHelpers,
};
use dal_test::{test, Result};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn autoconnect_basic(ctx: &mut DalContext) -> Result<()> {
    let even =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even")
            .await?;
    let odd =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "odd")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // update both sides attribute values
    update_attribute_value_for_component(
        ctx,
        even.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;
    update_attribute_value_for_component(
        ctx,
        odd.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now let's autoconnect!
    Component::autoconnect(ctx, odd.id()).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let incoming = Component::incoming_connections_for_id(ctx, odd.id()).await?;
    assert!(!incoming.is_empty());
    assert!(incoming.len() == 1);

    Ok(())
}

#[test]
async fn autoconnect_multiple_options(ctx: &mut DalContext) -> Result<()> {
    let even =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even")
            .await?;
    let even_2 =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even 2")
            .await?;
    let odd =
        create_component_for_default_schema_name_in_default_view(ctx, "small odd lego", "odd")
            .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // update both sides attribute values for all components
    update_attribute_value_for_component(
        ctx,
        even.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;

    update_attribute_value_for_component(
        ctx,
        even_2.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;
    update_attribute_value_for_component(
        ctx,
        odd.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now let's autoconnect!
    let (created_connections, potential_connections) =
        Component::autoconnect(ctx, odd.id()).await?;

    assert!(created_connections.is_empty());
    assert!(potential_connections.len() == 1);

    // Get first key and first value
    let first_key = potential_connections.keys().next().unwrap();

    // there should be two potential connections
    let (mut potential_connections, _) = potential_connections
        .get(first_key)
        .expect("no potential connections found")
        .clone();
    assert_eq!(potential_connections.len(), 2);

    // create one arbitrarily
    let (component_id, output_socket_id, _value) = potential_connections
        .pop()
        .expect("failed to pop potential connection");
    // Reset default prototype and create connection
    AttributeValue::use_default_prototype(ctx, first_key.attribute_value_id)
        .await
        .expect("failed to reset default prototype");
    Component::connect(
        ctx,
        component_id,
        output_socket_id,
        first_key.component_id,
        first_key.input_socket_id,
    )
    .await
    .expect("failed to create connection");

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now run autconnect again, shouldn't return any potential connections now!
    let (created_connections, potential_connections) =
        Component::autoconnect(ctx, odd.id()).await?;
    assert!(created_connections.is_empty());
    assert!(potential_connections.is_empty());

    let incoming = Component::incoming_connections_for_id(ctx, odd.id()).await?;
    assert!(!incoming.is_empty());
    assert!(incoming.len() == 1);

    Ok(())
}

#[test]
async fn autoconnect_multiple_incoming_connections(ctx: &mut DalContext) -> Result<()> {
    let small_even =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even")
            .await?;
    let small_even_2 =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even 2")
            .await?;
    let medium_even =
        create_component_for_default_schema_name_in_default_view(ctx, "medium even lego", "even")
            .await?;
    let medium_even_2 =
        create_component_for_default_schema_name_in_default_view(ctx, "medium even lego", "even 2")
            .await?;

    let odd =
        create_component_for_default_schema_name_in_default_view(ctx, "medium odd lego", "odd")
            .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // update both sides attribute values for all components

    // small even matches one
    update_attribute_value_for_component(
        ctx,
        small_even.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;

    update_attribute_value_for_component(
        ctx,
        small_even_2.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;
    update_attribute_value_for_component(
        ctx,
        odd.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;

    // medium even matches three
    update_attribute_value_for_component(
        ctx,
        medium_even.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await?;

    update_attribute_value_for_component(
        ctx,
        medium_even_2.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await?;

    update_attribute_value_for_component(
        ctx,
        odd.id(),
        &["root", "domain", "three"],
        serde_json::json!["3"],
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now let's autoconnect!
    let (created_connections, mut potential_connections) =
        Component::autoconnect(ctx, odd.id()).await?;

    // no connections should be created
    assert!(created_connections.is_empty());

    // but we should see two input sockets with potential connections
    assert!(potential_connections.len() == 2);

    // create one arbitrarily for each input socket
    for (key, (potential_connections, _)) in potential_connections.iter_mut() {
        let (component_id, output_socket_id, _value) = potential_connections
            .pop()
            .expect("failed to pop potential connection");

        // Reset default prototype and create connection
        AttributeValue::use_default_prototype(ctx, key.attribute_value_id)
            .await
            .expect("failed to reset default prototype");
        Component::connect(
            ctx,
            component_id,
            output_socket_id,
            key.component_id,
            key.input_socket_id,
        )
        .await
        .expect("failed to create connection");
    }

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now run autconnect again, shouldn't return any potential connections now!
    let (created_connections, potential_connections) =
        Component::autoconnect(ctx, odd.id()).await?;
    assert!(created_connections.is_empty());
    assert!(potential_connections.is_empty());

    let incoming = Component::incoming_connections_for_id(ctx, odd.id()).await?;
    assert!(!incoming.is_empty());
    assert!(incoming.len() == 2);

    Ok(())
}

#[test]
async fn autoconnect_multi_arity(ctx: &mut DalContext) -> Result<()> {
    // create a new schema with a multi-arity input socket  // Let's create a new asset
    let variant_zero = VariantAuthoringClient::create_schema_and_variant(
        ctx,
        "paulsTestAsset",
        None,
        None,
        "Integration Tests",
        "#00b0b0",
    )
    .await?;

    let my_asset_schema = variant_zero.schema(ctx).await?;

    let default_schema_variant = Schema::default_variant_id(ctx, my_asset_schema.id()).await?;
    assert_eq!(default_schema_variant, variant_zero.id());

    // Now let's update the variant
    let first_code_update = "function main() {\n
        
         const arrayProp = new PropBuilder().setName(\"arrayProp\").setKind(\"array\").setEntry(\n
            new PropBuilder().setName(\"arrayElem\").setKind(\"string\").build()\n
        ).build();\n
        const inputSocket = new SocketDefinitionBuilder().setName(\"one\").setArity(\"many\").build();\n
         return new AssetBuilder().addProp(arrayProp).addInputSocket(inputSocket).build()\n}"
        .to_string();

    VariantAuthoringClient::save_variant_content(
        ctx,
        variant_zero.id(),
        my_asset_schema.name.clone(),
        variant_zero.display_name(),
        variant_zero.category(),
        variant_zero.description(),
        variant_zero.link(),
        variant_zero.get_color(ctx).await?,
        variant_zero.component_type(),
        Some(first_code_update),
    )
    .await?;

    let updated_variant_id =
        VariantAuthoringClient::regenerate_variant(ctx, variant_zero.id()).await?;

    // We should still see that the schema variant we updated is the same as we have no components on the graph
    assert_eq!(variant_zero.id(), updated_variant_id);

    // now ensure the array prop takes it's value from the input socket
    let array_prop = Prop::find_prop_id_by_path(
        ctx,
        updated_variant_id,
        &PropPath::new(["root", "domain", "arrayProp"]),
    )
    .await?;

    let input_socket = InputSocket::find_with_name_or_error(ctx, "one", updated_variant_id).await?;

    let output_location = AttributeFuncDestination::Prop(array_prop);
    let input_location = AttributeFuncArgumentSource::InputSocket(input_socket.id());
    let normalize_to_array_func = Func::find_id_by_name(ctx, "si:normalizeToArray")
        .await?
        .expect("normalizeToArray func not found");

    let mut normalize_to_array_arg =
        FuncArgument::list_for_func(ctx, normalize_to_array_func).await?;

    // should only be one!
    assert_eq!(normalize_to_array_arg.len(), 1);

    let normalize_to_array_arg = normalize_to_array_arg
        .pop()
        .expect("unable to pop func argument");

    let arguments: Vec<AttributeArgumentBinding> = vec![
        (AttributeArgumentBinding {
            func_argument_id: normalize_to_array_arg.id,
            attribute_func_input_location: input_location,
            attribute_prototype_argument_id: None,
        }),
    ];

    AttributeBinding::upsert_attribute_binding(
        ctx,
        normalize_to_array_func,
        Some(EventualParent::SchemaVariant(updated_variant_id)),
        output_location,
        arguments,
    )
    .await?;

    // now that's all set up, let's create a component that uses this asset
    let component =
        create_component_for_schema_variant_on_default_view(ctx, updated_variant_id).await?;

    // now let's create a bunch of other components that can feed this one with individual values
    let small_even =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even")
            .await?;
    let small_even_2 =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even 2")
            .await?;

    // set each one to a different value
    update_attribute_value_for_component(
        ctx,
        small_even.id(),
        &["root", "domain", "one"],
        serde_json::json!["1"],
    )
    .await?;

    update_attribute_value_for_component(
        ctx,
        small_even_2.id(),
        &["root", "domain", "one"],
        serde_json::json!["2"],
    )
    .await?;

    // set the component's value to an array of the small even values
    update_attribute_value_for_component(
        ctx,
        component.id(),
        &["root", "domain", "arrayProp"],
        serde_json::json!(["1", "2"]),
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now let's autoconnect!
    let (created_connections, potential_connections) =
        Component::autoconnect(ctx, component.id()).await?;
    assert!(created_connections.len() == 2);
    assert!(potential_connections.is_empty());
    let incoming = Component::incoming_connections_for_id(ctx, component.id()).await?;
    assert!(!incoming.is_empty());
    assert!(incoming.len() == 2);

    // now let's create another component, and a duplicate potential connection
    let small_even_3 =
        create_component_for_default_schema_name_in_default_view(ctx, "small even lego", "even 3")
            .await?;

    // set it to a duplicate value
    update_attribute_value_for_component(
        ctx,
        small_even_3.id(),
        &["root", "domain", "one"],
        serde_json::json!["2"],
    )
    .await?;

    let new_component =
        create_component_for_schema_variant_on_default_view(ctx, updated_variant_id).await?;

    // set it to a duplicate value
    update_attribute_value_for_component(
        ctx,
        new_component.id(),
        &["root", "domain", "arrayProp"],
        serde_json::json!(["1", "2"]),
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // now let's autoconnect!
    let (created_connections, mut available_connections) =
        Component::autoconnect(ctx, new_component.id()).await?;
    // no created connections because we don't know which to choose
    assert!(created_connections.is_empty());
    assert!(available_connections.len() == 1);

    // make sure we see the expected components here
    for (_key, (potential_connections, _)) in available_connections.iter_mut() {
        assert!(potential_connections.len() == 3);
        let components_to_connect_to = potential_connections
            .iter()
            .map(|(component_id, _, _)| component_id)
            .collect::<Vec<_>>();
        assert!(components_to_connect_to.contains(&&small_even.id()));
        assert!(components_to_connect_to.contains(&&small_even_2.id()));
        assert!(components_to_connect_to.contains(&&small_even_3.id()));
    }

    Ok(())
}
