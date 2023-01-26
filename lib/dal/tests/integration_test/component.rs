use dal::edge::EdgeKind;
use dal::socket::{SocketEdgeKind, SocketKind};
use dal::{
    func::backend::js_command::CommandRunResult, generate_name, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, ChangeSet, ChangeSetStatus, Component, ComponentType,
    ComponentView, Connection, DalContext, DiagramKind, Edge, ExternalProvider, InternalProvider,
    Prop, PropId, PropKind, SchemaVariant, SocketArity, StandardModel, Visibility, WorkspacePk,
};
use dal_test::{
    helpers::builtins::{Builtin, SchemaBuiltinsTestHarness},
    helpers::setup_identity_func,
    test,
    test_harness::{
        create_component_and_schema, create_prop_and_set_parent, create_schema,
        create_schema_variant, create_schema_variant_with_root,
    },
};
use pretty_assertions_sorted::assert_eq;
use veritech_client::ResourceStatus;

mod code;
mod confirmation;
mod qualification;
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
async fn find_type_attribute_value_and_set_type(ctx: &mut DalContext, wid: WorkspacePk) {
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

    // Switch to workspace tenancy with a new change set.
    ctx.update_to_workspace_tenancies(wid)
        .await
        .expect("could not update to workspace tenancies");
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
    let found_attribute_value =
        Component::find_attribute_value(ctx, *component.id(), "type".to_string())
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
    let updated_attribute_value =
        Component::find_attribute_value(ctx, *component.id(), "type".to_string())
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
async fn dependent_values_resource_intelligence(mut octx: DalContext, wid: WorkspacePk) {
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
    let u12a_prop = create_prop_and_set_parent(
        ctx,
        PropKind::String,
        "u12a",
        noctua_root_prop.domain_prop_id,
    )
    .await;
    noctua_schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");

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

    // Create both components.
    ctx.update_to_workspace_tenancies(wid)
        .await
        .expect("could not update to workspace tenancies");
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
            "domain": {},
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
            "domain": {}
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
    ctx.update_visibility(Visibility::new_head(false));

    // Update the resource field on HEAD for the tail end of the relationship.
    ekwb_component
        .set_resource(
            ctx,
            CommandRunResult {
                status: ResourceStatus::Ok,
                value: Some(serde_json::json!["quantum"]),
                logs: Default::default(),
                message: Default::default(),
            },
        )
        .await
        .expect("could not set resource field");

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
            "domain": {},
            "resource": {
                "logs": [],
                "value": "quantum",
                "status": "ok",
            }
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
                    "value": "quantum",
                    "status": "ok",
                }
            }
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
            "domain": {},
            "resource": {
                "logs": [],
                "value": "quantum",
                "status": "ok",
            }
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
                    "value": "quantum",
                    "status": "ok",
                }
            }
        }], // expected
        noctua_component_view.properties // actual
    );
}

#[test]
async fn create_delete_and_restore_components(ctx: &DalContext) {
    let mut harness = SchemaBuiltinsTestHarness::new();
    let nginx_container = harness
        .create_component(ctx, "nginx", Builtin::DockerImage)
        .await;
    let butane_instance = harness
        .create_component(ctx, "userdata", Builtin::CoreOsButane)
        .await;

    let docker_image_schema_variant =
        SchemaVariant::get_by_id(ctx, &nginx_container.schema_variant_id)
            .await
            .expect("could not find schema variant by id")
            .expect("schema variant by id not found");
    let butane_schema_variant = SchemaVariant::get_by_id(ctx, &butane_instance.schema_variant_id)
        .await
        .expect("could not find schema variant by id")
        .expect("schema variant by id not found");

    let docker_image_sockets = docker_image_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");
    let butane_sockets = butane_schema_variant
        .sockets(ctx)
        .await
        .expect("cannot fetch sockets");

    let from_container_image_socket = docker_image_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationOutput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Container Image"
        })
        .expect("cannot find output socket");

    let to_container_image_socket = butane_sockets
        .iter()
        .find(|s| {
            s.edge_kind() == &SocketEdgeKind::ConfigurationInput
                && s.kind() == &SocketKind::Provider
                && s.diagram_kind() == &DiagramKind::Configuration
                && s.arity() == &SocketArity::Many
                && s.name() == "Container Image"
        })
        .expect("cannot find input socket");

    let _connection = Connection::new(
        ctx,
        nginx_container.node_id,
        *from_container_image_socket.id(),
        butane_instance.node_id,
        *to_container_image_socket.id(),
        EdgeKind::Configuration,
    )
    .await
    .expect("could not create connection");

    // required to happen *AFTER* the connection to trigger a dependantValuesUpdate
    nginx_container
        .update_attribute_value_for_prop_name(
            ctx,
            "/root/domain/image",
            Some(serde_json::json!["nginx"]),
        )
        .await;

    // check that the value of the butane instance
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "userdata",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "nginx.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Nginx\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill nginx\nExecStartPre=-/bin/podman rm nginx\nExecStartPre=/bin/podman pull nginx\nExecStart=/bin/podman run --name nginx nginx\n\n[Install]\nWantedBy=multi-user.target",
                        }
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Nginx\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill nginx\\nExecStartPre=-/bin/podman rm nginx\\nExecStartPre=/bin/podman pull nginx\\nExecStart=/bin/podman run --name nginx nginx\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"nginx.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );

    // delete the nginx container
    let comp = Component::get_by_id(ctx, &nginx_container.component_id)
        .await
        .expect("could not find component by id");
    let _result = comp.unwrap().delete_and_propagate(ctx).await;

    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "userdata",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );

    let _result = Component::restore_by_id(ctx, nginx_container.component_id).await;

    // check that the value of the butane instance
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "userdata",
                "type": "component",
                "protected": false,
            },
            "domain": {
                "systemd": {
                    "units": [
                        {
                            "name": "nginx.service",
                            "enabled": true,
                            "contents": "[Unit]\nDescription=Nginx\nAfter=network-online.target\nWants=network-online.target\n\n[Service]\nTimeoutStartSec=0\nExecStartPre=-/bin/podman kill nginx\nExecStartPre=-/bin/podman rm nginx\nExecStartPre=/bin/podman pull nginx\nExecStart=/bin/podman run --name nginx nginx\n\n[Install]\nWantedBy=multi-user.target",
                        }
                    ],
                },
                "variant": "fcos",
                "version": "1.4.0",
            },
            "code": {
                "si:generateButaneIgnition": {
                    "code": "{\n  \"ignition\": {\n    \"version\": \"3.3.0\"\n  },\n  \"systemd\": {\n    \"units\": [\n      {\n        \"contents\": \"[Unit]\\nDescription=Nginx\\nAfter=network-online.target\\nWants=network-online.target\\n\\n[Service]\\nTimeoutStartSec=0\\nExecStartPre=-/bin/podman kill nginx\\nExecStartPre=-/bin/podman rm nginx\\nExecStartPre=/bin/podman pull nginx\\nExecStart=/bin/podman run --name nginx nginx\\n\\n[Install]\\nWantedBy=multi-user.target\",\n        \"enabled\": true,\n        \"name\": \"nginx.service\"\n      }\n    ]\n  }\n}",
                    "format": "json",
                },
            },
        }], // expected
        butane_instance
            .component_view_properties(ctx)
            .await
            .drop_qualification()
            .to_value() // actual
    );
}
