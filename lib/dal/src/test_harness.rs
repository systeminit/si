use crate::billing_account::BillingAccountSignup;
use crate::jwt_key::JwtEncrypt;
use crate::{
    BillingAccount, ChangeSet, EditSession, Group, HistoryActor, KeyPair, StandardModel, Tenancy,
    User, Visibility, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};
use anyhow::Result;
use lazy_static::lazy_static;
use names::{Generator, Name};
use si_data::{NatsClient, NatsConfig, NatsTxn, PgPool, PgPoolConfig, PgTxn};
use sodiumoxide::crypto::secretbox;
use std::env;
use std::sync::Arc;

#[derive(Debug)]
pub struct TestConfig {
    pg: PgPoolConfig,
    nats: NatsConfig,
    jwt_encrypt: JwtEncrypt,
}

impl Default for TestConfig {
    fn default() -> Self {
        let mut nats = NatsConfig::default();
        if let Ok(value) = env::var("SI_TEST_NATS_URL") {
            nats.url = value;
        }
        let mut pg = PgPoolConfig::default();
        if let Ok(value) = env::var("SI_TEST_PG_HOSTNAME") {
            pg.hostname = value;
        }

        Self {
            pg,
            nats,
            jwt_encrypt: JwtEncrypt::default(),
        }
    }
}

lazy_static! {
    pub static ref SETTINGS: TestConfig = TestConfig::default();
    pub static ref INIT_LOCK: Arc<tokio::sync::Mutex<bool>> =
        Arc::new(tokio::sync::Mutex::new(false));
    pub static ref INIT_PG_LOCK: Arc<tokio::sync::Mutex<bool>> =
        Arc::new(tokio::sync::Mutex::new(false));
}

pub struct TestContext {
    // we need to keep this in scope to keep the tempdir from auto-cleaning itself
    #[allow(dead_code)]
    tmp_event_log_fs_root: tempfile::TempDir,
    pub pg: PgPool,
    pub nats_conn: NatsClient,
    pub secret_key: secretbox::Key,
}

impl TestContext {
    pub async fn init() -> Self {
        Self::init_with_settings(&SETTINGS).await
    }

    pub async fn init_with_settings(settings: &TestConfig) -> Self {
        let tmp_event_log_fs_root = tempfile::tempdir().expect("could not create temp dir");
        let pg = PgPool::new(&settings.pg)
            .await
            .expect("failed to connect to postgres");
        let nats_conn = NatsClient::new(&settings.nats)
            .await
            .expect("failed to connect to NATS");
        let secret_key = settings.jwt_encrypt.key.clone();

        Self {
            tmp_event_log_fs_root,
            pg,
            nats_conn,
            secret_key,
        }
    }

    pub fn entries(&self) -> (&PgPool, &NatsClient, &secretbox::Key) {
        (&self.pg, &self.nats_conn, &self.secret_key)
    }
}

pub async fn one_time_setup() -> Result<()> {
    let mut finished = INIT_PG_LOCK.lock().await;
    if *finished {
        return Ok(());
    }

    sodiumoxide::init().expect("crypto failed to init");

    let pg = PgPool::new(&SETTINGS.pg)
        .await
        .expect("failed to connect to postgres");
    pg.drop_and_create_public_schema()
        .await
        .expect("failed to drop the database");
    crate::migrate(&pg).await.expect("migration failed!");

    let mut conn = pg.get().await?;
    let txn = conn.transaction().await?;

    crate::create_jwt_key_if_missing(
        &txn,
        "config/public.pem",
        "config/private.pem",
        &SETTINGS.jwt_encrypt.key,
    )
    .await?;
    txn.commit().await?;

    let nats_conn = NatsClient::new(&SETTINGS.nats)
        .await
        .expect("failed to connect to NATS");
    crate::migrate_builtin_schemas(&pg, &nats_conn).await?;

    *finished = true;
    Ok(())
}

pub fn generate_fake_name() -> String {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    return name;
}

pub async fn create_change_set(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    history_actor: &HistoryActor,
) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(&txn, &nats, &tenancy, &history_actor, &name, None)
        .await
        .expect("cannot create change_set")
}

pub async fn create_edit_session(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    history_actor: &HistoryActor,
    change_set: &ChangeSet,
) -> EditSession {
    let name = generate_fake_name();
    EditSession::new(
        &txn,
        &nats,
        &change_set.tenancy,
        history_actor,
        &change_set.pk,
        &name,
        None,
    )
    .await
    .expect("cannot create edit_session")
}

pub fn create_visibility_edit_session(
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> Visibility {
    Visibility::new(change_set.pk, edit_session.pk, false)
}

pub fn create_visibility_change_set(change_set: &ChangeSet) -> Visibility {
    Visibility::new(change_set.pk, NO_EDIT_SESSION_PK, false)
}

pub fn create_visibility_head() -> Visibility {
    Visibility::new(NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK, false)
}

pub async fn create_billing_account_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    name: impl AsRef<str>,
) -> BillingAccount {
    BillingAccount::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &name,
        None,
    )
    .await
    .expect("cannot create billing_account")
}

pub async fn create_billing_account(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> BillingAccount {
    let name = generate_fake_name();
    BillingAccount::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &name,
        None,
    )
    .await
    .expect("cannot create billing_account")
}

pub async fn create_key_pair(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> KeyPair {
    let name = generate_fake_name();
    KeyPair::new(&txn, &nats, &tenancy, &visibility, &history_actor, &name)
        .await
        .expect("cannot create key_pair")
}

pub async fn create_user(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> User {
    let name = generate_fake_name();
    User::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &name,
        &format!("{}@test.systeminit.com", name),
        "liesAreTold",
    )
    .await
    .expect("cannot create user")
}

pub async fn create_group(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> Group {
    let name = generate_fake_name();
    Group::new(&txn, &nats, &tenancy, &visibility, &history_actor, &name)
        .await
        .expect("cannot create group")
}

pub async fn billing_account_signup(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    secret_key: &secretbox::Key,
) -> (BillingAccountSignup, String) {
    let tenancy = Tenancy::new_universal();
    let visibility = Visibility::new_head(false);
    let history_actor = HistoryActor::SystemInit;
    let billing_account_name = generate_fake_name();
    let user_name = format!("frank {}", billing_account_name);
    let user_email = format!("{}@example.com", billing_account_name);
    let user_password = "snakes";

    let nba = BillingAccount::signup(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await
    .expect("cannot signup a new billing_account");
    let auth_token = nba
        .user
        .login(&txn, &secret_key, &nba.billing_account.id(), "snakes")
        .await
        .expect("cannot log in newly created user");
    (nba, auth_token)
}
