use dal::func::argument::{FuncArgument, FuncArgumentKind};
use dal::job::definition::DependentValuesUpdate;
use dal::{
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue, Component,
    ComponentView, DalContext, ExternalProvider, Func, FuncBackendKind, FuncBackendResponseType,
    FuncBinding, InternalProvider, Prop, PropKind, SocketArity, StandardModel,
};
use dal_test::helpers::setup_identity_func;
use dal_test::{
    test,
    test_harness::{create_schema, create_schema_variant_with_root},
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn nested_object_prop_with_complex_func(ctx: &DalContext) {
    // Create and setup the schema and schema variant.
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    let ragnarok_prop = Prop::new(
        ctx,
        "ragnarok",
        PropKind::Object,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let kratos_prop = Prop::new(
        ctx,
        "kratos",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*ragnarok_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let atreus_prop = Prop::new(
        ctx,
        "atreus",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*ragnarok_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");

    // Setup the external provider and finalize.
    let (identity_func_id, identity_func_binding_id, identity_func_binding_return_value_id, _) =
        setup_identity_func(ctx).await;
    let (external_provider, _output_socket) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "kratos",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create external provider");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Collect the internal providers.
    let ragnarok_provider = InternalProvider::find_for_prop(ctx, *ragnarok_prop.id())
        .await
        .expect("could not perform find for prop")
        .expect("no internal provider for prop");
    let kratos_provider = InternalProvider::find_for_prop(ctx, *kratos_prop.id())
        .await
        .expect("could not perform find for prop")
        .expect("no internal provider for prop");
    let atreus_provider = InternalProvider::find_for_prop(ctx, *atreus_prop.id())
        .await
        .expect("could not perform find for prop")
        .expect("no internal provider for prop");

    // Add the external provider.
    let external_provider_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext::default_with_external_provider(*external_provider.id()),
    )
    .await
    .expect("could not perform find for context")
    .expect("attribute value not found");
    let mut external_provider_attribute_prototype = external_provider_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not perform get attribute prototype for attribute value")
        .expect("could not find attribute prototype for attribute value");

    // Create and set the func to take off a string field.
    let mut transformation_func = Func::new(
        ctx,
        "test:getKratos",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::String,
    )
    .await
    .expect("could not create func");
    let code = "function getKratos(input) {
        return input.ragnarok.kratos.toUpperCase() ?? '';
    }";
    transformation_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    transformation_func
        .set_handler(ctx, Some("getKratos"))
        .await
        .expect("set handler");
    external_provider_attribute_prototype
        .set_func_id(ctx, *transformation_func.id())
        .await
        .expect("set function on attribute prototype for external provider");
    let transformation_func_argument = FuncArgument::new(
        ctx,
        "ragnarok",
        FuncArgumentKind::Object,
        None,
        *transformation_func.id(),
    )
    .await
    .expect("could not create func argument");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *external_provider_attribute_prototype.id(),
        *transformation_func_argument.id(),
        *ragnarok_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Create the func for the object prop.
    let mut func = Func::new(
        ctx,
        "test:complexObject",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Object,
    )
    .await
    .expect("could not create func");
    let code = "function complex(_args) {
        return {
            kratos: \"poop\",
            atreus: \"canoe\"
        };
    }";
    func.set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    func.set_handler(ctx, Some("complex"))
        .await
        .expect("set handler");

    // Assign the func for the object prop.
    let ragnarok_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext::default_with_prop(*ragnarok_prop.id()),
    )
    .await
    .expect("could not perform find for context")
    .expect("attribute value not found");
    let mut ragnarok_attribute_prototype = ragnarok_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not perform get attribute prototype for attribute value")
        .expect("could not find attribute prototype for attribute value");
    ragnarok_attribute_prototype
        .set_func_id(ctx, *func.id())
        .await
        .expect("could not set func id");

    // Execute the function and update the values that depend on it.
    let mut attribute_value_for_prototype = ragnarok_attribute_prototype
        .attribute_values(ctx)
        .await
        .expect("could not perform get attribute values for prototype")
        .pop()
        .expect("attribute values empty");
    attribute_value_for_prototype
        .update_from_prototype_function(ctx)
        .await
        .expect("could not update from prototype function");

    ctx.enqueue_job(DependentValuesUpdate::new(
        ctx.access_builder(),
        *ctx.visibility(),
        vec![*attribute_value_for_prototype.id()],
    ))
    .await
    .expect("failed to enqueue job");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Now that everything is set up, create the component.
    let (component, _) = Component::new(ctx, "god-of-war", *schema_variant.id())
        .await
        .expect("unable to create component");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Confirm the component view renders what we expect
    let component_view = ComponentView::new(ctx, *component.id())
        .await
        .expect("cannot get component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "god-of-war",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "ragnarok": {
                        "atreus": "canoe",
                        "kratos": "poop"
                    }
                }
            }
        ],
        component_view.properties,
    );

    assert_eq!(
        Some(serde_json::json!["POOP"]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_external_provider(*external_provider.id()),
        )
        .await,
        "ensure external provider gets value of kratos internal provider in upper case",
    );

    // Prop structure gets the expected values...
    assert_eq!(
        Some(serde_json::json![{}]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_prop(*ragnarok_prop.id()),
        )
        .await,
        "ensure ragnarok prop gets expected value of {{}}",
    );

    assert_eq!(
        Some(serde_json::json!["poop"]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_prop(*kratos_prop.id()),
        )
        .await,
        "ensure kratos prop gets expected value",
    );

    assert_eq!(
        Some(serde_json::json!["canoe"]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_prop(*atreus_prop.id()),
        )
        .await,
        "ensure atreus prop gets expected value",
    );

    assert_eq!(
        Some(serde_json::json![{
            "kratos": "poop",
            "atreus": "canoe",
        }]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_internal_provider(*ragnarok_provider.id()),
        )
        .await
    );

    assert_eq!(
        Some(serde_json::json!["poop"]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_internal_provider(*kratos_provider.id()),
        )
        .await,
        "ensure internal provider for kratos prop gets expected value"
    );

    assert_eq!(
        Some(serde_json::json!["canoe"]),
        dump_value(
            ctx,
            AttributeReadContext::default_with_internal_provider(*atreus_provider.id()),
        )
        .await,
        "ensure internal provider for atreus prop gets expected value"
    );
}

#[test]
async fn map_with_object_entries_and_complex_funcs(ctx: &DalContext) {
    let mut schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");
    let schema_variant_id = *schema_variant.id();

    // Create direct children of domain, including a map of objects.
    let concat_prop = Prop::new(
        ctx,
        "concat",
        PropKind::String,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let map_prop = Prop::new(
        ctx,
        "map",
        PropKind::Map,
        None,
        schema_variant_id,
        Some(root_prop.domain_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let map_prop_id = *map_prop.id();

    // Setup the map and finalize.
    let map_item_prop = Prop::new(
        ctx,
        "item",
        PropKind::Object,
        None,
        schema_variant_id,
        Some(map_prop_id),
        None,
    )
    .await
    .expect("could not create prop");
    let _poop_prop = Prop::new(
        ctx,
        "poop",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*map_item_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    let _canoe_prop = Prop::new(
        ctx,
        "canoe",
        PropKind::String,
        None,
        schema_variant_id,
        Some(*map_item_prop.id()),
        None,
    )
    .await
    .expect("could not create prop");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Create a func for the first object.
    let mut prefix_func = Func::new(
        ctx,
        "test:complexObjectPrefix",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Object,
    )
    .await
    .expect("could not create func");
    let prefix_func_id = *prefix_func.id();
    let code = "function complex(input) {
        return {
            poop: \"update \" + input.concat,
            canoe: \"change \" + input.concat,
        };
    }";
    prefix_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    prefix_func
        .set_handler(ctx, Some("complex"))
        .await
        .expect("set handler");
    let prefix_func_argument = FuncArgument::new(
        ctx,
        "concat",
        FuncArgumentKind::String,
        None,
        prefix_func_id,
    )
    .await
    .expect("could not create func argument");

    // Create a func for the second object.
    let mut suffix_func = Func::new(
        ctx,
        "test:complexObjectSuffix",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Object,
    )
    .await
    .expect("could not create func");
    let suffix_func_id = *suffix_func.id();
    let code = "function complex(input) {
        return {
            poop: input.concat + \" update\",
            canoe: input.concat + \" change\",
        };
    }";
    suffix_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    suffix_func
        .set_handler(ctx, Some("complex"))
        .await
        .expect("set handler");
    let suffix_func_argument = FuncArgument::new(
        ctx,
        "concat",
        FuncArgumentKind::String,
        None,
        suffix_func_id,
    )
    .await
    .expect("could not create func argument");

    // Create a func for a output socket and external provider so that we can confirm that
    // connections using our value work. This will look through the code tree to find an item with
    // key "second-canoe" and then a field called "canoe".
    let mut canoe_from_second_func = Func::new(
        ctx,
        "test:canoeFromSecond",
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::String,
    )
    .await
    .expect("could not create func");
    let canoe_from_second_func_id = *canoe_from_second_func.id();
    let code = "function canoeFromSecond(input) {
        return input.map?.second?.canoe ?? \"\";
    }";
    canoe_from_second_func
        .set_code_plaintext(ctx, Some(code))
        .await
        .expect("set code");
    canoe_from_second_func
        .set_handler(ctx, Some("canoeFromSecond"))
        .await
        .expect("set handler");
    let canoe_from_second_func_argument = FuncArgument::new(
        ctx,
        "map",
        FuncArgumentKind::Map,
        None,
        canoe_from_second_func_id,
    )
    .await
    .expect("could not create func argument");
    let (canoe_from_second_func_binding, canoe_from_second_func_binding_return_value) =
        FuncBinding::create_and_execute(
            ctx,
            serde_json::json!({}),
            canoe_from_second_func_id,
            vec![],
        )
        .await
        .expect("could not perform find or create and execute");
    let (external_provider, _) = ExternalProvider::new_with_socket(
        ctx,
        *schema.id(),
        *schema_variant.id(),
        "second-canoe",
        None,
        canoe_from_second_func_id,
        *canoe_from_second_func_binding.id(),
        *canoe_from_second_func_binding_return_value.id(),
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create external provider");
    let external_provider_attribute_prototype_id = *external_provider
        .attribute_prototype_id()
        .expect("no attribute prototype id for external provider");

    // NOTE(nick): in the future, we hope to potentially just source the specific entry's value.
    // For now, since implicit internal providers are created for everything except for the
    // descendants of maps and arrays, we need to take in the entire map. Not a huge deal for now.
    let map_internal_provider = InternalProvider::find_for_prop(ctx, map_prop_id)
        .await
        .expect("could not perform find for prop")
        .expect("no implicit internal provider found for prop");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        external_provider_attribute_prototype_id,
        *canoe_from_second_func_argument.id(),
        *map_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Create the component and cache what we need to insert into the map.
    // prototype argument for each item.
    let (component, _) = Component::new(ctx, "the-game-awards-2022", *schema_variant.id())
        .await
        .expect("unable to create component");
    let component_id = *component.id();
    let insert_context = AttributeContext::builder()
        .set_prop_id(map_prop_id)
        .set_component_id(component_id)
        .to_context()
        .expect("could not build attribute context");
    let map_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(map_prop_id),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("attribute value not found");
    let concat_internal_provider = InternalProvider::find_for_prop(ctx, *concat_prop.id())
        .await
        .expect("could not find internal provider for prop")
        .expect("internal provider not found for prop");
    let concat_internal_provider_id = *concat_internal_provider.id();

    // Get what we need to update the user field.
    let domain_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root_prop.domain_prop_id),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("attribute value not found");
    let domain_attribute_value_id = *domain_attribute_value.id();
    let concat_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*concat_prop.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("attribute value not found");
    let concat_attribute_context = AttributeContext::builder()
        .set_prop_id(*concat_prop.id())
        .set_component_id(component_id)
        .to_context()
        .expect("could not build attribute context");

    // Insert one item into the map and set up the function, including the attribute prototype
    // argument.
    let first_inserted_album_attribute_value_id = AttributeValue::insert_for_context(
        ctx,
        insert_context,
        *map_attribute_value.id(),
        Some(serde_json::json![{}]),
        Some("first".to_string()),
    )
    .await
    .expect("could not insert for context");
    let first_inserted_album_attribute_value =
        AttributeValue::get_by_id(ctx, &first_inserted_album_attribute_value_id)
            .await
            .expect("could not perform get by id")
            .expect("attribute value not found by id");
    let mut first_inserted_album_attribute_prototype = first_inserted_album_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not perform attribute prototype for attribute value")
        .expect("no attribute prototype for attribute value");
    first_inserted_album_attribute_prototype
        .set_func_id(ctx, prefix_func_id)
        .await
        .expect("could not set func id for prototype");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *first_inserted_album_attribute_prototype.id(),
        *prefix_func_argument.id(),
        concat_internal_provider_id,
    )
    .await
    .expect("could not create attribute prototype argument");

    // Update the user field once.
    let (_, updated_concat_attribute_value_id) = AttributeValue::update_for_context(
        ctx,
        *concat_attribute_value.id(),
        Some(domain_attribute_value_id),
        concat_attribute_context,
        Some(serde_json::json!["first"]),
        None,
    )
    .await
    .expect("could not update for context");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the view and external provider attribute value looks as we want with one item.
    let component_view = ComponentView::new(ctx, component_id)
        .await
        .expect("cannot get component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "the-game-awards-2022",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "map": {
                        "first": {
                            "poop": "update first",
                            "canoe": "change first",
                        },
                    },
                    "concat": "first",
                },
            }
        ],
        component_view.properties
    );
    assert_eq!(
        Some(serde_json::json![""]), // expected
        dump_value(
            ctx,
            AttributeReadContext {
                external_provider_id: Some(*external_provider.id()),
                component_id: Some(component_id),
                ..AttributeReadContext::default()
            },
        )
        .await, // actual
    );

    // Insert second item into the map and set up the function, including the attribute prototype
    // argument.
    let second_inserted_album_attribute_value_id = AttributeValue::insert_for_context(
        ctx,
        insert_context,
        *map_attribute_value.id(),
        Some(serde_json::json![{}]),
        Some("second".to_string()),
    )
    .await
    .expect("could not insert for context");
    let second_inserted_album_attribute_value =
        AttributeValue::get_by_id(ctx, &second_inserted_album_attribute_value_id)
            .await
            .expect("could not perform get by id")
            .expect("attribute value not found by id");
    let mut second_inserted_album_attribute_prototype = second_inserted_album_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not perform attribute prototype for attribute value")
        .expect("no attribute prototype for attribute value");
    second_inserted_album_attribute_prototype
        .set_func_id(ctx, &suffix_func_id)
        .await
        .expect("could not set func id for prototype");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *second_inserted_album_attribute_prototype.id(),
        *suffix_func_argument.id(),
        concat_internal_provider_id,
    )
    .await
    .expect("could not create attribute prototype argument");

    // Update the user field again.
    AttributeValue::update_for_context(
        ctx,
        updated_concat_attribute_value_id,
        Some(domain_attribute_value_id),
        concat_attribute_context,
        Some(serde_json::json!["second"]),
        None,
    )
    .await
    .expect("could not update for context");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the view looks as we want with two items.
    let component_view = ComponentView::new(ctx, component_id)
        .await
        .expect("cannot get component view");
    assert_eq!(
        serde_json::json![
            {
                "si": {
                    "name": "the-game-awards-2022",
                    "type": "component",
                    "protected": false
                },
                "domain": {
                    "map": {
                        "first": {
                            "poop": "update second",
                            "canoe": "change second",
                        },
                        "second": {
                            "poop": "second update",
                            "canoe": "second change",
                        },
                    },
                    "concat": "second",
                }
            }
        ], // expected
        component_view.properties // actual
    );
    assert_eq!(
        Some(serde_json::json!["second change"]), // expected
        dump_value(
            ctx,
            AttributeReadContext {
                external_provider_id: Some(*external_provider.id()),
                component_id: Some(component_id),
                ..AttributeReadContext::default()
            },
        )
        .await, // actual
    );
}

/// "Dump" the [`serde_json::Value`] within the
/// [`FuncBindingReturnValue`](dal::FuncBindingReturnValue) corresponding to the
/// [`AttributeValue`](dal::AttributeValue) found for a given
/// [`AttributeReadContext`](dal::AttributeReadContext).
async fn dump_value(
    ctx: &DalContext,
    read_context: AttributeReadContext,
) -> Option<serde_json::Value> {
    let attribute_value = AttributeValue::find_for_context(ctx, read_context)
        .await
        .expect("could not perform find for context")
        .expect("attribute value not found");
    attribute_value
        .get_value(ctx)
        .await
        .expect("get value for attribute value")
}
