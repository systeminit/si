//! This module provides helpers and resources for constructing and running integration tests.

use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
    sync::{Arc, Once},
};

use buck2_resources::Buck2Resources;
use content_store::PgStoreTools;
use dal::{
    job::processor::{JobQueueProcessor, NatsProcessor},
    DalContext, JwtPublicSigningKey, ModelResult, ServicesContext, Tenancy, Workspace,
};
use derive_builder::Builder;
use jwt_simple::prelude::RS256KeyPair;
use lazy_static::lazy_static;
use rebaser_client::Config as RebaserClientConfig;
use si_crypto::{
    SymmetricCryptoService, SymmetricCryptoServiceConfig, SymmetricCryptoServiceConfigFile,
};
use si_data_nats::{NatsClient, NatsConfig};
use si_data_pg::{PgPool, PgPoolConfig};
use si_std::ResultExt;
use telemetry::prelude::*;
use tokio::{fs::File, io::AsyncReadExt, sync::Mutex};
use uuid::Uuid;
use veritech_client::CycloneEncryptionKey;
use veritech_server::StandardConfig;

pub use color_eyre::{
    self,
    eyre::{eyre, Result, WrapErr},
};
use dal::builtins::{func, schema};
pub use si_test_macros::{dal_test as test, sdf_test};
pub use signup::WorkspaceSignup;
pub use telemetry;
pub use tracing_subscriber;

pub mod helpers;
mod schemas;
mod signup;
pub mod test_harness;

const DEFAULT_TEST_PG_USER: &str = "si_test";
const DEFAULT_TEST_PG_PORT_STR: &str = "6432";

const ENV_VAR_NATS_URL: &str = "SI_TEST_NATS_URL";
const ENV_VAR_MODULE_INDEX_URL: &str = "SI_TEST_MODULE_INDEX_URL";
const ENV_VAR_PG_HOSTNAME: &str = "SI_TEST_PG_HOSTNAME";
const ENV_VAR_PG_DBNAME: &str = "SI_TEST_PG_DBNAME";
const ENV_VAR_CONTENT_STORE_PG_DBNAME: &str = "SI_TEST_CONTENT_STORE_PG_DBNAME";
const ENV_VAR_PG_USER: &str = "SI_TEST_PG_USER";
const ENV_VAR_PG_PORT: &str = "SI_TEST_PG_PORT";
const ENV_VAR_KEEP_OLD_DBS: &str = "SI_TEST_KEEP_OLD_DBS";
const SI_AWS_EC2_PKG: &str = "si-aws-ec2-2023-09-26.sipkg";
const SI_DOCKER_IMAGE_PKG: &str = "si-docker-image-2023-09-13.sipkg";
const SI_COREOS_PKG: &str = "si-coreos-2023-09-13.sipkg";

pub static COLOR_EYRE_INIT: Once = Once::new();

lazy_static! {
    static ref TEST_CONTEXT_BUILDER: Mutex<ContextBuilderState> = Mutex::new(Default::default());
}

/// A [`DalContext`] for a workspace in a visibility which is not in a change set
///
/// To use a borrowed `DalContext` version, use [`DalContextHeadRef`].
/// To use mutably borrowed `DalContext` version, use [`DalContextHeadMutRef`].
pub struct DalContextHead(pub DalContext);

/// A reference to a [`DalContext`] for a workspace in a visibility which is not in a change
/// set
///
/// To use an owned `DalContext` version, use [`DalContextHead`].
/// To use mutably borrowed `DalContext` version, use [`DalContextHeadMutRef`].
pub struct DalContextHeadRef<'a>(pub &'a DalContext);

/// A mutable reference to a [`DalContext`] for a workspace in a visibility which is not in a
/// change set
///
/// To use an owned `DalContext` version, use [`DalContextHead`].
/// To use a borrowed `DalContext` version, use [`DalContextHeadRef`].
pub struct DalContextHeadMutRef<'a>(pub &'a mut DalContext);

/// An authentication token, used when making SDF API requests
pub struct AuthToken(pub String);

/// A referrence to an authentication token, used when making SDF API requests
pub struct AuthTokenRef<'a>(pub &'a str);

#[derive(Builder, Clone, Debug)]
pub struct Config {
    #[builder(default = "PgPoolConfig::default()")]
    pg: PgPoolConfig,
    #[builder(default = "NatsConfig::default()")]
    nats: NatsConfig,
    #[builder(default = "module_index_client::DEFAULT_URL.to_string()")]
    module_index_url: String,
    cyclone_encryption_key_path: String,
    jwt_signing_public_key_path: String,
    jwt_signing_private_key_path: String,
    postgres_key_path: String,
    #[builder(default)]
    pkgs_path: Option<PathBuf>,
    symmetric_crypto_service_config: SymmetricCryptoServiceConfig,
    // TODO(nick): determine why this is unused.
    #[allow(dead_code)]
    #[builder(default)]
    rebaser_config: RebaserClientConfig,
    #[builder(default = "PgStoreTools::default_pool_config()")]
    content_store_pg_pool: PgPoolConfig,
}

impl Config {
    #[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test and
                                         // all are prefixed with `SI_TEST_`
    fn create_default(
        pg_dbname: &'static str,
        content_store_pg_dbname: &'static str,
    ) -> Result<Self> {
        let mut config = {
            let mut builder = ConfigBuilder::default();
            detect_and_configure_testing(&mut builder)?;
            builder.build()?
        };

        if let Ok(value) = env::var(ENV_VAR_NATS_URL) {
            config.nats.url = value;
        }

        if let Ok(value) = env::var(ENV_VAR_PG_HOSTNAME) {
            config.pg.hostname = value;
        }
        config.pg.dbname = env::var(ENV_VAR_PG_DBNAME).unwrap_or_else(|_| pg_dbname.to_string());
        config.pg.user =
            env::var(ENV_VAR_PG_USER).unwrap_or_else(|_| DEFAULT_TEST_PG_USER.to_string());
        config.pg.port = env::var(ENV_VAR_PG_PORT)
            .unwrap_or_else(|_| DEFAULT_TEST_PG_PORT_STR.to_string())
            .parse()?;
        config.pg.pool_max_size *= 32;
        config.pg.certificate_path = Some(config.postgres_key_path.clone().try_into()?);

        if let Ok(value) = env::var(ENV_VAR_PG_HOSTNAME) {
            config.content_store_pg_pool.hostname = value;
        }
        config.content_store_pg_pool.dbname = env::var(ENV_VAR_CONTENT_STORE_PG_DBNAME)
            .unwrap_or_else(|_| content_store_pg_dbname.to_string());
        config.content_store_pg_pool.user =
            env::var(ENV_VAR_PG_USER).unwrap_or_else(|_| DEFAULT_TEST_PG_USER.to_string());
        config.content_store_pg_pool.port = env::var(ENV_VAR_PG_PORT)
            .unwrap_or_else(|_| DEFAULT_TEST_PG_PORT_STR.to_string())
            .parse()?;
        config.content_store_pg_pool.pool_max_size *= 32;
        config.content_store_pg_pool.certificate_path =
            Some(config.postgres_key_path.clone().try_into()?);

        if let Ok(value) = env::var(ENV_VAR_MODULE_INDEX_URL) {
            config.module_index_url = value;
        }

        debug!(?config, "test config");

        Ok(config)
    }
}

#[remain::sorted]
#[allow(clippy::large_enum_variant)]
enum ContextBuilderState {
    Created(TestContextBuilder),
    Errored(Cow<'static, str>),
    Uninitialized,
}

impl ContextBuilderState {
    fn created(builder: TestContextBuilder) -> Self {
        Self::Created(builder)
    }

    fn errored(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Errored(message.into())
    }

    fn config(&self) -> Result<&Config> {
        match self {
            Self::Created(builder) => Ok(&builder.config),
            Self::Errored(msg) => Err(eyre!("global setup has failed: {msg}")),
            Self::Uninitialized => Err(eyre!("global setup is uninitialized")),
        }
    }
}

impl Default for ContextBuilderState {
    fn default() -> Self {
        Self::Uninitialized
    }
}

/// A context used for preparing and running tests containing DAL objects.
#[derive(Clone, Debug)]
pub struct TestContext {
    /// The test context configuration used to build this instance.
    config: Config,
    /// A PostgreSQL connection pool.
    pg_pool: PgPool,
    /// A connected NATS client
    nats_conn: NatsClient,
    /// A [`JobQueueProcessor`] impl
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<CycloneEncryptionKey>,
    /// A service that can encrypt values based on the loaded donkeys
    symmetric_crypto_service: SymmetricCryptoService,
    /// The pg_pool used by the content-addressable [`store`](content_store::Store) used by the
    /// "dal".
    content_store_pg_pool: PgPool,

    /// The configuration for the rebaser client used in tests
    rebaser_config: RebaserClientConfig,
}

impl TestContext {
    /// Builds and returns a suitable [`TestContext`] from a global configuration which is ready to
    /// run tests.
    ///
    /// # Implementation Details
    ///
    /// This functions wraps over a mutex which ensures that only the first caller will run global
    /// database creation, migrations, and other preparations.
    pub async fn global(
        pg_dbname: &'static str,
        content_store_pg_dbname: &'static str,
    ) -> Result<Self> {
        let mut mutex_guard = TEST_CONTEXT_BUILDER.lock().await;

        match &*mutex_guard {
            ContextBuilderState::Uninitialized => {
                let config = Config::create_default(pg_dbname, content_store_pg_dbname)
                    .si_inspect_err(|err| {
                        *mutex_guard = ContextBuilderState::errored(err.to_string())
                    })?;
                let test_context_builder = TestContextBuilder::create(config)
                    .await
                    .si_inspect_err(|err| {
                        *mutex_guard = ContextBuilderState::errored(err.to_string());
                    })?;

                // The stack gets too deep here, so we'll spawn the work as a task with a new
                // thread stack just for the global setup
                let handle = tokio::spawn(global_setup(test_context_builder.clone()));

                // Join this task and wait on its completion
                match handle.await {
                    // Global setup completed successfully
                    Ok(Ok(())) => {
                        debug!("task global_setup was successful");
                        *mutex_guard = ContextBuilderState::created(test_context_builder.clone());
                        test_context_builder.build_for_test().await
                    }
                    // Global setup errored
                    Ok(Err(err)) => {
                        *mutex_guard = ContextBuilderState::errored(err.to_string());
                        Err(err)
                    }
                    // Tokio task panicked or was cancelled
                    Err(err) => {
                        if err.is_panic() {
                            error!(error = %err, "spawned task global_setup panicked!");
                        } else if err.is_cancelled() {
                            error!(error = %err, "spawned task global_setup was cancelled!");
                        }
                        *mutex_guard = ContextBuilderState::errored(err.to_string());
                        Err(err.into())
                    }
                }
            }
            ContextBuilderState::Created(builder) => builder.build_for_test().await,
            ContextBuilderState::Errored(message) => {
                error!(error = %message, "global setup failed, aborting test");
                Err(eyre!("global setup failed: {}", message))
            }
        }
    }

    /// Creates a new [`ServicesContext`].
    pub async fn create_services_context(&self) -> ServicesContext {
        let veritech = veritech_client::Client::new(self.nats_conn.clone());

        ServicesContext::new(
            self.pg_pool.clone(),
            self.nats_conn.clone(),
            self.job_processor.clone(),
            veritech,
            self.encryption_key.clone(),
            self.config.pkgs_path.to_owned(),
            None,
            self.symmetric_crypto_service.clone(),
            self.rebaser_config.clone(),
            self.content_store_pg_pool.clone(),
        )
    }

    /// Gets a reference to the NATS configuration.
    pub fn nats_config(&self) -> &NatsConfig {
        &self.config.nats
    }
}

/// A builder for a [`TestContext`].
///
/// Each `TestContext` has an active connection pool to the database and messaging system, and
/// rather than share these single pools among all global set and all test tests, a new set of
/// dedicated pools can be created as needed. This builder holds all other state other than the
/// pool-acquiring steps.
#[derive(Clone, Debug)]
struct TestContextBuilder {
    /// The test context configuration used to build this instance.
    config: Config,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<CycloneEncryptionKey>,
}

impl TestContextBuilder {
    /// Creates a new builder.
    async fn create(config: Config) -> Result<Self> {
        let encryption_key = Arc::new(
            CycloneEncryptionKey::load(&config.cyclone_encryption_key_path)
                .await
                .wrap_err("failed to load EncryptionKey")?,
        );

        Ok(Self {
            config,
            encryption_key,
        })
    }

    /// Builds and returns a new [`TestContext`] with its own connection pooling for global setup.
    async fn build_for_global(&self) -> Result<TestContext> {
        let pg_pool = PgPool::new(&self.config.pg)
            .await
            .wrap_err("failed to create global setup PgPool")?;
        let content_store_pool = PgPool::new(&self.config.content_store_pg_pool)
            .await
            .wrap_err("failed to create global setup content store PgPool")?;

        self.build_inner(pg_pool, content_store_pool).await
    }

    /// Builds and returns a new [`TestContext`] with its own connection pooling for each test.
    async fn build_for_test(&self) -> Result<TestContext> {
        let pg_pool = self
            .create_test_specific_db_with_pg_pool(&self.config.pg)
            .await?;
        let content_store_pg_pool = self
            .create_test_specific_db_with_pg_pool(&self.config.content_store_pg_pool)
            .await?;

        self.build_inner(pg_pool, content_store_pg_pool).await
    }

    async fn build_inner(
        &self,
        pg_pool: PgPool,
        content_store_pg_pool: PgPool,
    ) -> Result<TestContext> {
        let universal_prefix = random_identifier_string();

        // Need to make a new NatsConfig so that we can add the test-specific subject prefix
        // without leaking it to other tests.
        let mut nats_config = self.config.nats.clone();
        nats_config.subject_prefix = Some(universal_prefix.clone());
        let mut config = self.config.clone();
        config.nats.subject_prefix = Some(universal_prefix.clone());

        let nats_conn = NatsClient::new(&nats_config)
            .await
            .wrap_err("failed to create NatsClient")?;
        let job_processor = Box::new(NatsProcessor::new(nats_conn.clone()))
            as Box<dyn JobQueueProcessor + Send + Sync>;

        let symmetric_crypto_service =
            SymmetricCryptoService::from_config(&self.config.symmetric_crypto_service_config)
                .await?;

        let mut rebaser_config = RebaserClientConfig::default();
        rebaser_config.set_subject_prefix(universal_prefix);

        Ok(TestContext {
            config,
            pg_pool,
            nats_conn,
            job_processor,
            encryption_key: self.encryption_key.clone(),
            symmetric_crypto_service,
            rebaser_config,
            content_store_pg_pool,
        })
    }

    async fn create_test_specific_db_with_pg_pool(
        &self,
        pg_pool_config: &PgPoolConfig,
    ) -> Result<PgPool> {
        // Connect to the 'postgres' database so we can copy our migrated template test database
        let mut new_pg_pool_config = pg_pool_config.clone();
        new_pg_pool_config.dbname = "postgres".to_string();
        let new_pg_pool = PgPool::new(&new_pg_pool_config)
            .await
            .wrap_err("failed to create PgPool to db 'postgres'")?;
        let db_conn = new_pg_pool
            .get()
            .await
            .wrap_err("failed to connect to db 'postgres'")?;

        // Create new database from template
        let db_name_suffix = random_identifier_string();
        let dbname = format!("{}_{}", pg_pool_config.dbname, db_name_suffix);
        let query = format!(
            "CREATE DATABASE {dbname} WITH TEMPLATE {} OWNER {};",
            pg_pool_config.dbname, pg_pool_config.user,
        );
        let db_exists_check = db_conn
            .query_opt(
                "SELECT datname FROM pg_database WHERE datname = $1",
                &[&dbname],
            )
            .await?;
        if db_exists_check.is_none() {
            info!(dbname = %dbname, "creating test-specific database");
            db_conn
                .execute(&query, &[])
                .instrument(debug_span!("creating test database from template"))
                .await
                .wrap_err("failed to create test specific database")?;
        } else {
            info!(dbname = %dbname, "test-specific database already exists");
        }
        // This is ugly, but we pretty much always want to know which test DB is used for
        // any given test when it fails, and the logging/tracing macros aren't captured
        // (or displayed) during tests, while `println!(...)` will be captured the same as
        // "normal" test output, meaning it respects --nocapture and being displayed for
        // failing tests.
        println!("Test database: {}", &dbname);

        // Return new PG pool that uess the new datatbase
        new_pg_pool_config.dbname = dbname;
        PgPool::new(&new_pg_pool_config)
            .await
            .wrap_err("failed to create PgPool to db 'postgres'")
    }
}

/// Generates a new pseudo-random NATS subject prefix.
pub fn random_identifier_string() -> String {
    Uuid::new_v4().as_simple().to_string()
}

// Returns a JWT public signing key, used to verify claims
pub async fn jwt_public_signing_key() -> Result<JwtPublicSigningKey> {
    let jwt_signing_public_key_path = {
        let context_builder = TEST_CONTEXT_BUILDER.lock().await;
        let config = context_builder.config()?;
        config.jwt_signing_public_key_path.clone()
    };
    let key = JwtPublicSigningKey::load(&jwt_signing_public_key_path).await?;

    Ok(key)
}

// Returns a JWT private signing key, used to sign claims
pub async fn jwt_private_signing_key() -> Result<RS256KeyPair> {
    let key_path = {
        let context_builder = TEST_CONTEXT_BUILDER.lock().await;
        let config = context_builder.config()?;
        config.jwt_signing_private_key_path.clone()
    };
    let key_str = {
        let mut file = File::open(key_path)
            .await
            .wrap_err("failed to open RSA256 key file")?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .await
            .wrap_err("failed to read from RSA256 file")?;
        buf
    };

    let key_pair = RS256KeyPair::from_pem(&key_str).expect("failed to parse RSA256 from pem file");

    Ok(key_pair)
}

/// Configures and builds a [`council_server::Server`] suitable for running alongside DAL object-related
/// tests.
pub async fn council_server(nats_config: NatsConfig) -> Result<council_server::Server> {
    let config = council_server::server::Config::builder()
        .nats(nats_config)
        .build()?;
    let server = council_server::Server::new_with_config(config).await?;
    Ok(server)
}

/// Configures and builds a [`pinga_server::Server`] suitable for running alongside DAL
/// object-related tests.
pub fn pinga_server(services_context: &ServicesContext) -> Result<pinga_server::Server> {
    let config: pinga_server::Config = {
        let mut config_file = pinga_server::ConfigFile::default();
        pinga_server::detect_and_configure_development(&mut config_file)
            .wrap_err("failed to detect and configure Pinga ConfigFile")?;
        config_file
            .try_into()
            .wrap_err("failed to build Pinga server config")?
    };

    let server = pinga_server::Server::from_services(
        config.instance_id(),
        config.concurrency(),
        services_context.clone(),
    )
    .wrap_err("failed to create Pinga server")?;

    Ok(server)
}

/// Configures and builds a [`rebaser_server::Server`] suitable for running alongside DAL
/// object-related tests.
pub fn rebaser_server(services_context: &ServicesContext) -> Result<rebaser_server::Server> {
    let _config: rebaser_server::Config = {
        let mut config_file = rebaser_server::ConfigFile::default();
        rebaser_server::detect_and_configure_development(&mut config_file)
            .wrap_err("failed to detect and configure Rebaser ConfigFile")?;
        config_file
            .try_into()
            .wrap_err("failed to build Rebaser server config")?
    };

    let server = rebaser_server::Server::from_services(
        services_context.encryption_key(),
        services_context.nats_conn().clone(),
        services_context.pg_pool().clone(),
        services_context.veritech().clone(),
        services_context.job_processor(),
        services_context.symmetric_crypto_service().clone(),
        services_context.rebaser_config().clone(),
        services_context.content_store_pg_pool().clone(),
    )
    .wrap_err("failed to create Rebaser server")?;

    Ok(server)
}

/// Configures and builds a [`veritech_server::Server`] suitable for running alongside DAL
/// object-related tests.
pub async fn veritech_server_for_uds_cyclone(
    nats_config: NatsConfig,
) -> Result<veritech_server::Server> {
    let config: veritech_server::Config = {
        let mut config_file = veritech_server::ConfigFile {
            nats: nats_config,
            ..Default::default()
        };
        veritech_server::detect_and_configure_development(&mut config_file)
            .wrap_err("failed to detect and configure Veritech ConfigFile")?;
        config_file
            .try_into()
            .wrap_err("failed to build Veritech server config")?
    };

    let server = veritech_server::Server::for_cyclone_uds(config)
        .await
        .wrap_err("failed to create Veritech server")?;

    Ok(server)
}

async fn global_setup(test_context_builer: TestContextBuilder) -> Result<()> {
    info!("running global test setup");
    let test_context = test_context_builer.build_for_global().await?;

    debug!("initializing crypto");
    sodiumoxide::init().map_err(|_| eyre!("failed to init sodiumoxide crypto"))?;

    // Create a `ServicesContext`
    let services_ctx = test_context.create_services_context().await;

    // Create a dedicated Council server with a unique subject prefix for each test
    let council_server = council_server(test_context.config.nats.clone()).await?;
    let (council_shutdown_request_tx, shutdown_request_rx) = tokio::sync::watch::channel(());
    let (subscriber_started_tx, mut subscriber_started_rx) = tokio::sync::watch::channel(());
    tokio::spawn(async move {
        council_server
            .run(subscriber_started_tx, shutdown_request_rx)
            .await
            .unwrap()
    });
    subscriber_started_rx.changed().await?;

    // Start up a Pinga server as a task exclusively to allow the migrations to run
    info!("starting Pinga server for initial migrations");
    let pinga_server = pinga_server(&services_ctx)?;
    let pinga_server_handle = pinga_server.shutdown_handle();
    tokio::spawn(pinga_server.run());

    // Start up a Rebaser server for migrations
    info!("starting Rebaser server for initial migrations");
    let rebaser_server = rebaser_server(&services_ctx)?;
    let rebaser_server_handle = rebaser_server.shutdown_handle();
    tokio::spawn(rebaser_server.run());

    // Start up a Veritech server as a task exclusively to allow the migrations to run
    info!("starting Veritech server for initial migrations");
    let veritech_server = veritech_server_for_uds_cyclone(test_context.config.nats.clone()).await?;
    let veritech_server_handle = veritech_server.shutdown_handle();
    tokio::spawn(veritech_server.run());

    info!("creating client with pg pool for global Content Store test database");

    info!("testing database connection");
    services_ctx
        .pg_pool()
        .test_connection()
        .await
        .wrap_err("failed to connect to database, is it running and available?")?;

    info!("testing global content store database connection");
    services_ctx
        .content_store_pg_pool()
        .test_connection()
        .await
        .wrap_err("failed to connect to content store database, is it running and available?")?;

    #[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test and
    // all are prefixed with `SI_TEST_`
    if !env::var(ENV_VAR_KEEP_OLD_DBS).is_ok_and(|v| !v.is_empty()) {
        info!("dropping old test-specific databases for dal");
        drop_old_test_databases(services_ctx.pg_pool())
            .await
            .wrap_err("failed to drop old databases")?;

        info!("dropping old test-specific content store databases");
        drop_old_test_databases(services_ctx.content_store_pg_pool())
            .await
            .wrap_err("failed to drop old test-specific content store databases")?;
    }

    // Ensure the database is totally clean, then run all migrations
    info!("dropping and re-creating the database schema");
    services_ctx
        .pg_pool()
        .drop_and_create_public_schema()
        .await
        .wrap_err("failed to drop and create the database")?;

    services_ctx
        .content_store_pg_pool()
        .drop_and_create_public_schema()
        .await
        .wrap_err("failed to drop and create content store database")?;

    info!("running database migrations");
    dal::migrate(services_ctx.pg_pool(), services_ctx.content_store_pg_pool())
        .await
        .wrap_err("failed to migrate database")?;

    // Check if the user would like to skip migrating schemas. This is helpful for boosting
    // performance when running integration tests that do not rely on builtin schemas.
    // let selected_test_builtin_schemas = determine_selected_test_builtin_schemas();

    info!("creating builtins");
    migrate_local_builtins(
        services_ctx.pg_pool(),
        services_ctx.nats_conn(),
        services_ctx.job_processor(),
        services_ctx.veritech().clone(),
        &services_ctx.encryption_key(),
        test_context
            .config
            .pkgs_path
            .to_owned()
            .expect("no pkgs path configured"),
        test_context.config.module_index_url.clone(),
        services_ctx.symmetric_crypto_service(),
        services_ctx.rebaser_config().clone(),
        services_ctx.content_store_pg_pool(),
    )
    .await
    .wrap_err("failed to run builtin migrations")?;

    // Shutdown the Pinga server (each test gets their own server instance with an exclusively
    // unique subject prefix)
    info!("shutting down initial migrations Pinga server");
    pinga_server_handle.shutdown().await;

    // Shutdown the Rebaser server (each test gets their own server instance with an exclusively
    // unique subject prefix)
    info!("shutting down initial migrations Rebaser server");
    rebaser_server_handle.shutdown().await;

    // Shutdown the Veritech server (each test gets their own server instance with an exclusively
    // unique subject prefix)
    info!("shutting down initial migrations Veritech server");
    veritech_server_handle.shutdown().await;

    info!("shutting down initial migrations Council server");
    council_shutdown_request_tx.send(())?;

    info!("global test setup complete");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[instrument(level = "info", skip_all)]
async fn migrate_local_builtins(
    dal_pg: &PgPool,
    nats: &NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    veritech: veritech_client::Client,
    encryption_key: &CycloneEncryptionKey,
    pkgs_path: PathBuf,
    module_index_url: String,
    symmetric_crypto_service: &SymmetricCryptoService,
    rebaser_config: RebaserClientConfig,
    content_store_pg_pool: &PgPool,
) -> ModelResult<()> {
    let services_context = ServicesContext::new(
        dal_pg.clone(),
        nats.clone(),
        job_processor,
        veritech,
        Arc::new(*encryption_key),
        Some(pkgs_path),
        Some(module_index_url),
        symmetric_crypto_service.clone(),
        rebaser_config,
        content_store_pg_pool.clone(),
    );
    let dal_context = services_context.into_builder(true);
    let mut ctx = dal_context.build_default().await?;

    let workspace = Workspace::builtin(&mut ctx).await?;
    ctx.update_tenancy(Tenancy::new(*workspace.pk()));
    ctx.update_to_head();
    ctx.update_snapshot_to_visibility().await?;

    info!("migrating intrinsic functions");
    func::migrate_intrinsics(&ctx).await?;

    // FIXME(nick): restore builtin migration functionality for all variants.
    info!("migrate minimal number of schemas for testing the new engine");

    schema::migrate_pkg(&ctx, SI_DOCKER_IMAGE_PKG, None).await?;
    schema::migrate_pkg(&ctx, SI_COREOS_PKG, None).await?;
    schema::migrate_pkg(&ctx, SI_AWS_EC2_PKG, None).await?;
    schemas::migrate_test_exclusive_schema_starfield(&ctx).await?;
    schemas::migrate_test_exclusive_schema_fallout(&ctx).await?;
    schemas::migrate_test_exclusive_schema_bethesda_secret(&ctx).await?;
    schemas::migrate_test_exclusive_schema_swifty(&ctx).await?;

    ctx.blocking_commit().await?;

    Ok(())
}

// fn determine_selected_test_builtin_schemas() -> SelectedTestBuiltinSchemas {
//     #[allow(clippy::disallowed_methods)] // Environment variables are used exclusively in test and
//     // all are prefixed with `SI_TEST_`
//     //
//     // TODO(fnichol): remove conditional schema execution
//     match env::var(ENV_VAR_BUILTIN_SCHEMAS) {
//         Ok(found_value) => {
//             let mut builtin_schemas = HashSet::new();
//
//             // If the value does not contain a comma, we will have exactly once item to iterate
//             // over.
//             for builtin_schema in found_value.split(',') {
//                 // Trim and ensure the string is lowercase.
//                 let cleaned = builtin_schema.trim().to_lowercase();
//
//                 // If we receive any keywords indicating that we need to return early, let's do so.
//                 if &cleaned == "none" || &cleaned == "false" {
//                     return SelectedTestBuiltinSchemas::None;
//                 } else if &cleaned == "all" {
//                     return SelectedTestBuiltinSchemas::All;
//                 } else if &cleaned == "test" || &cleaned == "true" {
//                     return SelectedTestBuiltinSchemas::Test;
//                 }
//
//                 // If we do not find any keywords, we assume that the user provided the name for a
//                 // builtin schema.
//                 builtin_schemas.insert(cleaned);
//             }
//             SelectedTestBuiltinSchemas::Some(builtin_schemas)
//         }
//         Err(_) => {
//             // If the variable is unset, then we migrate everything. This is the default behavior.
//             SelectedTestBuiltinSchemas::Test
//         }
//     }
// }

async fn drop_old_test_databases(pg_pool: &PgPool) -> Result<()> {
    let name_prefix = format!("{}_%", pg_pool.db_name());
    let pg_conn = pg_pool.get().await?;

    let rows = pg_conn
        .query(
            "SELECT datname FROM pg_database WHERE datname LIKE $1",
            &[&name_prefix.as_str()],
        )
        .await?;

    for row in rows {
        let dbname: String = row.try_get("datname")?;
        debug!(db_name = %dbname, "dropping database");
        pg_conn
            .execute(&format!("DROP DATABASE IF EXISTS {dbname}"), &[])
            .await?;
    }

    Ok(())
}

#[allow(clippy::disallowed_methods)] // Used to determine if running in testing
fn detect_and_configure_testing(builder: &mut ConfigBuilder) -> Result<()> {
    if env::var("BUCK_RUN_BUILD_ID").is_ok() || env::var("BUCK_BUILD_ID").is_ok() {
        detect_and_configure_testing_for_buck2(builder)
    } else if let Ok(dir) = env::var("CARGO_MANIFEST_DIR") {
        detect_and_configure_testing_for_cargo(dir, builder)
    } else {
        unimplemented!("tests must be run either with Cargo or Buck2");
    }
}

fn detect_and_configure_testing_for_buck2(builder: &mut ConfigBuilder) -> Result<()> {
    let resources = Buck2Resources::read()?;

    let cyclone_encryption_key_path = resources
        .get_ends_with("dev.encryption.key")?
        .to_string_lossy()
        .to_string();
    let jwt_signing_public_key_path = resources
        .get_ends_with("dev.jwt_signing_public_key.pem")?
        .to_string_lossy()
        .to_string();
    let jwt_signing_private_key_path = resources
        .get_ends_with("dev.jwt_signing_private_key.pem")?
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = resources
        .get_ends_with("dev.donkey.key")?
        .to_string_lossy()
        .to_string();
    let postgres_key = resources
        .get_ends_with("dev.postgres.root.crt")?
        .to_string_lossy()
        .to_string();
    let pkgs_path = resources
        .get_ends_with("pkgs_path")?
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        jwt_signing_private_key_path = jwt_signing_private_key_path.as_str(),
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_key = postgres_key.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    builder.cyclone_encryption_key_path(cyclone_encryption_key_path);
    builder.jwt_signing_public_key_path(jwt_signing_public_key_path);
    builder.jwt_signing_private_key_path(jwt_signing_private_key_path);
    builder.symmetric_crypto_service_config(
        SymmetricCryptoServiceConfigFile {
            active_key: Some(symmetric_crypto_service_key),
            active_key_base64: None,
            extra_keys: vec![],
        }
        .try_into()?,
    );
    builder.postgres_key_path(postgres_key);
    builder.pkgs_path(Some(pkgs_path.into()));

    Ok(())
}

fn detect_and_configure_testing_for_cargo(dir: String, builder: &mut ConfigBuilder) -> Result<()> {
    let cyclone_encryption_key_path = Path::new(&dir)
        .join("../../lib/cyclone-server/src/dev.encryption.key")
        .to_string_lossy()
        .to_string();
    let jwt_signing_public_key_path = Path::new(&dir)
        .join("../../config/keys/dev.jwt_signing_public_key.pem")
        .to_string_lossy()
        .to_string();
    let jwt_signing_private_key_path = Path::new(&dir)
        .join("../../config/keys/dev.jwt_signing_private_key.pem")
        .to_string_lossy()
        .to_string();
    let symmetric_crypto_service_key = Path::new(&dir)
        .join("../../lib/dal/dev.donkey.key")
        .to_string_lossy()
        .to_string();
    let postgres_key = Path::new(&dir)
        .join("../../config/keys/dev.postgres.root.crt")
        .to_string_lossy()
        .to_string();
    let pkgs_path = Path::new(&dir)
        .join("../../pkgs")
        .to_string_lossy()
        .to_string();

    warn!(
        cyclone_encryption_key_path = cyclone_encryption_key_path.as_str(),
        jwt_signing_private_key_path = jwt_signing_private_key_path.as_str(),
        jwt_signing_public_key_path = jwt_signing_public_key_path.as_str(),
        symmetric_crypto_service_key = symmetric_crypto_service_key.as_str(),
        postgres_key = postgres_key.as_str(),
        pkgs_path = pkgs_path.as_str(),
        "detected development run",
    );

    builder.cyclone_encryption_key_path(cyclone_encryption_key_path);
    builder.jwt_signing_public_key_path(jwt_signing_public_key_path);
    builder.jwt_signing_private_key_path(jwt_signing_private_key_path);
    builder.symmetric_crypto_service_config(
        SymmetricCryptoServiceConfigFile {
            active_key: Some(symmetric_crypto_service_key),
            active_key_base64: None,
            extra_keys: vec![],
        }
        .try_into()?,
    );
    builder.postgres_key_path(postgres_key);
    builder.pkgs_path(Some(pkgs_path.into()));

    Ok(())
}
