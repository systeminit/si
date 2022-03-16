use std::{env, path::Path, sync::Arc};

use anyhow::Result;
use lazy_static::lazy_static;
use names::{Generator, Name};
use si_data::{NatsClient, NatsConfig, NatsTxn, PgPool, PgPoolConfig, PgTxn};
use telemetry::prelude::*;
use uuid::Uuid;
use veritech::{EncryptionKey, Instance, StandardConfig};

use crate::{
    billing_account::BillingAccountSignup,
    component::ComponentKind,
    func::{binding::FuncBinding, FuncId},
    jwt_key::JwtSecretKey,
    key_pair::KeyPairId,
    node::NodeKind,
    schema, socket,
    socket::{Socket, SocketArity, SocketEdgeKind},
    BillingAccount, BillingAccountId, ChangeSet, Component, EditSession, EncryptedSecret, Func,
    FuncBackendKind, FuncBackendResponseType, Group, HistoryActor, KeyPair, Node, Organization,
    Prop, PropKind, QualificationCheck, Schema, SchemaId, SchemaKind, SchemaVariantId, Secret,
    SecretKind, SecretObjectType, StandardModel, System, Tenancy, User, Visibility, Workspace,
    NO_CHANGE_SET_PK, NO_EDIT_SESSION_PK,
};

#[derive(Debug)]
pub struct TestConfig {
    pg: PgPoolConfig,
    nats: NatsConfig,
    jwt_encrypt: JwtSecretKey,
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
        pg.dbname = env::var("SI_TEST_PG_DBNAME").unwrap_or_else(|_| "si_test".to_string());

        Self {
            pg,
            nats,
            jwt_encrypt: JwtSecretKey::default(),
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
    pub veritech: veritech::Client,
    pub encryption_key: EncryptionKey,
    pub jwt_secret_key: JwtSecretKey,
    pub telemetry: telemetry::NoopClient,
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
        // Create a dedicated Veritech server with a unique subject prefix for each test
        let nats_subject_prefix = nats_prefix();
        let veritech_server =
            veritech_server_for_uds_cyclone(settings.nats.clone(), nats_subject_prefix.clone())
                .await;
        tokio::spawn(veritech_server.run());
        let veritech =
            veritech::Client::with_subject_prefix(nats_conn.clone(), nats_subject_prefix);
        let encryption_key = EncryptionKey::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../cyclone/src/dev.encryption.key"),
        )
        .await
        .expect("failed to load dev encryption key");
        let secret_key = settings.jwt_encrypt.clone();
        let telemetry = telemetry::NoopClient;

        Self {
            tmp_event_log_fs_root,
            pg,
            nats_conn,
            veritech,
            encryption_key,
            jwt_secret_key: secret_key,
            telemetry,
        }
    }

    pub fn entries(
        &self,
    ) -> (
        &PgPool,
        &NatsClient,
        veritech::Client,
        &EncryptionKey,
        &JwtSecretKey,
    ) {
        (
            &self.pg,
            &self.nats_conn,
            self.veritech.clone(),
            &self.encryption_key,
            &self.jwt_secret_key,
        )
    }

    /// Gets a reference to the test context's telemetry.
    pub fn telemetry(&self) -> telemetry::NoopClient {
        self.telemetry
    }
}

async fn veritech_server_for_uds_cyclone(
    nats_config: NatsConfig,
    subject_prefix: String,
) -> veritech::Server {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cyclone_spec = veritech::CycloneSpec::LocalUds(
        veritech::LocalUdsInstance::spec()
            .try_cyclone_cmd_path(
                dir.join("../../target/debug/cyclone")
                    .canonicalize()
                    .expect("failed to canonicalize cyclone bin path")
                    .to_string_lossy()
                    .to_string(),
            )
            .expect("failed to setup cyclone_cmd_path")
            .cyclone_decryption_key_path(
                dir.join("../../lib/cyclone/src/dev.decryption.key")
                    .canonicalize()
                    .expect("failed to canonicalize cyclone decryption key path")
                    .to_string_lossy()
                    .to_string(),
            )
            .try_lang_server_cmd_path(
                dir.join("../../bin/lang-js/target/lang-js")
                    .canonicalize()
                    .expect("failed to canonicalize lang-js path")
                    .to_string_lossy()
                    .to_string(),
            )
            .expect("failed to setup lang_js_cmd_path")
            .qualification()
            .resolver()
            .sync()
            .code_generation()
            .build()
            .expect("failed to build cyclone spec"),
    );
    let config = veritech::Config::builder()
        .nats(nats_config)
        .subject_prefix(subject_prefix)
        .cyclone_spec(cyclone_spec)
        .build()
        .expect("failed to build spec");
    veritech::Server::for_cyclone_uds(config)
        .await
        .expect("failed to create server")
}

fn nats_prefix() -> String {
    Uuid::new_v4().to_simple().to_string()
}

pub async fn one_time_setup() -> Result<()> {
    let mut finished = INIT_PG_LOCK.lock().await;
    if *finished {
        return Ok(());
    }

    sodiumoxide::init().expect("crypto failed to init");
    info!("Initializing tests");

    // The stack seems to get too deep here, so we create a new one just for the migrations
    tokio::task::spawn(async {
        let encryption_key = EncryptionKey::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../cyclone/src/dev.encryption.key"),
        )
        .await
        .expect("failed to load dev encryption key");

        let nats_conn = NatsClient::new(&SETTINGS.nats)
            .await
            .expect("failed to connect to NATS");

        // Start up a Veritech server as a task exclusively to allow the migrations to run
        let nats_subject_prefix = nats_prefix();
        let veritech_server =
            veritech_server_for_uds_cyclone(SETTINGS.nats.clone(), nats_subject_prefix.clone())
                .await;
        let veritech_server_handle = veritech_server.shutdown_handle();
        tokio::spawn(veritech_server.run());
        let veritech =
            veritech::Client::with_subject_prefix(nats_conn.clone(), nats_subject_prefix);
        let pg = PgPool::new(&SETTINGS.pg)
            .await
            .expect("failed to connect to postgres");
        pg.drop_and_create_public_schema()
            .await
            .expect("failed to drop the database");

        crate::migrate(&pg).await.expect("migration failed!");

        let mut conn = pg.get().await.expect("Unable to get pg connection");
        let txn = conn
            .transaction()
            .await
            .expect("Unable to start pg transaction");

        crate::create_jwt_key_if_missing(
            &txn,
            concat!(env!("CARGO_MANIFEST_DIR"), "/", "config/public.pem"),
            concat!(env!("CARGO_MANIFEST_DIR"), "/", "config/private.pem"),
            &SETTINGS.jwt_encrypt.key,
        )
        .await
        .expect("Unable to initialize jwt if missing");
        txn.commit().await.expect("Unable to commit transaction");

        crate::migrate_builtin_schemas(&pg, &nats_conn, veritech, &encryption_key)
            .await
            .expect("Failed to migrate builtins");

        let visibility = Visibility::new_head(false);
        let history_actor = HistoryActor::SystemInit;
        let tenancy = Tenancy::new_universal();
        let txn = conn
            .transaction()
            .await
            .expect("Unable to start pg transaction");
        let nats = nats_conn.transaction();
        let _ =
            find_or_create_production_system(&txn, &nats, &tenancy, &visibility, &history_actor)
                .await;
        txn.commit().await.expect("Unable to commit transaction");

        // Shutdown the Veritech server (each test gets their own server instance with an exclusively
        // unique subject prefix)
        veritech_server_handle.shutdown().await;
    })
    .await
    .expect("Postgres initialization failed");

    info!("Initialized tests");
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

pub async fn create_organization(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> Organization {
    let name = generate_fake_name();
    Organization::new(txn, nats, &tenancy.into(), visibility, history_actor, &name)
        .await
        .expect("cannot create organization")
}

pub async fn create_workspace(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> Workspace {
    let name = generate_fake_name();
    Workspace::new(txn, nats, tenancy, visibility, history_actor, &name)
        .await
        .expect("cannot create workspace")
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
    jwt_secret_key: &JwtSecretKey,
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
        .login(txn, jwt_secret_key, nba.billing_account.id(), "snakes")
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
    Schema::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &name,
        kind,
        &ComponentKind::Standard,
    )
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

#[allow(clippy::too_many_arguments)]
pub async fn create_schema_variant(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> schema::SchemaVariant {
    create_schema_variant_with_root(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        veritech,
        encryption_key,
    )
    .await
    .0
}

#[allow(clippy::too_many_arguments)]
pub async fn create_schema_variant_with_root(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
) -> (schema::SchemaVariant, schema::builtins::RootProp) {
    let name = generate_fake_name();
    let (variant, root) = schema::SchemaVariant::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        name,
        veritech,
        encryption_key,
    )
    .await
    .expect("cannot create schema variant");

    let input_socket = Socket::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "input",
        &SocketEdgeKind::Configures,
        &SocketArity::Many,
    )
    .await
    .expect("Unable to create socket");
    variant
        .add_socket(txn, nats, visibility, history_actor, input_socket.id())
        .await
        .expect("Unable to add socket to variant");

    let output_socket = Socket::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "output",
        &SocketEdgeKind::Output,
        &SocketArity::Many,
    )
    .await
    .expect("Unable to create socket");
    variant
        .add_socket(txn, nats, visibility, history_actor, output_socket.id())
        .await
        .expect("Unable to add socket to variant");

    let includes_socket = Socket::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        "includes",
        &SocketEdgeKind::Includes,
        &SocketArity::Many,
    )
    .await
    .expect("Unable to create socket");
    variant
        .add_socket(txn, nats, visibility, history_actor, includes_socket.id())
        .await
        .expect("Unable to add socket to variant");
    (variant, root)
}

pub async fn create_component_and_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
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
    let schema_variant = create_schema_variant(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        veritech.clone(),
        encryption_key,
    )
    .await;
    schema_variant
        .set_schema(txn, nats, visibility, history_actor, schema.id())
        .await
        .expect("cannot set schema variant");
    let name = generate_fake_name();
    let (entity, _) = Component::new_for_schema_variant_with_node(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        &name,
        schema_variant.id(),
    )
    .await
    .expect("cannot create entity");
    entity
}

#[allow(clippy::too_many_arguments)]
pub async fn create_component_for_schema_variant(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema_variant_id: &SchemaVariantId,
) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_schema_variant_with_node(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        &name,
        schema_variant_id,
    )
    .await
    .expect("cannot create component");
    component
}

#[allow(clippy::too_many_arguments)]
pub async fn create_component_for_schema(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    schema_id: &SchemaId,
) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_schema_with_node(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        &name,
        schema_id,
    )
    .await
    .expect("cannot create component");
    component
        .set_schema(txn, nats, visibility, history_actor, schema_id)
        .await
        .expect("cannot set the schema for our component");
    component
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
        txn,
        nats,
        &tenancy.into(),
        visibility,
        history_actor,
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

pub async fn create_system(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> System {
    let name = generate_fake_name();
    System::new(txn, nats, tenancy, visibility, history_actor, name)
        .await
        .expect("cannot create system")
}

pub async fn find_or_create_production_system(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> System {
    let name = "production".to_string();

    match System::find_by_attr(txn, tenancy, visibility, "name", &name)
        .await
        .expect("cannot find system")
        .pop()
    {
        Some(s) => s,
        None => {
            let (system, _system_node) =
                System::new_with_node(txn, nats, tenancy, visibility, history_actor, name)
                    .await
                    .expect("cannot create named system");

            system
        }
    }
}

pub async fn create_prop(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> Prop {
    let name = generate_fake_name();
    Prop::new(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        name,
        PropKind::String,
    )
    .await
    .expect("cannot create prop")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_prop_of_kind(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    prop_kind: PropKind,
) -> Prop {
    let name = generate_fake_name();
    Prop::new(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        name,
        prop_kind,
    )
    .await
    .expect("cannot create prop")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_prop_of_kind_with_name(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    veritech: veritech::Client,
    encryption_key: &EncryptionKey,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    prop_kind: PropKind,
    name: impl AsRef<str>,
) -> Prop {
    let name = name.as_ref();
    Prop::new(
        txn,
        nats,
        veritech,
        encryption_key,
        tenancy,
        visibility,
        history_actor,
        name,
        prop_kind,
    )
    .await
    .expect("cannot create prop")
}

pub async fn create_func(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
) -> Func {
    let name = generate_fake_name();
    Func::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        name,
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_func_binding(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    args: serde_json::Value,
    func_id: FuncId,
    backend_kind: FuncBackendKind,
) -> FuncBinding {
    FuncBinding::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        args,
        func_id,
        backend_kind,
    )
    .await
    .expect("cannot create func")
}

pub async fn encrypt_message(
    txn: &PgTxn<'_>,
    tenancy: &Tenancy,
    visibility: &Visibility,
    key_pair_id: KeyPairId,
    message: &serde_json::Value,
) -> Vec<u8> {
    let public_key = KeyPair::get_by_id(txn, tenancy, visibility, &key_pair_id)
        .await
        .expect("failed to fetch key pair")
        .expect("failed to find key pair");

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(message).expect("failed to serialize message"),
        public_key.public_key(),
    );
    crypted
}

pub async fn create_secret(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    key_pair_id: KeyPairId,
    billing_account_id: BillingAccountId,
) -> Secret {
    let name = generate_fake_name();
    EncryptedSecret::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &encrypt_message(
            txn,
            tenancy,
            visibility,
            key_pair_id,
            &serde_json::json!({ "name": name }),
        )
        .await,
        key_pair_id,
        Default::default(),
        Default::default(),
        billing_account_id,
    )
    .await
    .expect("cannot create secret")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_secret_with_message(
    txn: &PgTxn<'_>,
    nats: &NatsTxn,
    tenancy: &Tenancy,
    visibility: &Visibility,
    history_actor: &HistoryActor,
    key_pair_id: KeyPairId,
    message: &serde_json::Value,
    billing_account_id: BillingAccountId,
) -> Secret {
    let name = generate_fake_name();
    EncryptedSecret::new(
        txn,
        nats,
        tenancy,
        visibility,
        history_actor,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &encrypt_message(txn, tenancy, visibility, key_pair_id, message).await,
        key_pair_id,
        Default::default(),
        Default::default(),
        billing_account_id,
    )
    .await
    .expect("cannot create secret")
}
