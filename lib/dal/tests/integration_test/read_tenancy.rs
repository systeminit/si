use crate::test_setup;
use dal::{
    test_harness::billing_account_signup, BillingAccountId, OrganizationId, ReadTenancy,
    StandardModel, Tenancy, WorkspaceId,
};
use test_env_log::test;

#[test(tokio::test)]
async fn check_organization_specific_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let write_tenancy = Tenancy::new_organization(vec![*nba.organization.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_organization_in_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read tenancy");
    let write_tenancy = Tenancy::new_billing_account(vec![*nba.billing_account.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_specific_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let write_tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_workspace_in_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read tenancy");
    assert_eq!(tenancy.billing_accounts(), vec![*nba.billing_account.id()]);
    let write_tenancy = Tenancy::new_billing_account(vec![*nba.billing_account.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_specific_organization() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read tenancy");
    assert_eq!(tenancy.billing_accounts(), vec![*nba.billing_account.id()]);
    let write_tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_workspace_in_organization() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read tenancy");
    let write_tenancy = Tenancy::new_organization(vec![*nba.organization.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
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

    let tenancy = ReadTenancy::new_billing_account(vec![BillingAccountId::from(-1)]);

    let write_tenancy = Tenancy::new_empty();
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = Tenancy::new_billing_account(vec![1.into()]);
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = Tenancy::new_organization(vec![1.into()]);
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = Tenancy::new_workspace(vec![1.into()]);
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let mut write_tenancy = Tenancy::new_empty();
    write_tenancy.universal = true;
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let mut write_tenancy = Tenancy::new_billing_account(vec![1.into()]);
    write_tenancy.universal = true;
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let mut write_tenancy = Tenancy::new_organization(vec![1.into()]);
    write_tenancy.universal = true;
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let mut write_tenancy = Tenancy::new_workspace(vec![1.into()]);
    write_tenancy.universal = true;
    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
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

    let tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = Tenancy::new_billing_account(vec![1.into()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
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

    let tenancy = ReadTenancy::new_billing_account(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);
    let write_tenancy = Tenancy::new_billing_account(vec![2.into()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
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

    let tenancy = ReadTenancy::new_billing_account(vec![2.into()]);
    let write_tenancy = Tenancy::new_billing_account(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
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

    let tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = Tenancy::new_billing_account(vec![2.into()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
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

    let tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = Tenancy::new_organization(vec![1.into()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_organization_pk_identical() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read tenancy");
    let write_tenancy = Tenancy::new_organization(vec![*nba.organization.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_organization_pk_overlapping() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba2, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba3, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_organization(
        &txn,
        vec![
            *nba.organization.id(),
            *nba2.organization.id(),
            *nba3.organization.id(),
        ],
    )
    .await
    .expect("unable to set organization read tenancy");
    let write_tenancy = Tenancy::new_organization(vec![*nba2.organization.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_organization_pk_reverse_overlapping() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba2, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba3, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_organization(&txn, vec![*nba2.organization.id()])
        .await
        .expect("unable to set organization read tenancy");
    let write_tenancy = Tenancy::new_organization(vec![
        *nba.organization.id(),
        *nba2.organization.id(),
        *nba3.organization.id(),
    ]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_organization_pk_mismatched() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read tenancy");
    let write_tenancy = Tenancy::new_organization(vec![OrganizationId::from(-1)]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_workspace_pk_identical() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read tenancy");
    let write_tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_pk_overlapping() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba2, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba3, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_workspace(
        &txn,
        vec![
            *nba.workspace.id(),
            *nba2.workspace.id(),
            *nba3.workspace.id(),
        ],
    )
    .await
    .expect("unable to set workspace read tenancy");
    let write_tenancy = Tenancy::new_workspace(vec![*nba2.workspace.id()]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_pk_reverse_overlapping() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba2, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let (nba3, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_workspace(&txn, vec![*nba2.workspace.id()])
        .await
        .expect("unable to set workspace read tenancy");
    let write_tenancy = Tenancy::new_workspace(vec![
        *nba.workspace.id(),
        *nba2.workspace.id(),
        *nba3.workspace.id(),
    ]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_pk_mismatched() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read tenancy");
    let write_tenancy = Tenancy::new_workspace(vec![WorkspaceId::from(-1)]);

    let check = tenancy
        .check(&txn, &write_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}
