use crate::test_setup;
use dal::{
    test_harness::billing_account_signup, BillingAccountId, OrganizationId, ReadTenancy,
    StandardModel, Tenancy, WorkspaceId, WriteTenancy,
};
use test_env_log::test;

#[test(tokio::test)]
async fn check_organization_specific_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_organization_in_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let read_tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_specific_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_workspace_in_billing_account() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read read_tenancy");
    assert_eq!(
        read_tenancy.billing_accounts(),
        vec![*nba.billing_account.id()]
    );
    let write_tenancy = WriteTenancy::new_billing_account(*nba.billing_account.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_specific_organization() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let read_tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read read_tenancy");
    assert_eq!(
        read_tenancy.billing_accounts(),
        vec![*nba.billing_account.id()]
    );
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_workspace_in_organization() {
    test_setup!(ctx, secret_key, _pg, _conn, txn, nats_conn, nats, _veritech, _encr_key);
    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;

    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
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

    let read_tenancy = ReadTenancy::new_billing_account(vec![BillingAccountId::from(-1)]);

    let write_tenancy = WriteTenancy::new_billing_account(1.into());
    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = WriteTenancy::new_organization(1.into());
    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = WriteTenancy::new_workspace(1.into());
    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);

    let write_tenancy = WriteTenancy::new_billing_account(1.into()).into_universal();
    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let write_tenancy = WriteTenancy::new_organization(1.into()).into_universal();
    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);

    let write_tenancy = WriteTenancy::new_workspace(1.into()).into_universal();
    let check = write_tenancy
        .check(&txn, &read_tenancy)
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

    let read_tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = WriteTenancy::new_billing_account(1.into());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
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

    let read_tenancy = ReadTenancy::new_billing_account(vec![
        1.into(),
        2.into(),
        3.into(),
        4.into(),
        5.into(),
        6.into(),
    ]);
    let write_tenancy = WriteTenancy::new_billing_account(2.into());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
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

    let read_tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = WriteTenancy::new_billing_account(2.into());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
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

    let read_tenancy = ReadTenancy::new_billing_account(vec![1.into()]);
    let write_tenancy = WriteTenancy::new_organization(1.into());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_organization_pk_identical() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let read_tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba.organization.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
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
    let read_tenancy = ReadTenancy::new_organization(
        &txn,
        vec![
            *nba.organization.id(),
            *nba2.organization.id(),
            *nba3.organization.id(),
        ],
    )
    .await
    .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(*nba2.organization.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_organization_pk_mismatched() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let read_tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set organization read read_tenancy");
    let write_tenancy = WriteTenancy::new_organization(OrganizationId::from(-1));

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn check_workspace_pk_identical() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(*nba.workspace.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
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
    let read_tenancy = ReadTenancy::new_workspace(
        &txn,
        vec![
            *nba.workspace.id(),
            *nba2.workspace.id(),
            *nba3.workspace.id(),
        ],
    )
    .await
    .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(*nba2.workspace.id());

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(check);
}

#[test(tokio::test)]
async fn check_workspace_pk_mismatched() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read read_tenancy");
    let write_tenancy = WriteTenancy::new_workspace(WorkspaceId::from(-1));

    let check = write_tenancy
        .check(&txn, &read_tenancy)
        .await
        .expect("cannot check tenancy");
    assert!(!check);
}

#[test(tokio::test)]
async fn into_tenancy() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    let read_tenancy = ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
        .await
        .expect("unable to set workspace read read_tenancy");
    let mut tenancy = Tenancy::new_workspace(vec![*nba.workspace.id()]);
    tenancy.universal = true;
    tenancy.organization_ids = vec![*nba.organization.id()];
    tenancy.billing_account_ids = vec![*nba.billing_account.id()];
    assert_eq!(Tenancy::from(&read_tenancy), tenancy);

    let read_tenancy = ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
        .await
        .expect("unable to set workspace read read_tenancy");
    let mut tenancy = Tenancy::new_organization(vec![*nba.organization.id()]);
    tenancy.universal = true;
    tenancy.billing_account_ids = vec![*nba.billing_account.id()];
    assert_eq!(Tenancy::from(&read_tenancy), tenancy);

    let read_tenancy = ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]);
    let mut tenancy = Tenancy::new_billing_account(vec![*nba.billing_account.id()]);
    tenancy.universal = true;
    assert_eq!(Tenancy::from(&read_tenancy), tenancy);
}

#[test(tokio::test)]
async fn from_tenancy() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech, _encr_key);

    let (nba, _) = billing_account_signup(&txn, &nats, &secret_key).await;
    assert_eq!(
        ReadTenancy::new_workspace(&txn, vec![*nba.workspace.id()])
            .await
            .expect("unable to generate read tenancy"),
        Tenancy::new_workspace(vec![*nba.workspace.id()])
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to convert to read tenancy")
    );
    assert_eq!(
        ReadTenancy::new_organization(&txn, vec![*nba.organization.id()])
            .await
            .expect("unable to generate read tenancy"),
        Tenancy::new_organization(vec![*nba.organization.id()])
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to convert to read tenancy")
    );
    assert_eq!(
        ReadTenancy::new_billing_account(vec![*nba.billing_account.id()]),
        Tenancy::new_billing_account(vec![*nba.billing_account.id()])
            .clone_into_read_tenancy(&txn)
            .await
            .expect("unable to convert to read tenancy")
    );
}
