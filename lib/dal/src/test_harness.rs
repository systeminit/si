use crate::billing_account::BillingAccountSignup;
use crate::jwt_key::JwtEncrypt;
use crate::node::NodeKind;
use crate::{
    schema, socket, BillingAccount, ChangeSet, Component, EditSession, Group, HistoryActor,
    KeyPair, Node, QualificationCheck, Schema, SchemaKind, StandardModel, Tenancy, User,
    Visibility, NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};
use anyhow::Result;
use async_trait::async_trait;
use lazy_static::lazy_static;
use names::{Generator, Name};
use si_data::{NatsClient, NatsConfig, NatsTxn, PgPool, PgPoolConfig, PgTxn};
use sodiumoxide::crypto::secretbox;
use std::env;
use std::sync::Arc;
use telemetry::{ClientError, TelemetryClient, Verbosity};

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

#[derive(Clone, Copy, Debug)]
pub struct NoopTelemetryClient;

#[async_trait]
impl TelemetryClient for NoopTelemetryClient {
    async fn set_verbosity(&mut self, _updated: Verbosity) -> Result<(), ClientError> {
        Ok(())
    }

    async fn increase_verbosity(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn decrease_verbosity(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn set_custom_tracing(
        &mut self,
        _directives: impl Into<String> + Send + 'async_trait,
    ) -> Result<(), ClientError> {
        Ok(())
    }

    async fn enable_opentelemetry(&mut self) -> Result<(), ClientError> {
        Ok(())
    }

    async fn disable_opentelemetry(&mut self) -> Result<(), ClientError> {
        Ok(())
    }
}

pub struct TestContext {
    // we need to keep this in scope to keep the tempdir from auto-cleaning itself
    #[allow(dead_code)]
    tmp_event_log_fs_root: tempfile::TempDir,
    pub pg: PgPool,
    pub nats_conn: NatsClient,
    pub secret_key: secretbox::Key,
    pub telemetry: NoopTelemetryClient,
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
        let telemetry = NoopTelemetryClient;

        Self {
            tmp_event_log_fs_root,
            pg,
            nats_conn,
            secret_key,
            telemetry,
        }
    }

    pub fn entries(&self) -> (&PgPool, &NatsClient, &secretbox::Key) {
        (&self.pg, &self.nats_conn, &self.secret_key)
    }

    /// Gets a reference to the test context's telemetry.
    pub fn telemetry(&self) -> NoopTelemetryClient {
        self.telemetry
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
        concat!(env!("CARGO_MANIFEST_DIR"), "/", "config/public.pem"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/", "config/private.pem"),
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
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn create_change_set(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    history_actor: &HistoryActor,
) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(txn, nats, tenancy, history_actor, &name, None)
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
        txn,
        nats,
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
    BillingAccount::new(txn, nats, tenancy, visibility, history_actor, &name, None)
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
    BillingAccount::new(txn, nats, tenancy, visibility, history_actor, &name, None)
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
    KeyPair::new(txn, nats, tenancy, visibility, history_actor, &name)
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
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
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
    Group::new(txn, nats, tenancy, visibility, history_actor, &name)
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
        txn,
        nats,
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
        .login(txn, secret_key, nba.billing_account.id(), "snakes")
        .await
        .expect("cannot log in newly created user");
    (nba, auth_token)
}

pub async fn create_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    kind: &SchemaKind,
) -> Schema {
    let name = generate_fake_name();
    Schema::new(txn, nats, tenancy, visibility, history_actor, &name, kind)
        .await
        .expect("cannot create schema")
}

pub async fn create_schema_ui_menu(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> schema::UiMenu {
    schema::UiMenu::new(txn, nats, tenancy, visibility, history_actor)
        .await
        .expect("cannot create schema ui menu")
}

pub async fn create_schema_variant(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> schema::SchemaVariant {
    let name = generate_fake_name();
    schema::SchemaVariant::new(txn, nats, tenancy, visibility, history_actor, name)
        .await
        .expect("cannot create schema variant")
}

pub async fn create_component_and_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> Component {
    let schema = create_schema(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &SchemaKind::Concept,
    )
    .await;
    let schema_variant = create_schema_variant(txn, nats, tenancy, visibility, history_actor).await;
    schema_variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await
        .expect("cannot set schema variant");
    let name = generate_fake_name();
    let entity = Component::new(txn, nats, tenancy, visibility, history_actor, &name)
        .await
        .expect("cannot create entity");
    entity
}

pub async fn create_node(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    node_kind: &NodeKind,
) -> Node {
    let node = Node::new(
        &txn,
        &nats,
        &tenancy,
        &visibility,
        &history_actor,
        node_kind,
    )
    .await
    .expect("cannot create node");
    node
}

pub async fn create_socket(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> socket::Socket {
    let name = generate_fake_name();
    socket::Socket::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        name,
        &socket::SocketEdgeKind::Configures,
        &socket::SocketArity::One,
    )
    .await
    .expect("cannot create socket")
}

pub async fn create_qualification_check(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> QualificationCheck {
    let name = generate_fake_name();
    QualificationCheck::new(txn, nats, tenancy, visibility, history_actor, name)
        .await
        .expect("cannot create qualification check")
}
