use dal::edge::EdgeKind;
use dal::schema::variant::root_prop::SiPropChild;
use dal::socket::SocketEdgeKind;
use dal::{
    func::backend::js_action::ActionRunResult, generate_name, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, ChangeSet, ChangeSetStatus, Component, ComponentType,
    ComponentView, Connection, DalContext, Edge, ExternalProvider, InternalProvider, Prop, PropId,
    PropKind, SchemaVariant, Socket, SocketArity, StandardModel, Visibility,
};
use dal_test::helpers::component_bag::ComponentBagger;
use dal_test::{
    helpers::setup_identity_func,
    test,
    test_harness::{
        create_component_and_schema, create_schema, create_schema_variant,
        create_schema_variant_with_root,
    },
};
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

mod code;
mod qualification;
mod resource;
mod validation;
mod view;

#[test]
async fn new(ctx: &DalContext) {
    let _component = create_component_and_schema(ctx).await;
}

#[test]
async fn new_for_schema_variant_with_node(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let mut schema_variant = create_schema_variant(ctx, *schema.id()).await;
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("could not finalize schema variant");

    let (component, node) = Component::new(ctx, "mastodon", *schema_variant.id())
        .await
        .expect("cannot create component");

    // Test the find for node query.
    let found_component = Component::find_for_node(ctx, *node.id())
        .await
        .expect("could not find component for node")
        .expect("component for node not found");
    assert_eq!(
        *found_component.id(), // actual
        *component.id()        // expected
    );
}

#[test]
async fn name_from_context(ctx: &DalContext) {
    let schema = create_schema(ctx).await;
    let mut schema_variant = create_schema_variant(ctx, *schema.id()).await;
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("could not finalize schema variant");

    let (component, _) = Component::new(ctx, "mastodon", *schema_variant.id())
        .await
        .expect("cannot create component");
    let _ = Component::new(ctx, "wooly mammoth", *schema_variant.id())
        .await
        .expect("cannot create second component");

    let component_name = component
        .name(ctx)
        .await
        .expect("Unable to retrieve component name");

    assert_eq!(component_name, "mastodon");
}

#[test]
async fn find_type_attribute_value_and_set_type(ctx: &mut DalContext) {
    // Start on head visibility.
    ctx.update_to_head();

    let schema = create_schema(ctx).await;
    let (mut schema_variant, root_prop) = SchemaVariant::new(ctx, *schema.id(), "v0")
        .await
        .expect("cannot create schema variant");
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("cannot finalize schema variant");

    let new_change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));
    let (component, _) = Component::new(ctx, generate_name(), *schema_variant.id())
        .await
        .expect("could not create component");
    let component_id = *component.id();

    // Find the prop corresponding to "/root/si/type". Ensure that we find only one prop.
    let si_prop = Prop::get_by_id(ctx, &root_prop.si_prop_id)
        .await
        .expect("could not perform get by id")
        .expect("prop not found");
    let si_child_props = si_prop
        .child_props(ctx)
        .await
        .expect("could not find child props");
    let mut filtered_si_child_prop_ids: Vec<PropId> = si_child_props
        .iter()
        .filter(|p| p.name() == "type")
        .map(|p| *p.id())
        .collect();
    let type_prop_id = filtered_si_child_prop_ids
        .pop()
        .expect("filtered si child props are empty");
    assert!(filtered_si_child_prop_ids.is_empty());

    // With that prop, find the attribute value that we want to use for our assertion. Ensure
    // that the context is exactly as we expect because we will rely on both the prop and the
    // component fields being accurate for our query.
    let expected_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(type_prop_id),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        },
    )
    .await
    .expect("could not perform find for context")
    .expect("attribute value not found");
    assert_eq!(
        type_prop_id,                               // expected
        expected_attribute_value.context.prop_id()  // actual
    );
    assert_eq!(
        component_id,                                    // expected
        expected_attribute_value.context.component_id()  // actual
    );

    // Now, test our query. Ensure we have the right context too.
    let found_attribute_value = Component::find_si_child_attribute_value(
        ctx,
        *component.id(),
        *schema_variant.id(),
        SiPropChild::Type,
    )
    .await
    .expect("could not find type attribute value");
    assert_eq!(
        expected_attribute_value.context, // expected
        found_attribute_value.context,    // actual
    );

    // Check the found type.
    let found_raw_value = found_attribute_value
        .get_value(ctx)
        .await
        .expect("could not get value from attribute value")
        .expect("value is none");
    let found_component_type: ComponentType =
        serde_json::from_value(found_raw_value).expect("could not deserialize");
    assert_eq!(
        ComponentType::Component, // expected
        found_component_type,     // actual
    );

    // Check our query wrapper.
    let found_component_type_from_wrapper =
        component.get_type(ctx).await.expect("could not get type");
    assert_eq!(
        ComponentType::Component,          // expected
        found_component_type_from_wrapper, // actual
    );

    // Update the type.
    let new_component_type = ComponentType::ConfigurationFrame;
    component
        .set_type(ctx, new_component_type)
        .await
        .expect("could not set type");

    // Check that the type was updated. Ensure that we have the right attribute value too (specific
    // to the component now that's been updated).
    let updated_attribute_value = Component::find_si_child_attribute_value(
        ctx,
        *component.id(),
        *schema_variant.id(),
        SiPropChild::Type,
    )
    .await
    .expect("could not find type attribute value");
    assert_eq!(
        expected_attribute_value.context, // expected
        updated_attribute_value.context,  // actual
    );
    let updated_raw_value = updated_attribute_value
        .get_value(ctx)
        .await
        .expect("could not get value from attribute value")
        .expect("value is none");
    let updated_component_type: ComponentType =
        serde_json::from_value(updated_raw_value).expect("could not deserialize");
    assert_eq!(
        new_component_type,     // expected
        updated_component_type, // actual
    );

    // Check our query wrapper (again).
    let updated_component_type_from_wrapper =
        component.get_type(ctx).await.expect("could not get type");
    assert_eq!(
        new_component_type,                  // expected
        updated_component_type_from_wrapper, // actual
    );
}

#[test]
async fn dependent_values_resource_intelligence(mut octx: DalContext) {
    // Switch to head visibility to author schemas and intra-schema-variant relationships.
    let ctx = &mut octx;
    ctx.update_to_head();

    // Create "ekwb" schema.
    let ekwb_schema = create_schema(ctx).await;
    let (mut ekwb_schema_variant, ekwb_root_prop) =
        create_schema_variant_with_root(ctx, *ekwb_schema.id()).await;
    ekwb_schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    // Create "noctua" schema.
    let noctua_schema = create_schema(ctx).await;
    let (mut noctua_schema_variant, noctua_root_prop) =
        create_schema_variant_with_root(ctx, *noctua_schema.id()).await;
    let u12a_prop = Prop::new(
        ctx,
        "u12a",
        PropKind::String,
        None,
        *noctua_schema_variant.id(),
        Some(noctua_root_prop.domain_prop_id),
    )
    .await
    .expect("could not create prop");
    noctua_schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Gather the identity func.
    let (
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        identity_func_argument_id,
    ) = setup_identity_func(ctx).await;

    // Create "ekwb" output socket.
    let (ekwb_external_provider, _) = ExternalProvider::new_with_socket(
        ctx,
        *ekwb_schema.id(),
        *ekwb_schema_variant.id(),
        "Cooling",
        None,
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create external provider");
    let ekwb_resource_internal_provider =
        InternalProvider::find_for_prop(ctx, ekwb_root_prop.resource_prop_id)
            .await
            .expect("could not perform internal provider get for prop")
            .expect("internal provider not found for prop");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *ekwb_external_provider
            .attribute_prototype_id()
            .expect("no attribute prototype id for external provider"),
        identity_func_argument_id,
        *ekwb_resource_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    // Create "noctua" input socket.
    let (noctua_explicit_internal_provider, _) = InternalProvider::new_explicit_with_socket(
        ctx,
        *noctua_schema_variant.id(),
        "Cooling",
        identity_func_id,
        identity_func_binding_id,
        identity_func_binding_return_value_id,
        SocketArity::Many,
        false,
    )
    .await
    .expect("could not create explicit internal provider");
    let u12a_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext::default_with_prop(*u12a_prop.id()),
    )
    .await
    .expect("could not perform attribute value find for context")
    .expect("attribute value not found");
    let mut u12a_attribute_prototype = u12a_attribute_value
        .attribute_prototype(ctx)
        .await
        .expect("could not fetch attribute prototype for attribute value")
        .expect("attribute prototype not found for attribute value");
    u12a_attribute_prototype
        .set_func_id(ctx, identity_func_id)
        .await
        .expect("could not set func id for attribute prototype");
    AttributePrototypeArgument::new_for_intra_component(
        ctx,
        *u12a_attribute_prototype.id(),
        identity_func_argument_id,
        *noctua_explicit_internal_provider.id(),
    )
    .await
    .expect("could not create attribute prototype argument");

    let new_change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));

    let (ekwb_component, _) = Component::new(ctx, "ekwb", *ekwb_schema_variant.id())
        .await
        .expect("cannot create component");
    let (noctua_component, _) = Component::new(ctx, "noctua", *noctua_schema_variant.id())
        .await
        .expect("cannot create component");
    let ekwb_component_id = *ekwb_component.id();
    let noctua_component_id = *noctua_component.id();

    // Connect the two components.
    Edge::connect_providers_for_components(
        ctx,
        *noctua_explicit_internal_provider.id(),
        noctua_component_id,
        *ekwb_external_provider.id(),
        ekwb_component_id,
    )
    .await
    .expect("could not connect providers for components");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure everything looks correct post connection.
    let ekwb_component_view = ComponentView::new(ctx, ekwb_component_id)
        .await
        .expect("could not generate component view");
    let noctua_component_view = ComponentView::new(ctx, noctua_component_id)
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ekwb",
                "type": "component",
                "protected": false
            },
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
                "type": "component",
                "protected": false
            },
        }], // expected
        noctua_component_view.properties // actual
    );

    // Now, merge the change set and ensure we are on HEAD.
    assert_eq!(new_change_set.pk, ctx.visibility().change_set_pk);
    let mut change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
        .await
        .expect("could not fetch change set by pk")
        .expect("no change set found for pk");
    change_set
        .apply(ctx)
        .await
        .expect("cannot apply change set");
    assert_eq!(&change_set.status, &ChangeSetStatus::Applied);

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    ctx.update_visibility(Visibility::new_head(false));

    // Update the resource field on HEAD for the tail end of the relationship.
    ekwb_component
        .set_resource(
            ctx,
            ActionRunResult {
                status: ResourceStatus::Ok,
                payload: Some(serde_json::json![{ "quantum": true }]),
                logs: Default::default(),
                message: Default::default(),
                last_synced: Default::default(),
            },
        )
        .await
        .expect("could not set resource field");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    // Ensure the value is propagated end-to-end.
    let ekwb_component_view = ComponentView::new(ctx, ekwb_component_id)
        .await
        .expect("could not generate component view");
    let noctua_component_view = ComponentView::new(ctx, noctua_component_id)
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ekwb",
                "type": "component",
                "protected": false
            },
            "resource": {
                "logs": [],
                "payload": { "quantum": true },
                "status": "ok",
            },
            "resource_value": {}
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
                "type": "component",
                "protected": false
            },
            "domain": {
                "u12a": {
                    "logs": [],
                    "payload": { "quantum": true },
                    "status": "ok",
                }
            },
        }], // expected
        noctua_component_view.properties // actual
    );

    // Create a new change set and change our visibility to it (analogous to opening a new change
    // set in the frontend).
    let new_change_set = ChangeSet::new(ctx, "poop", None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(new_change_set.pk, None));

    // Ensure the views are identical to HEAD.
    let ekwb_component_view = ComponentView::new(ctx, ekwb_component_id)
        .await
        .expect("could not generate component view");
    let noctua_component_view = ComponentView::new(ctx, noctua_component_id)
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ekwb",
                "type": "component",
                "protected": false
            },
            "resource": {
                "logs": [],
                "payload": { "quantum": true },
                "status": "ok",
            },
            "resource_value": {}
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
                "type": "component",
                "protected": false
            },
            "domain": {
                "u12a": {
                    "logs": [],
                    "payload": { "quantum": true },
                    "status": "ok",
                }
            },
        }], // expected
        noctua_component_view.properties // actual
    );
}

#[test]
async fn create_delete_and_restore_components(ctx: &mut DalContext) {
    let mut bagger = ComponentBagger::new();

    // Restoration is only a well defined operation for objects that existed on HEAD at some point
    // so for this test, we need to create and merge the component before running delete and restore

    let mut change_set = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(change_set.pk, None));

    let fallout_bag = bagger.create_component(ctx, "source", "fallout").await;
    let starfield_bag = bagger
        .create_component(ctx, "destination", "starfield")
        .await;

    let from_fallout_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "fallout",
        SocketEdgeKind::ConfigurationOutput,
        fallout_bag.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find fallout socket");
    let to_fallout_socket = Socket::find_by_name_for_edge_kind_and_node(
        ctx,
        "fallout",
        SocketEdgeKind::ConfigurationInput,
        starfield_bag.node_id,
    )
    .await
    .expect("could not perform socket find'")
    .expect("could not find socket");

    let _connection = Connection::new(
        ctx,
        fallout_bag.node_id,
        *from_fallout_socket.id(),
        starfield_bag.node_id,
        *to_fallout_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    // required to happen *AFTER* the connection to trigger a dependantValuesUpdate
    let name_prop = fallout_bag.find_prop(ctx, &["root", "si", "name"]).await;
    let rads_prop = fallout_bag
        .find_prop(ctx, &["root", "domain", "rads"])
        .await;
    fallout_bag
        .update_attribute_value_for_prop(
            ctx,
            *name_prop.id(),
            Some(serde_json::json!["source-updated"]),
        )
        .await;
    fallout_bag
        .update_attribute_value_for_prop(ctx, *rads_prop.id(), Some(serde_json::json![1]))
        .await;

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "source-updated",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "source-updated",
               "rads": 1,
               "active": true
           },
        }], // expected
        fallout_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "destination",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "destination",
               "universe": {
                   "galaxies": [
                       {
                           "sun": "source-updated-sun",
                           "planets": 1
                       },
                   ],
               },
           },
        }], // expected
        starfield_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );

    // Apply changeset
    change_set
        .apply(ctx)
        .await
        .expect("could not apply change set");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    let change_set_2 = ChangeSet::new(ctx, generate_name(), None)
        .await
        .expect("could not create new change set");
    ctx.update_visibility(Visibility::new(change_set_2.pk, None));

    // delete the source component
    let mut component = Component::get_by_id(ctx, &fallout_bag.component_id)
        .await
        .expect("could not retrieve component by id")
        .expect("component missing");
    component
        .delete_and_propagate(ctx)
        .await
        .expect("Deletion of nginx component should work");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "destination",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "destination",
               "universe": {
                   "galaxies": [],
               },
           },
        }], // expected
        starfield_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );

    Component::restore_and_propagate(ctx, fallout_bag.component_id)
        .await
        .expect("Restoring nginx component should work");

    ctx.blocking_commit()
        .await
        .expect("could not commit & run jobs");

    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "source-updated",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "source-updated",
               "rads": 1,
               "active": true
           },
        }], // expected
        fallout_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );
    assert_eq!(
        serde_json::json![{
           "si": {
               "name": "destination",
               "type": "component",
               "color": "#ffffff",
               "protected": false,
           },
           "domain": {
               "name": "destination",
               "universe": {
                   "galaxies": [
                       {
                           "sun": "source-updated-sun",
                           "planets": 1
                       },
                   ],
               },
           },
        }], // expected
        starfield_bag
            .component_view_properties(ctx)
            .await
            .to_value()
            .expect("could not convert to value") // actual
    );
}
