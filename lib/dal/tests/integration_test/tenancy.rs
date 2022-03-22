use crate::dal::test;
use crate::test_setup;
use dal::Tenancy;

#[test]
async fn check_universal() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_universal();
    let read_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_empty_always_fails() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_empty();
    let read_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_billing_account_pk_identical() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_billing_account(vec![1.into()]);
    let read_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_overlapping() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_billing_account(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);
    let read_tenancy = Tenancy::new_billing_account(vec![2.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_reverse_overlapping() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_billing_account(vec![2.into()]);
    let read_tenancy = Tenancy::new_billing_account(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_billing_account_pk_mismatched() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_billing_account(vec![1.into()]);
    let read_tenancy = Tenancy::new_billing_account(vec![2.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_billing_account_pk_mismatched_level() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_organization(vec![1.into()]);
    let read_tenancy = Tenancy::new_billing_account(vec![1.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_organization_pk_identical() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_organization(vec![1.into()]);
    let read_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_overlapping() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_organization(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);
    let read_tenancy = Tenancy::new_organization(vec![2.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_reverse_overlapping() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_organization(vec![2.into()]);
    let read_tenancy = Tenancy::new_organization(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_organization_pk_mismatched() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_organization(vec![1.into()]);
    let read_tenancy = Tenancy::new_organization(vec![2.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test]
async fn check_workspace_pk_identical() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_workspace(vec![1.into()]);
    let read_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_overlapping() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_workspace(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);
    let read_tenancy = Tenancy::new_workspace(vec![2.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_reverse_overlapping() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_workspace(vec![2.into()]);
    let read_tenancy = Tenancy::new_workspace(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test]
async fn check_workspace_pk_mismatched() {
    test_setup!(
        ctx,
        _secret_key,
        pg,
        conn,
        txn,
        nats_conn,
        _nats,
        _veritech,
        _encr_key
    );

    let tenancy = Tenancy::new_workspace(vec![1.into()]);
    let read_tenancy = Tenancy::new_workspace(vec![2.into()]);

    let check = tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}
