use crate::test_setup;

use dal::qualification_resolver::UNSET_ID_VALUE;
use dal::test_harness::{create_schema, create_schema_variant};
use dal::{
    Component, HistoryActor, Prop, PropKind, Schema, SchemaKind, StandardModel, Tenancy, Visibility,
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

    let _component = Component::new_for_schema_variant_with_node(
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
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Implementation,
    )
    .await;

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
    let (mut component, node) = Component::new_for_schema_with_node(
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
    assert_eq!(qualifications.len(), 1);
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
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Implementation,
    )
    .await;

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
    let (mut component, node) = Component::new_for_schema_with_node(
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
    assert_eq!(qualifications.len(), 1);
}
