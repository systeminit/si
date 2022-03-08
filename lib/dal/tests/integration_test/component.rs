use crate::test_setup;

use dal::qualification_resolver::UNSET_ID_VALUE;
use dal::test_harness::{
    create_component_and_schema, create_component_for_schema_variant, create_schema,
    create_schema_variant, create_schema_variant_with_root, find_or_create_production_system,
};
use dal::{
    BillingAccount, Component, HistoryActor, Organization, Prop, PropKind, Resource, Schema,
    SchemaKind, StandardModel, Tenancy, Visibility, Workspace,
};
use pretty_assertions_sorted::{assert_eq, assert_eq_sorted};
use serde_json::json;

mod view;

#[tokio::test]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _component = create_component_and_schema(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
    )
    .await;
}

#[tokio::test]
async fn new_for_schema_variant_with_node() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let system =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concept,
    )
    .await;
    let schema_variant = create_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encr_key,
    )
    .await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant");

    let (component, _node) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        schema_variant.id(),
    )
    .await
    .expect("cannot create component");

    // All components get a Resource record when created.
    let resource = Resource::get_by_component_id_and_system_id(
        &txn,
        &tenancy,
        &visibility,
        component.id(),
        system.id(),
    )
    .await
    .expect("cannot retrieve resource for Component & System");
    assert!(resource.is_some());
    let resource = resource.unwrap();
    assert_eq!(
        resource
            .component(&txn, &visibility)
            .await
            .expect("cannot retrieve component for resource")
            .expect("no component found for resource")
            .id(),
        component.id()
    );
    assert_eq!(
        resource
            .system(&txn, &visibility)
            .await
            .expect("cannot retrieve system for resource")
            .expect("no system found for resource")
            .id(),
        system.id()
    );
}

#[tokio::test]
async fn schema_relationships() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Implementation,
    )
    .await;
    let schema_variant = create_schema_variant(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encr_key,
    )
    .await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant to schema");
    let _component = create_component_for_schema_variant(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await;
}

#[tokio::test]
async fn qualification_view() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Implementation,
    )
    .await;
    let (schema_variant, root) = create_schema_variant_with_root(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        veritech.clone(),
        encr_key,
    )
    .await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant to schema");
    let (component, _) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        schema_variant.id(),
    )
    .await
    .expect("Unable to create component");

    let prop = Prop::new(
        &txn,
        &nats,
        veritech,
        encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "some_property",
        PropKind::String,
    )
    .await
    .expect("cannot create prop");
    prop.set_parent_prop(
        &txn,
        &nats,
        &visibility,
        &history_actor,
        root.domain_prop_id,
    )
    .await
    .expect("Unable to set some_property parent to root.domain");
    prop.add_schema_variant(
        &txn,
        &nats,
        &visibility,
        &history_actor,
        schema_variant.id(),
    )
    .await
    .expect("cannot add schema variant for prop");

    let qualification_check_component = component
        .veritech_qualification_check_component(&txn, &tenancy, &visibility, UNSET_ID_VALUE.into())
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
#[tokio::test]
async fn list_qualifications() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image schema")
    .pop()
    .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "ash",
        schema.id(),
    )
    .await
    .expect("cannot create docker_image component");

    component
        .check_qualifications(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            UNSET_ID_VALUE.into(),
        )
        .await
        .expect("cannot check qualifications");
    let qualifications = component
        .list_qualifications(&txn, &tenancy, &visibility, UNSET_ID_VALUE.into())
        .await
        .expect("cannot list qualifications");
    assert_eq!(qualifications.len(), 2);
}

// Also brittle, same reason
#[tokio::test]
async fn list_qualifications_by_component_id() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _ =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let schema = Schema::find_by_attr(
        &txn,
        &tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image schema")
    .pop()
    .expect("no docker image schema found");
    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "ash",
        schema.id(),
    )
    .await
    .expect("cannot create docker_image component");

    component
        .check_qualifications(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &tenancy,
            &visibility,
            &history_actor,
            UNSET_ID_VALUE.into(),
        )
        .await
        .expect("cannot check qualifications");
    let qualifications = Component::list_qualifications_by_component_id(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        UNSET_ID_VALUE.into(),
    )
    .await
    .expect("cannot list qualifications");
    assert_eq!(qualifications.len(), 2);
}

// Also brittle, same reason
#[tokio::test]
async fn get_resource_by_component_id() {
    test_setup!(
        ctx,
        _secret_key,
        _pg,
        _conn,
        txn,
        _nats_conn,
        nats,
        veritech,
        encr_key,
    );

    let billing_account_tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;

    let billing_account = BillingAccount::new(
        &txn,
        &nats,
        &billing_account_tenancy,
        &visibility,
        &history_actor,
        "coheed",
        Some(&"coheed and cambria".to_string()),
    )
    .await
    .expect("cannot create new billing account");

    let organization_tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);
    let organization = Organization::new(
        &txn,
        &nats,
        &organization_tenancy,
        &visibility,
        &history_actor,
        "iron maiden",
    )
    .await
    .expect("cannot create organization");

    let mut workspace_tenancy = Tenancy::new_organization(vec![*organization.id()]);
    workspace_tenancy
        .billing_account_ids
        .push(*billing_account.id());
    let workspace = Workspace::new(
        &txn,
        &nats,
        &workspace_tenancy,
        &visibility,
        &history_actor,
        "iron maiden",
    )
    .await
    .expect("cannot create workspace");

    workspace
        .set_organization(&txn, &nats, &visibility, &history_actor, organization.id())
        .await
        .expect("Unable to set organization to workspace");

    let tenancy = Tenancy::new_workspace(vec![*workspace.id()]);

    let mut schema_tenancy = tenancy.clone();
    schema_tenancy.universal = true;

    let schema = Schema::find_by_attr(
        &txn,
        &schema_tenancy,
        &visibility,
        "name",
        &"docker_image".to_string(),
    )
    .await
    .expect("cannot find docker image schema")
    .pop()
    .expect("no docker image schema found");

    let system =
        find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    let (component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
        "chvrches",
        schema.id(),
    )
    .await
    .expect("cannot create ash component");

    component
        .sync_resource(
            &txn,
            &nats,
            veritech.clone(),
            encr_key,
            &history_actor,
            *system.id(),
        )
        .await
        .expect("cannot sync resource");

    let resource = Component::get_resource_by_component_and_system(
        &txn,
        &tenancy,
        &visibility,
        *component.id(),
        *system.id(),
    )
    .await
    .expect("cannot get resource");
    assert_eq!(
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
        serde_json::json!("Cant touch this: chvrches")
    );
}

// FIXME(nick,adam): fix output stream test or figure out another way how to do this. This is
// relatively low priority since it just checks if the output matches the expected between the
// execution output stream itself and the view that was created afterwards.
//
// #[tokio::test]
// async fn qualification_view_output_stream() {
//     test_setup!(ctx, _secret_key, _pg, _conn, txn, nats_conn, nats, veritech, _encr_key);
//     let tenancy = Tenancy::new_universal();
//     let visibility = create_visibility_head();
//     let history_actor = HistoryActor::SystemInit;
//
//     let func = Func::new(
//         &txn,
//         &nats,
//         &tenancy,
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
