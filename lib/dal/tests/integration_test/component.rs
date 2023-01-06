use dal::{
    func::backend::js_command::CommandRunResult, generate_name, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, ChangeSet, ChangeSetStatus, Component, ComponentView,
    DalContext, DiagramKind, Edge, ExternalProvider, InternalProvider, PropKind, SocketArity,
    StandardModel, Visibility, WorkspaceId,
};
use dal_test::{
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
        .finalize(ctx)
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
        .finalize(ctx)
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
async fn dependent_values_resource_intelligence(mut octx: DalContext, wid: WorkspaceId) {
    // Switch to universal head (tenancy and visibility) to author schemas and
    // intra-schema-variant relationships.
    let ctx = &mut octx;
    ctx.update_to_universal_head();

    // Create "ekwb" schema.
    let ekwb_schema = create_schema(ctx).await;
    let (mut ekwb_schema_variant, ekwb_root_prop) =
        create_schema_variant_with_root(ctx, *ekwb_schema.id()).await;
    ekwb_schema_variant
        .finalize(ctx)
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
        .finalize(ctx)
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
        DiagramKind::Configuration,
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
        DiagramKind::Configuration,
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
            },
            "domain": {},
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
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
            },

            "domain": {},
            "resource": {
                "status": "ok",
                "value": "quantum",
                "logs": [],
            }
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
            },
            "domain": {
                "u12a": {
                    "status": "ok",
                    "value": "quantum",
                    "logs": [],
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
            },

            "domain": {},
            "resource": {
                "status": "ok",
                "value": "quantum",
                "logs": [],
            }
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
            },
            "domain": {
                "u12a": {
                    "status": "ok",
                    "value": "quantum",
                    "logs": [],
                }
            }
        }], // expected
        noctua_component_view.properties // actual
    );
}
