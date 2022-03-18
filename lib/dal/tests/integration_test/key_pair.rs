use crate::test_setup;

use dal::key_pair::PublicKey;
use dal::test_harness::{
    create_billing_account, create_change_set, create_edit_session, create_key_pair,
    create_visibility_edit_session,
};
use dal::{BillingAccount, HistoryActor, KeyPair, StandardModel, Tenancy};
use test_env_log::test;

#[test(tokio::test)]
async fn new() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key
    );
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let _key_pair = KeyPair::new(
        &txn,
        &nats,
        &(&tenancy).into(),
        &visibility,
        &history_actor,
        "funky",
    )
    .await
    .expect("cannot create key_pair");
}

#[test(tokio::test)]
async fn belongs_to() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key
    );
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;

    key_pair
        .set_billing_account(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            billing_account.id(),
        )
        .await
        .expect("cannot set billing account");

    let belongs_to_ba: BillingAccount = key_pair
        .billing_account(&txn, &visibility)
        .await
        .expect("cannot get belongs to billing account")
        .expect("billing account should exist");
    assert_eq!(&billing_account, &belongs_to_ba);

    key_pair
        .unset_billing_account(&txn, &nats, &visibility, &history_actor)
        .await
        .expect("cannot set billing account");

    let belongs_to_ba: Option<BillingAccount> = key_pair
        .billing_account(&txn, &visibility)
        .await
        .expect("cannot get belongs to billing account");
    assert!(
        belongs_to_ba.is_none(),
        "billing account is not associated anymore"
    );
}

#[test(tokio::test)]
async fn public_key_get_current() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        nats,
        _veritech,
        _encr_key
    );
    let tenancy = Tenancy::new_universal();
    let history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(&txn, &nats, &tenancy, &history_actor).await;
    let edit_session = create_edit_session(&txn, &nats, &history_actor, &change_set).await;
    let visibility = create_visibility_edit_session(&change_set, &edit_session);
    let billing_account =
        create_billing_account(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    let first_key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    first_key_pair
        .set_billing_account(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            billing_account.id(),
        )
        .await
        .expect("cannot set billing account");
    let second_key_pair = create_key_pair(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    second_key_pair
        .set_billing_account(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            billing_account.id(),
        )
        .await
        .expect("cannot set billing account");

    let pk = PublicKey::get_current(
        &txn,
        &tenancy
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to generate read tenancy"),
        &visibility,
        billing_account.id(),
    )
    .await
    .expect("cannot get public key");

    assert_eq!(second_key_pair.pk(), pk.pk());
    assert_eq!(second_key_pair.public_key(), pk.public_key());
}
