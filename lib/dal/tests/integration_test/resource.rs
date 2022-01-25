use crate::test_setup;

use dal::{Component, HistoryActor, Resource, StandardModel, System, Tenancy, Visibility};

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
    let component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create component");
    let system = System::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "production system",
    )
    .await
    .expect("cannot create system");

    let _resource = Resource::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        component.id(),
        system.id(),
    )
    .await
    .expect("cannot create resource for component/system");
}

#[tokio::test]
async fn find_for_component_and_system_id() {
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
    let mastodon_component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
    )
    .await
    .expect("cannot create component");
    let blue_oyster_component = Component::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "Blue Öyster Cult",
    )
    .await
    .expect("cannot create component");
    let production_system = System::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "production system",
    )
    .await
    .expect("cannot create system");
    let staging_system = System::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "staging system",
    )
    .await
    .expect("cannot create staging system");

    let original_resource = Resource::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        mastodon_component.id(),
        production_system.id(),
    )
    .await
    .expect("cannot create resource for component/system");

    // None of the following should be found by `Resource::find_for_component_id_and_system_id`.
    let _different_component_in_same_system = Resource::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        blue_oyster_component.id(),
        production_system.id(),
    )
    .await
    .expect("cannot create resource for different component in same system");
    let _same_component_in_different_system = Resource::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        mastodon_component.id(),
        staging_system.id(),
    )
    .await
    .expect("cannot create resource for same component in different system");
    let _different_component_in_different_system = Resource::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        blue_oyster_component.id(),
        staging_system.id(),
    )
    .await
    .expect("cannot create resource for different component in different system");

    let mut found_resources = Resource::find_for_component_id_and_system_id(
        &txn,
        &tenancy,
        &visibility,
        mastodon_component.id(),
        production_system.id(),
    )
    .await
    .expect("cannot retrieve resource for component/system");

    assert_eq!(found_resources.len(), 1);
    let found_resource = found_resources
        .pop()
        .expect("unable to pop resource from vec");
    assert_eq!(
        original_resource, found_resource,
        "Resource::find_for_component_id_and_system_id needs to find the same resource we created"
    )
}
