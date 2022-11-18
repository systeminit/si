use dal::{
    func::backend::js_command::CommandRunResult, generate_name, AttributePrototypeArgument,
    AttributeReadContext, AttributeValue, ChangeSet, ChangeSetStatus, Component, ComponentView,
    DalContext, DiagramKind, Edge, ExternalProvider, InternalProvider, Prop, PropKind, Schema,
    SchemaKind, SocketArity, StandardModel, Visibility, WorkspaceId,
};
use dal_test::{
    helpers::setup_identity_func,
    test,
    test_harness::{
        create_component_and_schema, create_component_for_schema_variant,
        create_prop_of_kind_and_set_parent_with_name, create_schema, create_schema_variant,
        create_schema_variant_with_root,
    },
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;
use veritech_client::ResourceStatus;

mod code;
mod validation;
mod view;

#[test]
async fn new(ctx: &DalContext) {
    let _component = create_component_and_schema(ctx).await;
}

#[test]
async fn new_for_schema_variant_with_node(ctx: &DalContext) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;

    let (component, node) =
        Component::new_for_schema_variant_with_node(ctx, "mastodon", schema_variant.id())
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
async fn schema_relationships(ctx: &DalContext) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;
    let _component = create_component_for_schema_variant(ctx, schema_variant.id()).await;
}

#[test]
async fn qualification_view(ctx: &DalContext) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;

    let prop = Prop::new(ctx, "some_property", PropKind::String, None)
        .await
        .expect("cannot create prop");
    prop.set_parent_prop(ctx, root.domain_prop_id)
        .await
        .expect("Unable to set some_property parent to root.domain");

    schema_variant
        .finalize(ctx)
        .await
        .expect("cannot finalize SchemaVariant");

    let (component, _) =
        Component::new_for_schema_variant_with_node(ctx, "mastodon", schema_variant.id())
            .await
            .expect("Unable to create component");

    let qualification_check_component = component
        .veritech_qualification_check_component(ctx)
        .await
        .expect("cannot create QualificationCheckComponent");

    assert_eq!(
        serde_json::to_value(&qualification_check_component)
            .expect("cannot serialize QualificationCheckComponent"),
        json!({
            "data": {
                "kind": "standard",
                "properties": { "si": { "name": "mastodon" }, "domain": {}, "code": {} },
            },
            "parents": [],
        }),
    );
}

// NOTE: This test is brittle. It's going to rely on the existing configuration of the dockerImage, but it's going
// to prove what we want right now. Figuring out a test that is less brittle is a great idea, but I'm choosing
// expediency.
#[test]
async fn list_qualifications(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, "ash", schema.id())
        .await
        .expect("cannot create `Docker Image` component");

    component
        .check_qualifications(ctx)
        .await
        .expect("cannot check qualifications");
    let qualifications = component
        .list_qualifications(ctx)
        .await
        .expect("cannot list qualifications");
    assert_eq!(qualifications.len(), 2);
}

// Also brittle, same reason
#[test]
async fn list_qualifications_by_component_id(ctx: &DalContext) {
    let schema = Schema::find_by_attr(ctx, "name", &"Docker Image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, "ash", schema.id())
        .await
        .expect("cannot create `Docker Image` component");

    component
        .check_qualifications(ctx)
        .await
        .expect("cannot check qualifications");
    let qualifications = Component::list_qualifications_by_component_id(ctx, *component.id())
        .await
        .expect("cannot list qualifications");
    assert_eq!(qualifications.len(), 2);
}

#[test]
async fn name_from_context(ctx: &DalContext) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;

    let (component, _) =
        Component::new_for_schema_variant_with_node(ctx, "mastodon", schema_variant.id())
            .await
            .expect("cannot create component");
    let _ = Component::new_for_schema_variant_with_node(ctx, "wooly mammoth", schema_variant.id())
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
    let ekwb_schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (ekwb_schema_variant, ekwb_root_prop) =
        create_schema_variant_with_root(ctx, *ekwb_schema.id()).await;
    ekwb_schema_variant
        .finalize(ctx)
        .await
        .expect("unable to finalize schema variant");

    // Create "noctua" schema.
    let noctua_schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (noctua_schema_variant, noctua_root_prop) =
        create_schema_variant_with_root(ctx, *noctua_schema.id()).await;
    let u12a_prop = create_prop_of_kind_and_set_parent_with_name(
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
        AttributeReadContext {
            prop_id: Some(*u12a_prop.id()),
            schema_id: Some(*noctua_schema.id()),
            schema_variant_id: Some(*noctua_schema_variant.id()),
            ..AttributeReadContext::default()
        },
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

    let (ekwb_component, _) =
        Component::new_for_schema_variant_with_node(ctx, "ekwb", ekwb_schema_variant.id())
            .await
            .expect("cannot create component");
    let (noctua_component, _) =
        Component::new_for_schema_variant_with_node(ctx, "noctua", noctua_schema_variant.id())
            .await
            .expect("cannot create component");

    // Connect the two components.
    Edge::connect_providers_for_components(
        ctx,
        *noctua_explicit_internal_provider.id(),
        *noctua_component.id(),
        *ekwb_external_provider.id(),
        *ekwb_component.id(),
    )
    .await
    .expect("could not connect providers for components");

    // Cache the read contexts for generating views for our components.
    let ekwb_component_view_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*ekwb_schema.id()),
        schema_variant_id: Some(*ekwb_schema_variant.id()),
        component_id: Some(*ekwb_component.id()),
        ..AttributeReadContext::default()
    };
    let noctua_component_view_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*noctua_schema.id()),
        schema_variant_id: Some(*noctua_schema_variant.id()),
        component_id: Some(*noctua_component.id()),
        ..AttributeReadContext::default()
    };

    // Ensure everything looks correct post connection.
    let ekwb_component_view = ComponentView::for_context(ctx, ekwb_component_view_context)
        .await
        .expect("could not generate component view");
    let noctua_component_view = ComponentView::for_context(ctx, noctua_component_view_context)
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ekwb",
            },
            "code": {},
            "domain": {},
        }], // expected
        ekwb_component_view.properties // actual
    );
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "noctua",
            },
            "code": {},
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
    let ekwb_component_view = ComponentView::for_context(ctx, ekwb_component_view_context)
        .await
        .expect("could not generate component view");
    let noctua_component_view = ComponentView::for_context(ctx, noctua_component_view_context)
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ekwb",
            },
            "code": {},
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
            "code": {},
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
    let ekwb_component_view = ComponentView::for_context(ctx, ekwb_component_view_context)
        .await
        .expect("could not generate component view");
    let noctua_component_view = ComponentView::for_context(ctx, noctua_component_view_context)
        .await
        .expect("could not generate component view");
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "ekwb",
            },
            "code": {},
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
            "code": {},
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

// FIXME(nick,adam): fix output stream test or figure out another way how to do this. This is
// relatively low priority since it just checks if the output matches the expected between the
// execution output stream itself and the view that was created afterwards.
//
// #[test]
// async fn qualification_view_output_stream() {
//
//     let tenancy = Tenancy::new_universal();
//     let visibility = create_visibility_head();
//     let history_actor = HistoryActor::SystemInit;
//
//     let func = Func::new(
//         &txn,
//         &nats,
//         &(&tenancy).into(),
//         &visibility,
//         &history_actor,
//         "lateralus",
//         FuncBackendKind::JsQualification,
//         FuncBackendResponseType::Qualification,
//     )
//     .await
//     .expect("cannot create func");
//     let args = FuncBackendJsQualificationArgs::new();
//     let args_json = serde_json::to_value(args).expect("cannot serialize args to json");
//     let func_binding = FuncBinding::new(
//         &txn,
//         &nats,
//         &tenancy,
//         &visibility,
//         &HistoryActor::SystemInit,
//         Default::default(),
//         *func.id(),
//         FuncBackendKind::JsQualification,
//     )
//     .await
//     .expect(
//         "could not create func binding",
//     );
//
//     let func_binding_return_value = func_binding
//         .execute(&txn, &nats, veritech)
//         .await
//         .expect("cannot execute binding");
//
//     let output_stream = execution.into_output_stream().expect("output stream empty");
//     let before = output_stream
//         .into_iter()
//         .map(|stream| stream.message)
//         .collect::<HashSet<String>>();
//
//     let qualification_view = QualificationView::new(&txn, func_binding_return_value)
//         .await
//         .expect("could not create qualification view");
//     let after = qualification_view
//         .output
//         .into_iter()
//         .map(|view| view.line)
//         .collect::<HashSet<String>>();
//
//     // NOTE(nick): HashSets are "sorted", so we can compare these directly.
//     assert_eq!(before, after);
// }
