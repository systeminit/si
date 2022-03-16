use crate::test_setup;

use dal::test_harness::create_component_and_schema;
use dal::{HistoryActor, Resource, StandardModel, System, Tenancy, Visibility, WriteTenancy};
use test_env_log::test;

#[test(tokio::test)]
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
    let component = create_component_and_schema(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
    )
    .await;
    let system = System::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        "production system",
    )
    .await
    .expect("cannot create system");

    let _resource = Resource::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        component.id(),
        system.id(),
    )
    .await
    .expect("cannot create resource for component/system");
}

#[test(tokio::test)]
async fn get_by_component_and_system_id() {
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
    let mastodon_component = create_component_and_schema(
        &txn,
        &nats,
        veritech.clone(),
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
    )
    .await;
    let blue_oyster_component = create_component_and_schema(
        &txn,
        &nats,
        veritech,
        &encr_key,
        &tenancy,
        &visibility,
        &history_actor,
    )
    .await;

    let write_tenancy = WriteTenancy::from(&tenancy);
    let read_tenancy = write_tenancy
        .clone_into_read_tenancy(&txn)
        .await
        .expect("failed to generate read tenancy");
    let production_system = System::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "production system",
    )
    .await
    .expect("cannot create system");
    let staging_system = System::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        "staging system",
    )
    .await
    .expect("cannot create staging system");
    let original_resource = Resource::new(
        &txn,
        &nats,
        &write_tenancy,
        &visibility,
        &history_actor,
        mastodon_component.id(),
        production_system.id(),
    )
    .await
    .expect("cannot create resource for component/system");

    // None of the following should be found by `Resource::get_by_component_id_and_system_id`.
    let _different_component_in_same_system = Resource::new(
        &txn,
        &nats,
        &write_tenancy,
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
        &write_tenancy,
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
        &write_tenancy,
        &visibility,
        &history_actor,
        blue_oyster_component.id(),
        staging_system.id(),
    )
    .await
    .expect("cannot create resource for different component in different system");

    let found_resource = Resource::get_by_component_id_and_system_id(
        &txn,
        &read_tenancy,
        &visibility,
        mastodon_component.id(),
        production_system.id(),
    )
    .await
    .expect("cannot retrieve resource for component/system");

    let found_resource = found_resource.expect("unable to get resource from component and system");
    assert_eq!(
        original_resource, found_resource,
        "Resource::get_by_component_id_and_system_id needs to find the same resource we created"
    )
}
