use dal::{
    qualification_resolver::UNSET_ID_VALUE,
    test::helpers::create_system_with_node,
    test_harness::{
        create_component_and_schema, create_component_for_schema_variant, create_schema,
        create_schema_variant, create_schema_variant_with_root,
    },
    Component, DalContext, Prop, PropKind, Resource, Schema, SchemaKind, StandardModel,
    WorkspaceId,
};
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};
use serde_json::json;

use crate::dal::test;

mod view;

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _component = create_component_and_schema(ctx).await;
}

#[test]
async fn new_for_schema_variant_with_node(ctx: &DalContext<'_, '_>, wid: WorkspaceId) {
    let (system, _system_node) = create_system_with_node(ctx, &wid).await;

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

    // A components does not get a Resource record when created.
    let resource = Resource::get_by_component_id_and_system_id(ctx, component.id(), system.id())
        .await
        .expect("cannot retrieve resource for Component & System");
    assert!(resource.is_none());

    component
        .add_to_system(ctx, *system.id())
        .await
        .expect("failed to add node to system");

    // A components gets a Resource record when added to a system.
    let resource = Resource::get_by_component_id_and_system_id(ctx, component.id(), system.id())
        .await
        .expect("cannot retrieve resource for Component & System");
    assert!(resource.is_some());

    let resource = resource.unwrap();
    assert_eq!(
        resource
            .component(ctx)
            .await
            .expect("cannot retrieve component for resource")
            .expect("no component found for resource")
            .id(),
        component.id()
    );
    assert_eq!(
        resource
            .system(ctx)
            .await
            .expect("cannot retrieve system for resource")
            .expect("no system found for resource")
            .id(),
        system.id()
    );
}

#[test]
async fn schema_relationships(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;
    let _component = create_component_for_schema_variant(ctx, schema_variant.id()).await;
}

#[test]
async fn qualification_view(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;

    let prop = Prop::new(ctx, "some_property", PropKind::String)
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
        .veritech_qualification_check_component(ctx, UNSET_ID_VALUE.into())
        .await
        .expect("cannot create QualificationCheckComponent");

    assert_eq_sorted!(
        serde_json::to_value(&qualification_check_component)
            .expect("cannot serialize QualificationCheckComponent"),
        json!({
            "data": {
                "system": null,
                "kind": "standard",
                "properties": { "si": { "name": "mastodon" }, "domain": {} }
            },
            "parents": [],
            "codes": []
        }),
    );
}

// NOTE: This test is brittle. It's going to rely on the existing configuration of the dockerImage, but it's going
// to prove what we want right now. Figuring out a test that is less brittle is a great idea, but I'm choosing
// expediency.
#[test]
async fn list_qualifications(ctx: &DalContext<'_, '_>) {
    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, "ash", schema.id())
        .await
        .expect("cannot create docker_image component");

    component
        .check_qualifications(ctx, UNSET_ID_VALUE.into())
        .await
        .expect("cannot check qualifications");
    let qualifications = component
        .list_qualifications(ctx, UNSET_ID_VALUE.into())
        .await
        .expect("cannot list qualifications");
    assert_eq!(qualifications.len(), 2);
}

// Also brittle, same reason
#[test]
async fn list_qualifications_by_component_id(ctx: &DalContext<'_, '_>) {
    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, "ash", schema.id())
        .await
        .expect("cannot create docker_image component");

    component
        .check_qualifications(ctx, UNSET_ID_VALUE.into())
        .await
        .expect("cannot check qualifications");
    let qualifications =
        Component::list_qualifications_by_component_id(ctx, *component.id(), UNSET_ID_VALUE.into())
            .await
            .expect("cannot list qualifications");
    assert_eq!(qualifications.len(), 2);
}

// Also brittle, same reason
#[test]
async fn get_resource_by_component_id(ctx: &DalContext<'_, '_>, wid: WorkspaceId) {
    let (system, _system_node) = create_system_with_node(ctx, &wid).await;

    let schema = Schema::find_by_attr(ctx, "name", &"docker_image".to_string())
        .await
        .expect("cannot find docker image schema")
        .pop()
        .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(ctx, "chvrches", schema.id())
        .await
        .expect("cannot create ash component");

    component
        .add_to_system(ctx, *system.id())
        .await
        .expect("failed to add component to system");

    component
        .sync_resource(ctx, *system.id())
        .await
        .expect("cannot sync resource");

    let resource =
        Component::get_resource_by_component_and_system(ctx, *component.id(), *system.id())
            .await
            .expect("cannot get resource");

    assert_eq!(
        serde_json::json!("Cant touch this: chvrches"),
        *resource
            .expect("Resource missing")
            .data
            .as_object()
            .expect("None resource sync data")
            .get("data")
            .expect("Missing 'data' key from resource sync data")
            .as_object()
            .expect("Null 'data' key")
            .get("data")
            .expect("Missing 'data.data' key from resource sync data")
            .get("name")
            .expect("Missing name in resource sync data"),
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
