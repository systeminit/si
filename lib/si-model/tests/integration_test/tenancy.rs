use si_model::Tenancy;
use crate::test_setup;

#[tokio::test]
async fn check_universal() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_universal();
    let check_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_empty_always_fails() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_empty();
    let check_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[tokio::test]
async fn check_billing_account_pk_identical() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_billing_account(vec![1]);
    let check_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_billing_account_pk_overlapping() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_billing_account(vec![1, 2, 3, 4, 5, 6]);
    let check_tenancy = Tenancy::new_billing_account(vec![2]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_billing_account_pk_mismatched() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_billing_account(vec![1]);
    let check_tenancy = Tenancy::new_billing_account(vec![2]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[tokio::test]
async fn check_billing_account_pk_mismatched_level() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_billing_account(vec![1]);
    let check_tenancy = Tenancy::new_organization(vec![1]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}


#[tokio::test]
async fn check_organization_pk_identical() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_organization(vec![1]);
    let check_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_organization_pk_overlapping() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_organization(vec![1, 2, 3, 4, 5, 6]);
    let check_tenancy = Tenancy::new_organization(vec![2]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_organization_pk_mismatched() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_organization(vec![1]);
    let check_tenancy = Tenancy::new_organization(vec![2]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[tokio::test]
async fn check_workspace_pk_identical() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_workspace(vec![1]);
    let check_tenancy = tenancy.clone();

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_workspace_pk_overlapping() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_workspace(vec![1, 2, 3, 4, 5, 6]);
    let check_tenancy = Tenancy::new_workspace(vec![2]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[tokio::test]
async fn check_workspace_pk_mismatched() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, _nats);

    let tenancy = Tenancy::new_workspace(vec![1]);
    let check_tenancy = Tenancy::new_workspace(vec![2]);

    let check = tenancy
        .check(&txn, &check_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}
