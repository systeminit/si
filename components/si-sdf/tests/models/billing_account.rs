use names::{Generator, Name};

use crate::{one_time_setup, TestContext};

use si_sdf::data::{NatsTxn, PgTxn};
use si_sdf::models::{BillingAccount, Capability, Group, Organization, PublicKey, User, Workspace};

#[derive(Debug)]
pub struct NewBillingAccount {
    pub billing_account: BillingAccount,
    pub user: User,
    pub group: Group,
    pub workspace: Workspace,
    pub organization: Organization,
    pub public_key: PublicKey,
    pub user_password: String,
}

pub async fn new_billing_account(txn: &PgTxn<'_>, nats: &NatsTxn) -> BillingAccount {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    BillingAccount::new(&txn, &nats, &name, format!("{} description", name))
        .await
        .expect("cannot create billing account")
}

pub async fn signup_new_billing_account(txn: &PgTxn<'_>, nats: &NatsTxn) -> NewBillingAccount {
    let mut generator = Generator::with_naming(Name::Numbered);
    let billing_account_name = generator.next().unwrap();
    let mut name_generator = Generator::default();
    let user_name = name_generator.next().unwrap();
    let user_password = name_generator.next().unwrap();
    let (billing_account, user, group, organization, workspace, public_key) =
        BillingAccount::signup(
            &txn,
            &nats,
            &billing_account_name,
            format!("{} description", billing_account_name),
            &user_name,
            format!("{}@tclown.com", user_name),
            &user_password,
        )
        .await
        .expect("cannot create new billing account");
    NewBillingAccount {
        billing_account,
        user,
        group,
        organization,
        workspace,
        public_key,
        user_password,
    }
}

#[tokio::test]
async fn new() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let ba = BillingAccount::new(&txn, &nats, "af", "adam and fletcher")
        .await
        .expect("cannot create billing account");
    assert_eq!(ba.name, "af");
    assert_eq!(ba.description, "adam and fletcher");
}

#[tokio::test]
async fn signup() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let (billing_account, user, group, organization, workspace, public_key) =
        BillingAccount::signup(
            &txn,
            &nats,
            "goodbye sniper",
            "I shoot, you run",
            "leo",
            "leo@localhost.localdomain",
            "snoopdogg",
        )
        .await
        .expect("cannot signup new billing account");
    assert_eq!(billing_account.name, "goodbye sniper");
    assert_eq!(billing_account.description, "I shoot, you run");
    assert_eq!(user.name, "leo");
    assert_eq!(user.email, "leo@localhost.localdomain");
    assert_eq!(group.name, "administrators");
    assert_eq!(group.user_ids, vec![user.id.clone()]);
    assert_eq!(group.capabilities, vec![Capability::new("any", "any")]);
    assert_eq!(workspace.name, "default");
    assert_eq!(organization.name, "default");
    assert_eq!(public_key.name, billing_account.name);
    assert_eq!(
        public_key.si_storable.billing_account_id,
        billing_account.id
    );
}

#[tokio::test]
async fn get() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    let ba = BillingAccount::get(&txn, &nba.billing_account.id)
        .await
        .expect("cannot get billing account");
    assert_eq!(ba, nba.billing_account);
}

#[tokio::test]
async fn get_by_name() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    let ba = BillingAccount::get_by_name(&txn, &nba.billing_account.name)
        .await
        .expect("cannot get billing account by name");
    assert_eq!(ba, nba.billing_account);
}

#[tokio::test]
async fn rotate_key_pair() {
    one_time_setup().await.expect("one time setup failed");
    let ctx = TestContext::init().await;
    let (pg, nats_conn, _veritech, _event_log_fs, _secret_key) = ctx.entries();
    let nats = nats_conn.transaction();
    let mut conn = pg.pool.get().await.expect("cannot connect to pg");
    let txn = conn.transaction().await.expect("cannot create txn");

    let nba = signup_new_billing_account(&txn, &nats).await;
    BillingAccount::rotate_key_pair(&txn, &nats, nba.billing_account.id)
        .await
        .expect("cannot rotate key pair");
}
