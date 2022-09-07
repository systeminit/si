use std::{env, path::Path, sync::Arc};

use anyhow::Result;
use lazy_static::lazy_static;
use names::{Generator, Name};
use si_data::{NatsClient, NatsConfig, PgPool, PgPoolConfig};

use uuid::Uuid;
use veritech::{EncryptionKey, Instance, StandardConfig};

use crate::{
    billing_account::BillingAccountSignup,
    component::ComponentKind,
    func::{binding::FuncBinding, FuncId},
    job::processor::{sync_processor::SyncProcessor, JobQueueProcessor},
    jwt_key::JwtSecretKey,
    key_pair::KeyPairId,
    node::NodeKind,
    schema,
    socket::{Socket, SocketArity, SocketEdgeKind, SocketKind},
    BillingAccount, BillingAccountId, ChangeSet, Component, DalContext, DiagramKind,
    EncryptedSecret, Func, FuncBackendKind, FuncBackendResponseType, Group, HistoryActor, KeyPair,
    Node, Organization, Prop, PropId, PropKind, QualificationCheck, Schema, SchemaId, SchemaKind,
    SchemaVariantId, Secret, SecretKind, SecretObjectType, StandardModel, System, User, Visibility,
    Workspace, WriteTenancy, NO_CHANGE_SET_PK,
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
    pub job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
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
        let job_processor =
            Box::new(SyncProcessor::new()) as Box<dyn JobQueueProcessor + Send + Sync>;

        // Create a dedicated Veritech server with a unique subject prefix for each test
        let nats_subject_prefix = nats_prefix();
        let veritech_server =
            veritech_server_for_uds_cyclone(settings.nats.clone(), nats_subject_prefix.clone())
                .await;
        tokio::spawn(veritech_server.run());
        let veritech =
            veritech::Client::with_subject_prefix(nats_conn.clone(), nats_subject_prefix);
        let encryption_key = EncryptionKey::load(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../cyclone-server/src/dev.encryption.key"),
        )
        .await
        .expect("failed to load dev encryption key");
        let secret_key = settings.jwt_encrypt.clone();
        let telemetry = telemetry::NoopClient;

        Self {
            tmp_event_log_fs_root,
            pg,
            nats_conn,
            job_processor,
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
        Box<dyn JobQueueProcessor + Send + Sync>,
        veritech::Client,
        &EncryptionKey,
        &JwtSecretKey,
    ) {
        (
            &self.pg,
            &self.nats_conn,
            self.job_processor.clone(),
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
                dir.join("../../lib/cyclone-server/src/dev.decryption.key")
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
    Uuid::new_v4().as_simple().to_string()
}

pub async fn one_time_setup() -> Result<()> {
    let _ = crate::test::TestContext::global().await;
    Ok(())
}

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn create_change_set(ctx: &DalContext) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(ctx, &name, None)
        .await
        .expect("cannot create change_set")
}

pub fn create_visibility_change_set(change_set: &ChangeSet) -> Visibility {
    Visibility::new(change_set.pk, None)
}

pub fn create_visibility_head() -> Visibility {
    Visibility::new(NO_CHANGE_SET_PK, None)
}

pub async fn create_billing_account_with_name(
    ctx: &DalContext,
    name: impl AsRef<str>,
) -> BillingAccount {
    BillingAccount::new(ctx, &name, None)
        .await
        .expect("cannot create billing_account")
}

pub async fn create_billing_account(ctx: &DalContext) -> BillingAccount {
    let name = generate_fake_name();
    create_billing_account_with_name(ctx, name).await
}

pub async fn create_organization(ctx: &DalContext) -> Organization {
    let name = generate_fake_name();
    Organization::new(ctx, &name)
        .await
        .expect("cannot create organization")
}

pub async fn create_workspace(ctx: &DalContext) -> Workspace {
    let name = generate_fake_name();
    Workspace::new(ctx, &name)
        .await
        .expect("cannot create workspace")
}

pub async fn create_key_pair(ctx: &DalContext) -> KeyPair {
    let name = generate_fake_name();
    KeyPair::new(ctx, &name)
        .await
        .expect("cannot create key_pair")
}

pub async fn create_user(ctx: &DalContext) -> User {
    let name = generate_fake_name();
    User::new(
        ctx,
        &name,
        &format!("{}@test.systeminit.com", name),
        "liesAreTold",
    )
    .await
    .expect("cannot create user")
}

pub async fn create_group(ctx: &DalContext) -> Group {
    let name = generate_fake_name();
    Group::new(ctx, &name).await.expect("cannot create group")
}

pub async fn billing_account_signup(
    ctx: &DalContext,
    jwt_secret_key: &JwtSecretKey,
) -> (BillingAccountSignup, String) {
    let _write_tenancy = WriteTenancy::new_universal();
    let _visibility = Visibility::new_head(false);
    let _history_actor = HistoryActor::SystemInit;
    let billing_account_name = generate_fake_name();
    let user_name = format!("frank {}", billing_account_name);
    let user_email = format!("{}@example.com", billing_account_name);
    let user_password = "snakes";

    let nba = BillingAccount::signup(
        ctx,
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await
    .expect("cannot signup a new billing_account");
    let auth_token = nba
        .user
        .login(ctx, jwt_secret_key, nba.billing_account.id(), "snakes")
        .await
        .expect("cannot log in newly created user");
    (nba, auth_token)
}

pub async fn create_schema(ctx: &DalContext, kind: &SchemaKind) -> Schema {
    let name = generate_fake_name();
    Schema::new(ctx, &name, kind, &ComponentKind::Standard)
        .await
        .expect("cannot create schema")
}

pub async fn create_schema_ui_menu(ctx: &DalContext) -> schema::UiMenu {
    schema::UiMenu::new(ctx, &DiagramKind::Configuration)
        .await
        .expect("cannot create schema ui menu")
}

pub async fn create_schema_variant(ctx: &DalContext, schema_id: SchemaId) -> schema::SchemaVariant {
    create_schema_variant_with_root(ctx, schema_id).await.0
}

pub async fn create_schema_variant_with_root(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> (schema::SchemaVariant, schema::RootProp) {
    let name = generate_fake_name();
    let (variant, root) = schema::SchemaVariant::new(ctx, schema_id, name)
        .await
        .expect("cannot create schema variant");

    let schema = variant
        .schema(ctx)
        .await
        .expect("cannot find schema")
        .expect("schema not found");
    let diagram_kind = schema
        .diagram_kind()
        .expect("no diagram kind for schema kind");

    let input_socket = Socket::new(
        ctx,
        "input",
        SocketKind::Provider,
        &SocketEdgeKind::ConfigurationInput,
        &SocketArity::Many,
        &diagram_kind,
    )
    .await
    .expect("Unable to create socket");
    variant
        .add_socket(ctx, input_socket.id())
        .await
        .expect("Unable to add socket to variant");

    let output_socket = Socket::new(
        ctx,
        "output",
        SocketKind::Provider,
        &SocketEdgeKind::ConfigurationOutput,
        &SocketArity::Many,
        &diagram_kind,
    )
    .await
    .expect("Unable to create socket");
    variant
        .add_socket(ctx, output_socket.id())
        .await
        .expect("Unable to add socket to variant");

    let system_socket = Socket::new(
        ctx,
        "system",
        SocketKind::Provider,
        &SocketEdgeKind::System,
        &SocketArity::Many,
        &DiagramKind::Configuration,
    )
    .await
    .expect("unable to create socket");
    variant
        .add_socket(ctx, system_socket.id())
        .await
        .expect("Unable to add socket to variant");

    (variant, root)
}

pub async fn create_component_and_schema(ctx: &DalContext) -> Component {
    let schema = create_schema(ctx, &SchemaKind::Configuration).await;
    let schema_variant = create_schema_variant(ctx, *schema.id()).await;
    schema_variant
        .finalize(ctx)
        .await
        .expect("unable to finalize schema variant");
    let name = generate_fake_name();
    let (entity, _) = Component::new_for_schema_variant_with_node(ctx, &name, schema_variant.id())
        .await
        .expect("cannot create component");
    entity
}

#[allow(clippy::too_many_arguments)]
pub async fn create_component_for_schema_variant(
    ctx: &DalContext,
    schema_variant_id: &SchemaVariantId,
) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_schema_variant_with_node(ctx, &name, schema_variant_id)
        .await
        .expect("cannot create component");
    component
}

#[allow(clippy::too_many_arguments)]
pub async fn create_component_for_schema(ctx: &DalContext, schema_id: &SchemaId) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_schema_with_node(ctx, &name, schema_id)
        .await
        .expect("cannot create component");
    component
}

pub async fn create_node(ctx: &DalContext, node_kind: &NodeKind) -> Node {
    Node::new(ctx, node_kind).await.expect("cannot create node")
}

pub async fn create_qualification_check(ctx: &DalContext) -> QualificationCheck {
    let name = generate_fake_name();
    QualificationCheck::new(ctx, name)
        .await
        .expect("cannot create qualification check")
}

pub async fn create_system(ctx: &DalContext) -> System {
    let name = generate_fake_name();
    System::new(ctx, name).await.expect("cannot create system")
}

pub async fn create_prop(ctx: &DalContext) -> Prop {
    let name = generate_fake_name();
    Prop::new(ctx, name, PropKind::String)
        .await
        .expect("cannot create prop")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_prop_of_kind(ctx: &DalContext, prop_kind: PropKind) -> Prop {
    let name = generate_fake_name();
    Prop::new(ctx, name, prop_kind)
        .await
        .expect("cannot create prop")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_prop_of_kind_with_name(
    ctx: &DalContext,
    prop_kind: PropKind,
    name: impl AsRef<str>,
) -> Prop {
    let name = name.as_ref();
    Prop::new(ctx, name, prop_kind)
        .await
        .expect("cannot create prop")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_prop_of_kind_and_set_parent_with_name(
    ctx: &DalContext,
    prop_kind: PropKind,
    name: impl AsRef<str>,
    parent_prop_id: PropId,
) -> Prop {
    let name = name.as_ref();
    let new_prop = Prop::new(ctx, name, prop_kind)
        .await
        .expect("cannot create prop");
    new_prop
        .set_parent_prop(ctx, parent_prop_id)
        .await
        .expect("cannot set parent to new prop");
    new_prop
}

pub async fn create_func(ctx: &DalContext) -> Func {
    let name = generate_fake_name();
    Func::new(
        ctx,
        name,
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func")
}

#[allow(clippy::too_many_arguments)]
pub async fn create_func_binding(
    ctx: &DalContext,
    args: serde_json::Value,
    func_id: FuncId,
    backend_kind: FuncBackendKind,
) -> FuncBinding {
    FuncBinding::new(ctx, args, func_id, backend_kind)
        .await
        .expect("cannot create func")
}

pub async fn encrypt_message(
    ctx: &DalContext,
    key_pair_id: KeyPairId,
    message: &serde_json::Value,
) -> Vec<u8> {
    let public_key = KeyPair::get_by_id(ctx, &key_pair_id)
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
    ctx: &DalContext,
    key_pair_id: KeyPairId,
    billing_account_id: BillingAccountId,
) -> Secret {
    let name = generate_fake_name();
    EncryptedSecret::new(
        ctx,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &encrypt_message(ctx, key_pair_id, &serde_json::json!({ "name": name })).await,
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
    ctx: &DalContext,
    key_pair_id: KeyPairId,
    message: &serde_json::Value,
    billing_account_id: BillingAccountId,
) -> Secret {
    let name = generate_fake_name();
    EncryptedSecret::new(
        ctx,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &encrypt_message(ctx, key_pair_id, message).await,
        key_pair_id,
        Default::default(),
        Default::default(),
        billing_account_id,
    )
    .await
    .expect("cannot create secret")
}
