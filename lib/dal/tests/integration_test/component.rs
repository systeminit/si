use crate::test_setup;

use dal::qualification_resolver::UNSET_ID_VALUE;
use dal::socket::{Socket, SocketArity, SocketEdgeKind};
use dal::test_harness::{create_schema, create_schema_variant, find_or_create_production_system};
use dal::{
    BillingAccount, Component, HistoryActor, Organization, Prop, PropKind, Resource, Schema,
    SchemaKind, StandardModel, Tenancy, Visibility, Workspace,
};
use serde_json::json;

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
        _veritech,
    );
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let _component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create entity");
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
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant");
    let includes_socket = Socket::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
    )
    .await
    .expect("cannot create includes socket for schema variant");
    schema_variant
        .add_socket(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            includes_socket.id(),
        )
        .await
        .expect("cannot add includes socket to schema variant");

    let (component, _node) = Component::new_for_schema_variant_with_node(
        &txn,
        &nats,
        veritech,
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
        _veritech,
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
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant to schema");
    let component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create entity");
    component
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema for entity");
    component
        .set_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot set schema for entity");
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
        _veritech,
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
    let schema_variant =
        create_schema_variant(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_variant
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema variant to schema");
    let component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create entity");
    component
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema for entity");
    component
        .set_schema_variant(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            schema_variant.id(),
        )
        .await
        .expect("cannot set schema for entity");
    let prop = Prop::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "some_property",
        PropKind::String,
    )
    .await
    .expect("cannot create prop");
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
        .veritech_qualification_check_component(&txn, &tenancy, &visibility)
        .await
        .expect("cannot create QualificationCheckComponent");

    assert_eq!(
        serde_json::to_value(&qualification_check_component)
            .expect("cannot serialize QualificationCheckComponent"),
        json!({
            "name": "mastodon",
            "properties": {
                "name": "mastodon"
            }
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
    let (mut component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        "ash",
        schema.id(),
    )
    .await
    .expect("cannot create docker_image component");

    component
        .set_name(&txn, &nats, &visibility, &history_actor, "chvrches")
        .await
        .expect("cannot set name");
    component
        .check_qualifications(
            &txn,
            &nats,
            veritech.clone(),
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
    let (mut component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        "ash",
        schema.id(),
    )
    .await
    .expect("cannot create docker_image component");

    component
        .set_name(&txn, &nats, &visibility, &history_actor, "chvrches")
        .await
        .expect("cannot set name");
    component
        .check_qualifications(
            &txn,
            &nats,
            veritech.clone(),
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

    let (mut component, _node) = Component::new_for_schema_with_node(
        &txn,
        &nats,
        veritech.clone(),
        &tenancy,
        &visibility,
        &history_actor,
        "ash",
        schema.id(),
    )
    .await
    .expect("cannot create ash component");

    component
        .set_name(&txn, &nats, &visibility, &history_actor, "chvrches")
        .await
        .expect("cannot set name");

    component
        .sync_resource(&txn, &nats, veritech.clone(), &history_actor, *system.id())
        .await
        .expect("cannot sync resource");

    let resource = Component::get_resource_by_component_and_system(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        *component.id(),
        *system.id(),
    )
    .await
    .expect("cannot get resource");
    assert_eq!(
        *resource
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
