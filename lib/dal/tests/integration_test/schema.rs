use crate::test_setup;

use dal::schema::SchemaKind;
use dal::test_harness::{billing_account_signup, create_schema, create_schema_ui_menu};
use dal::{component::ComponentKind, HistoryActor, Schema, StandardModel, Tenancy, Visibility};

pub mod ui_menu;
pub mod variant;

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
    let _schema = Schema::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
}

#[tokio::test]
async fn billing_accounts() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let schema = Schema::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
    schema
        .add_billing_account(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            nba.billing_account.id(),
        )
        .await
        .expect("cannot add billing account");

    let relations = schema
        .billing_accounts(&txn, &visibility)
        .await
        .expect("cannot get billing accounts");
    assert_eq!(relations, vec![nba.billing_account.clone()]);

    schema
        .remove_billing_account(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            nba.billing_account.id(),
        )
        .await
        .expect("cannot remove billing account");
    let relations = schema
        .billing_accounts(&txn, &visibility)
        .await
        .expect("cannot get billing accounts");
    assert_eq!(relations, vec![]);
}

#[tokio::test]
async fn organizations() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let schema = Schema::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
    schema
        .add_organization(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            nba.organization.id(),
        )
        .await
        .expect("cannot add organization");

    let relations = schema
        .organizations(&txn, &visibility)
        .await
        .expect("cannot get organization");
    assert_eq!(relations, vec![nba.organization.clone()]);

    schema
        .remove_organization(
            &txn,
            &nats,
            &visibility,
            &history_actor,
            nba.organization.id(),
        )
        .await
        .expect("cannot remove organization");
    let relations = schema
        .organizations(&txn, &visibility)
        .await
        .expect("cannot get organizations");
    assert_eq!(relations, vec![]);
}

#[tokio::test]
async fn workspaces() {
    test_setup!(ctx, secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let (nba, _token) = billing_account_signup(&txn, &nats, secret_key).await;
    let schema = Schema::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
    schema
        .add_workspace(&txn, &nats, &visibility, &history_actor, nba.workspace.id())
        .await
        .expect("cannot add organization");

    let relations = schema
        .workspaces(&txn, &visibility)
        .await
        .expect("cannot get workspaces");
    assert_eq!(relations, vec![nba.workspace.clone()]);

    schema
        .remove_workspace(&txn, &nats, &visibility, &history_actor, nba.workspace.id())
        .await
        .expect("cannot remove workspace");
    let relations = schema
        .workspaces(&txn, &visibility)
        .await
        .expect("cannot get workspace");
    assert_eq!(relations, vec![]);
}

#[tokio::test]
async fn ui_menus() {
    test_setup!(ctx, _secret_key, pg, conn, txn, nats_conn, nats, _veritech);
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let schema = create_schema(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &SchemaKind::Concrete,
    )
    .await;
    let schema_ui_menu =
        create_schema_ui_menu(&txn, &nats, &tenancy, &visibility, &history_actor).await;
    schema_ui_menu
        .set_schema(&txn, &nats, &visibility, &history_actor, schema.id())
        .await
        .expect("cannot set schema");
    let ui_menus = schema
        .ui_menus(&txn, schema.tenancy(), &visibility)
        .await
        .expect("cannot get ui menus");
    assert_eq!(ui_menus, vec![schema_ui_menu.clone()]);
}
