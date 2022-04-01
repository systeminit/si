use dal::{BillingAccountSignup, DalContext, JwtSecretKey};

use crate::dal::test;
use dal::schema::SchemaKind;
use dal::test_harness::{billing_account_signup, create_schema, create_schema_ui_menu};
use dal::{component::ComponentKind, Schema, StandardModel};

pub mod ui_menu;
pub mod variant;

#[test]
async fn new(ctx: &DalContext<'_, '_>) {
    let _schema = Schema::new(
        ctx,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
}

#[test]
async fn billing_accounts(ctx: &DalContext<'_, '_>, jwt_secret_key: &JwtSecretKey) {
    let (nba, _token) = billing_account_signup(ctx, jwt_secret_key).await;
    let schema = Schema::new(
        ctx,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
    schema
        .add_billing_account(ctx, nba.billing_account.id())
        .await
        .expect("cannot add billing account");

    let relations = schema
        .billing_accounts(ctx)
        .await
        .expect("cannot get billing accounts");
    assert_eq!(relations, vec![nba.billing_account.clone()]);

    schema
        .remove_billing_account(ctx, nba.billing_account.id())
        .await
        .expect("cannot remove billing account");
    let relations = schema
        .billing_accounts(ctx)
        .await
        .expect("cannot get billing accounts");
    assert_eq!(relations, vec![]);
}

#[test]
async fn organizations(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let schema = Schema::new(
        ctx,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
    schema
        .add_organization(ctx, nba.organization.id())
        .await
        .expect("cannot add organization");

    let relations = schema
        .organizations(ctx)
        .await
        .expect("cannot get organization");
    assert_eq!(relations, vec![nba.organization.clone()]);

    schema
        .remove_organization(ctx, nba.organization.id())
        .await
        .expect("cannot remove organization");
    let relations = schema
        .organizations(ctx)
        .await
        .expect("cannot get organizations");
    assert_eq!(relations, vec![]);
}

#[test]
async fn workspaces(ctx: &DalContext<'_, '_>, nba: &BillingAccountSignup) {
    let schema = Schema::new(
        ctx,
        "mastodon",
        &SchemaKind::Concrete,
        &ComponentKind::Standard,
    )
    .await
    .expect("cannot create schema");
    schema
        .add_workspace(ctx, nba.workspace.id())
        .await
        .expect("cannot add organization");

    let relations = schema.workspaces(ctx).await.expect("cannot get workspaces");
    assert_eq!(relations, vec![nba.workspace.clone()]);

    schema
        .remove_workspace(ctx, nba.workspace.id())
        .await
        .expect("cannot remove workspace");
    let relations = schema.workspaces(ctx).await.expect("cannot get workspace");
    assert_eq!(relations, vec![]);
}

#[test]
async fn ui_menus(ctx: &DalContext<'_, '_>) {
    let schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let schema_ui_menu = create_schema_ui_menu(ctx).await;
    schema_ui_menu
        .set_schema(ctx, schema.id())
        .await
        .expect("cannot set schema");
    let ui_menus = schema.ui_menus(ctx).await.expect("cannot get ui menus");
    assert_eq!(ui_menus, vec![schema_ui_menu.clone()]);
}
