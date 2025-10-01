#![warn(
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::missing_panics_doc,
    clippy::panic_in_result_fn
)]
#![allow(clippy::missing_errors_doc)]

use std::{
    cell::RefCell,
    cmp,
    fmt::{
        self,
        Debug,
    },
    net::ToSocketAddrs,
    rc::Rc,
    sync::Arc,
    time::Duration,
};

use bytes::Buf;
use deadpool::managed::Object;
use deadpool_postgres::{
    Config,
    ConfigError,
    CreatePoolError,
    Manager,
    ManagerConfig,
    Pool,
    PoolConfig,
    PoolError,
    RecyclingMethod,
    SslMode,
    Transaction,
    TransactionBuilder,
};
use futures::{
    Stream,
    StreamExt,
};
use ouroboros::self_referencing;
use serde::{
    Deserialize,
    Serialize,
};
use si_std::{
    ResultExt,
    SensitiveString,
};
use si_tls::{
    CertificateResolver,
    CertificateSource,
    TlsError,
};
use telemetry::prelude::*;
use tokio::sync::Mutex;
use tokio_postgres::{
    CancelToken,
    Client,
    Column,
    CopyInSink,
    CopyOutStream,
    IsolationLevel,
    Portal,
    Row,
    SimpleQueryMessage,
    Statement,
    ToStatement,
    row::RowIndex,
    types::{
        BorrowToSql,
        FromSql,
        ToSql,
        Type,
    },
};
pub use tokio_postgres::{
    error::SqlState,
    types as postgres_types,
};
use tokio_postgres_rustls::MakeRustlsConnect;

const MIGRATION_LOCK_NUMBER: i64 = 42;
const MAX_POOL_SIZE_MINIMUM: usize = 32;

const TEST_QUERY: &str = "SELECT 1";

// If a table's structure changes, cached query plans against that table need to
// be invalidated, or postgresql will return an error. This prevents that error
// after migrating the database in a production system running pb_bouncer, which
// holds on to connections and reuses them even if our services are restarted.
// We could avoid needing to discard plans by selecting exactly the columns we
// need instead of SELECT * (unless the column type changes!)
//
// We set `CLIENT_MIN_MESSAGES` to `ERROR` to silence the possible warning from `ROLLBACK` about
// there not being any open transaction. We immediately set it back to the default value
// (`WARNING`) after so we're not hiding any "real" warnings that might happen.
const CONNECTION_RECYCLING_METHOD: &str = r#"
    SET CLIENT_MIN_MESSAGES TO ERROR;
    ROLLBACK;
    SET CLIENT_MIN_MESSAGES TO WARNING;
    CLOSE ALL;
    SET SESSION AUTHORIZATION DEFAULT;
    RESET ALL;
    UNLISTEN *;
    SELECT pg_advisory_unlock_all();
    DISCARD TEMP;
    DISCARD SEQUENCES;
    DISCARD PLANS;
"#;

#[remain::sorted]
#[derive(thiserror::Error, Debug)]
pub enum PgError {
    #[error(transparent)]
    Pg(#[from] tokio_postgres::Error),
    #[error("transaction not exclusively referenced when commit attempted; arc_strong_count={0}")]
    TxnCommitNotExclusive(usize),
    #[error("transaction not exclusively referenced when rollback attempted; arc_strong_count={0}")]
    TxnRollbackNotExclusive(usize),
    #[error("unexpected row returned: {0:?}")]
    UnexpectedRow(Box<PgRow>),
}

#[remain::sorted]
#[derive(thiserror::Error, Debug)]
pub enum PgPoolError {
    #[error("failed to decode base64 encoded key")]
    Base64Decode(#[source] base64::DecodeError),
    #[error("failed to certificate from bytes")]
    CreateCertificate(#[from] std::io::Error),
    #[error("creating pg pool error: {0}")]
    CreatePoolError(#[from] CreatePoolError),
    #[error("pg pool config error: {0}")]
    DeadpoolConfig(#[from] ConfigError),
    #[error("tokio pg error: {0}")]
    Pg(#[from] PgError),
    #[error("pg pool error: {0}")]
    PoolError(#[from] PoolError),
    #[error("failed to read pem")]
    ReadPem(std::io::Error),
    #[error("migration error: {0}")]
    Refinery(#[from] refinery::Error),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to resolve pg hostname")]
    ResolveHostname(std::io::Error),
    #[error("resolved hostname returned no entries")]
    ResolveHostnameNoEntries,
    #[error("test connection query returned incorrect result; expected={0}, got={1}")]
    TestConnectionResult(i32, i32),
    #[error("tls error: {0}")]
    Tls(#[from] TlsError),
    #[error("tokio task join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("tokio pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
}

pub type PgPoolResult<T> = Result<T, PgPoolError>;
pub type PgTxn = PgSharedTransaction;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PgPoolConfig {
    pub user: String,
    pub password: SensitiveString,
    pub certificate: Option<CertificateSource>,
    pub dbname: String,
    pub application_name: String,
    pub hostname: String,
    pub port: u16,
    pub pool_max_size: usize,
    pub pool_timeout_wait_secs: Option<u64>,
    pub pool_timeout_create_secs: Option<u64>,
    pub pool_timeout_recycle_secs: Option<u64>,
    pub pool_total_connection_lifetime_secs: u64,
    pub pool_idle_connection_lifetime_secs: u64,
    /// If set to `None`, the eviction task won't be started
    pub pool_lifetime_check_interval_secs: u64,
    pub recycling_method: Option<RecyclingMethodConfig>,
}

impl Default for PgPoolConfig {
    fn default() -> Self {
        let pool_max_size = cmp::max(MAX_POOL_SIZE_MINIMUM, num_cpus::get_physical() * 4);

        PgPoolConfig {
            user: String::from("si"),
            password: SensitiveString::from("bugbear"),
            certificate: None,
            dbname: String::from("si"),
            application_name: String::from("si-unknown-app"),
            hostname: String::from("localhost"),
            port: 5432,
            pool_max_size,
            pool_timeout_wait_secs: None,
            pool_timeout_create_secs: None,
            pool_timeout_recycle_secs: None,
            pool_total_connection_lifetime_secs: 20 * 3600, // 20 hours (RDS Proxy has a 24-hour max)
            pool_idle_connection_lifetime_secs: 6 * 3600,   // 6 hours
            pool_lifetime_check_interval_secs: 2 * 3600,    // 2 hours
            recycling_method: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RecyclingMethodConfig {
    Clean,
    Custom,
    Fast,
    Verified,
}

impl From<RecyclingMethodConfig> for RecyclingMethod {
    fn from(config: RecyclingMethodConfig) -> Self {
        match config {
            RecyclingMethodConfig::Clean => RecyclingMethod::Clean,
            RecyclingMethodConfig::Custom => {
                RecyclingMethod::Custom(CONNECTION_RECYCLING_METHOD.to_string())
            }
            RecyclingMethodConfig::Fast => RecyclingMethod::Fast,
            RecyclingMethodConfig::Verified => RecyclingMethod::Verified,
        }
    }
}

#[derive(Clone)]
pub struct PgPool {
    pool: Pool,
    metadata: Arc<ConnectionMetadata>,
}

impl std::fmt::Debug for PgPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgPool")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Debug)]
struct ConnectionMetadata {
    db_system: &'static str,
    db_connection_string: String,
    db_name: String,
    db_user: String,
    db_pool_max_size: usize,
    net_peer_ip: String,
    net_peer_port: u16,
    net_transport: &'static str,
}

impl PgPool {
    #[instrument(
        name = "pg_pool::new",
        skip_all,
        level = "debug",
        fields(
            db.system = Empty,
            db.connection_string = Empty,
            db.name = Empty,
            db.user = Empty,
            db.pool.max_size = Empty,
            net.peer.ip = Empty,
            net.peer.port = Empty,
            net.transport = Empty,
        )
    )]
    pub async fn new(settings: &PgPoolConfig) -> PgPoolResult<Self> {
        let span = current_span_for_instrument_at!("debug");

        let mut cfg = Config::new();
        cfg.hosts = Some(vec![settings.hostname.clone()]);
        cfg.port = Some(settings.port);
        cfg.user = Some(settings.user.clone());
        cfg.password = Some(settings.password.clone().into());
        cfg.dbname = Some(settings.dbname.clone());
        cfg.application_name = Some(settings.application_name.clone());

        cfg.manager = Some(ManagerConfig {
            recycling_method: settings
                .recycling_method
                .as_ref()
                .cloned()
                .unwrap_or(RecyclingMethodConfig::Fast)
                .into(),
        });

        let mut pool_config = PoolConfig::new(settings.pool_max_size);
        if let Some(secs) = settings.pool_timeout_wait_secs {
            pool_config.timeouts.wait = Some(Duration::from_secs(secs));
        }
        if let Some(secs) = settings.pool_timeout_create_secs {
            pool_config.timeouts.create = Some(Duration::from_secs(secs));
        }
        if let Some(secs) = settings.pool_timeout_recycle_secs {
            pool_config.timeouts.recycle = Some(Duration::from_secs(secs));
        }

        // TODO(scott): we should set this to Require as below once
        // the postgres:stable image has been updated
        // to include the SSL bits.
        // cfg.ssl_mode = Some(SslMode::Require);
        cfg.ssl_mode = Some(SslMode::Prefer);
        let tls_config = Self::tls_config(settings.certificate.as_ref()).await?;
        debug!(db.pool_config = ?pool_config);
        cfg.pool = Some(pool_config);
        let pool = cfg.create_pool(Some(deadpool_postgres::Runtime::Tokio1), tls_config)?;

        let resolving_hostname = format!("{}:{}", settings.hostname, settings.port);
        let net_peer_ip = tokio::task::spawn_blocking(move || {
            resolving_hostname
                .to_socket_addrs()
                .map_err(PgPoolError::ResolveHostname)
                .and_then(|mut iter| iter.next().ok_or(PgPoolError::ResolveHostnameNoEntries))
                .map(|socket_addr| socket_addr.ip().to_string())
        })
        .await??;

        let metadata = ConnectionMetadata {
            db_system: "postgresql",
            db_connection_string: format!(
                "postgresql://{}:{}/{}?application_name={}",
                settings.hostname, settings.port, settings.dbname, settings.application_name
            ),
            db_name: settings.dbname.clone(),
            db_user: settings.user.clone(),
            db_pool_max_size: settings.pool_max_size,
            net_peer_ip,
            net_peer_port: settings.port,
            net_transport: "ip_tcp",
        };

        span.record("db.system", metadata.db_system);
        span.record(
            "db.connection_string",
            metadata.db_connection_string.as_str(),
        );
        span.record("db.name", metadata.db_name.as_str());
        span.record("db.user", metadata.db_user.as_str());
        span.record("db.pool.max_size", metadata.db_pool_max_size);
        span.record("net.peer.ip", metadata.net_peer_ip.as_str());
        span.record("net.peer.port", metadata.net_peer_port);
        span.record("net.transport", metadata.net_transport);

        // We need this to get rid of connections that may have been dropped by other systems,
        // such as RDS proxy. Initially we didn't have a limit on the total lifetime of connections,
        // but this causes some bugs since AWS very much does.
        let check_interval = Duration::from_secs(settings.pool_lifetime_check_interval_secs);

        let max_age = Duration::from_secs(settings.pool_total_connection_lifetime_secs);
        let max_idle_time = Duration::from_secs(settings.pool_idle_connection_lifetime_secs);

        tokio::spawn(evict_old_and_idle_pgpool_connections(
            pool.clone(),
            check_interval,
            max_age,
            max_idle_time,
        ));

        let pg_pool = Self {
            pool,
            metadata: Arc::new(metadata),
        };

        // Warm up the pool and test that we can connect to the database. Note that this is only
        // advisory--it will not terminate any process or service that may be running or about to
        // be run. We assume that the pool is an autonomous actor that can make forward progress
        // towards its goal and maintain its own healthiness. This is in order to prevent a
        // database network connection hiccup from crashing a fleet of services which may get
        // immediately rescheduled/restarted only to fall into a perpetual crash loop while not
        // being able to serve any traffic--including health/readiness status.
        drop(tokio::spawn(
            test_connection_infallible_and_warm_up_pool_task(pg_pool.clone()),
        ));

        Ok(pg_pool)
    }

    // Creates a tls_config for connecting to postgres securely
    async fn tls_config(cert: Option<&CertificateSource>) -> PgPoolResult<MakeRustlsConnect> {
        let mut root_cert_store = CertificateResolver::create_root_store().await?;
        if let Some(cert) = cert {
            cert.add_to_cert_store(&mut root_cert_store).await?;
        }
        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();

        Ok(MakeRustlsConnect::new(config))
    }

    /// Attempts to establish a database connection and returns an error if not successful.
    #[instrument(
        name = "pg_pool.test_connection",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn test_connection(&self) -> PgPoolResult<()> {
        let conn = self.pool.get().await.si_inspect_err(
            |err| warn!(si.error.message = %err, "failed to get test database connection from pool"),
        )?;
        debug!("connected to database");
        let row = conn
            .query_one(TEST_QUERY, &[])
            .await
            .si_inspect_err(|err| {
                warn!(
                    error = %err,
                    db.statement = &TEST_QUERY,
                    "failed to execute test query"
                )
            })?;
        let col: i32 = row.try_get(0).si_inspect_err(|err| {
            warn!(
                error = %err,
                db.statement = &TEST_QUERY,
                "failed to parse column 0 of row from test query result"
            )
        })?;
        if col != 1 {
            warn!("test query did not return expected value; expected=1, got={col}");
            return Err(PgPoolError::TestConnectionResult(1, col));
        }
        debug!("test connection successful");
        Ok(())
    }

    /// Gets the database name for connections in the pool.
    pub fn db_name(&self) -> &str {
        &self.metadata.db_name
    }

    /// Retrieve object from pool or wait for one to become available.
    #[instrument(
        name = "pg_pool.get",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = Empty,
            db.pool.size = Empty,
            db.pool.available = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn get(&self) -> PgPoolResult<InstrumentedClient> {
        let span = current_span_for_instrument_at!("debug");

        let pool_status = self.pool.status();
        span.record("db.pool.max_size", pool_status.max_size);
        span.record("db.pool.size", pool_status.size);
        span.record("db.pool.available", pool_status.available);

        let inner = self.pool.get().await?;

        Ok(InstrumentedClient {
            inner,
            metadata: self.metadata.clone(),
        })
    }

    #[instrument(
        name = "pg_pool.migrate",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn migrate(&self, runner: refinery::Runner) -> PgPoolResult<()> {
        let mut conn = self.pool.get().await?;
        conn.query_one("SELECT pg_advisory_lock($1)", &[&MIGRATION_LOCK_NUMBER])
            .await?;
        let client = &mut **conn;
        match runner.run_async(client).await {
            Ok(_) => {
                conn.query_one("SELECT pg_advisory_unlock($1)", &[&MIGRATION_LOCK_NUMBER])
                    .await?;
                Ok(())
            }
            Err(e) => {
                conn.query_one("SELECT pg_advisory_unlock($1)", &[&MIGRATION_LOCK_NUMBER])
                    .await?;
                Err(e.into())
            }
        }
    }

    #[instrument(
        name = "pg_pool.drop_and_create_public_schema",
        skip_all,
        level = "debug"
    )]
    pub async fn drop_and_create_public_schema(&self) -> PgPoolResult<()> {
        let conn = self.get().await?;
        conn.execute("DROP SCHEMA IF EXISTS public CASCADE", &[])
            .await?;
        conn.execute("CREATE SCHEMA public", &[]).await?;
        Ok(())
    }
}

// Ensure that we only grab the current span if we're at debug level or lower, otherwise use none.
//
// When recording a parent span for long running tasks such as a transaction we want the direct
// span parent. However, `Span::current()` returns a suitable parent span, according to the tracing
// `Subscriber`, meaning that instead of capturing the transaction starting span, we might capture
// a calling function up the stack that is at the info level or higher. In other words, then
// "transaction span" might be an ancestor span unless we're really careful.
macro_rules! current_span_for_debug {
    () => {
        Span::none()
    };
}

/// An instrumented wrapper for `deadpool::managed::Object<deadpool_postgres::Manager>`
pub struct InstrumentedClient {
    inner: Object<Manager>,
    metadata: Arc<ConnectionMetadata>,
}

impl InstrumentedClient {
    /// Like [`tokio_postgres::Transaction::prepare`](#method.prepare-1)
    /// but uses an existing statement from the cache if possible.
    #[instrument(
        name = "pg_client.prepare_cached",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, PgError> {
        self.inner.prepare_cached(query).await.map_err(Into::into)
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    #[instrument(
        name = "pg_client.prepare_typed_cached",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, PgError> {
        self.inner
            .prepare_typed_cached(query, types)
            .await
            .map_err(Into::into)
    }

    /// Begins a new database transaction.
    ///
    /// The transaction will roll back by default - use the `commit` method to commit it.
    #[instrument(
        name = "pg_client.transaction",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.transaction = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn transaction(&mut self) -> Result<InstrumentedTransaction<'_>, PgError> {
        Ok(InstrumentedTransaction::new(
            self.inner.transaction().await?,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Returns a builder for a transaction with custom settings.
    ///
    /// Unlike the `transaction` method, the builder can be used to control the transaction's
    /// isolation level and other
    /// attributes.
    pub fn build_transaction(&mut self) -> InstrumentedTransactionBuilder {
        InstrumentedTransactionBuilder {
            inner: self.inner.build_transaction(),
            metadata: self.metadata.clone(),
        }
    }

    /// Creates a new prepared statement.
    ///
    /// Prepared statements can be executed repeatedly, and may contain query parameters (indicated
    /// by `$1`, `$2`, etc), which are set when executed. Prepared statements can only be used with
    /// the connection that created them.
    #[instrument(
        name = "pg_client.prepare",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare(&self, query: &str) -> Result<Statement, PgError> {
        self.inner.prepare(query).await.map_err(Into::into)
    }

    /// Like `prepare`, but allows the types of query parameters to be explicitly specified.
    ///
    /// The list of types may be smaller than the number of parameters - the types of the remaining
    /// parameters will be inferred. For example, `client.prepare_typed(query, &[])` is equivalent
    /// to `client.prepare(query)`.
    #[instrument(
        name = "pg_client.prepare_typed",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_typed(
        &self,
        query: &str,
        parameter_types: &[Type],
    ) -> Result<Statement, PgError> {
        self.inner
            .prepare_typed(query, parameter_types)
            .await
            .map_err(Into::into)
    }

    /// Executes a statement, returning a vector of the resulting rows.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_client.query",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            db.rows = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<PgRow>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        let r = self
            .inner
            .query(statement, params)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|inner| PgRow { inner })
                    .collect::<Vec<_>>()
            })
            .map_err(Into::into);
        if let Ok(ref rows) = r {
            span.record("db.rows", rows.len());
        }
        r
    }

    /// Executes a statement which returns a single row, returning it.
    ///
    /// Returns an error if the query does not return exactly one row.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_client.query_one",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            db.rows = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_one(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<PgRow, PgError> {
        let span = current_span_for_instrument_at!("debug");

        let r = self
            .inner
            .query_one(statement, params)
            .await
            .map(|inner| PgRow { inner })
            .map_err(Into::into);
        if r.is_ok() {
            span.record("db.rows", 1);
        }
        r
    }

    /// Executes a statements which returns zero or one rows, returning it.
    ///
    /// Returns an error if the query returns more than one row.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_client.query_opt",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            db.rows = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_opt(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<PgRow>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        let r = self
            .inner
            .query_opt(statement, params)
            .await
            .map(|maybe| maybe.map(|inner| PgRow { inner }))
            .map_err(Into::into);
        if let Ok(ref maybe) = r {
            span.record(
                "db.rows",
                match maybe {
                    Some(_) => 1,
                    None => 0,
                },
            );
        }
        r
    }

    /// The maximally flexible version of [`query`].
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    ///
    /// [`query`]: #method.query
    #[instrument(
        name = "pg_client.query_raw",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_raw<P, I>(
        &self,
        statement: &str,
        params: I,
    ) -> Result<impl Stream<Item = Result<PgRow, PgError>> + use<P, I>, PgError>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner
            .query_raw(statement, params)
            .await
            .map(|row_stream| {
                row_stream
                    .map(|row_result| row_result.map(|inner| PgRow { inner }).map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Executes a statement, returning the number of rows modified.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// If the statement does not modify any rows (e.g. `SELECT`), 0 is returned.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_client.execute",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, PgError> {
        self.inner
            .execute(statement, params)
            .await
            .map_err(Into::into)
    }

    /// The maximally flexible version of [`execute`].
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    ///
    /// [`execute`]: #method.execute
    #[instrument(
        name = "pg_client.execute_raw",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn execute_raw<P, I>(&self, statement: &str, params: I) -> Result<u64, PgError>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner
            .execute_raw(statement, params)
            .await
            .map_err(Into::into)
    }

    /// Executes a `COPY FROM STDIN` statement, returning a sink used to write the copy data.
    ///
    /// PostgreSQL does not support parameters in `COPY` statements, so this method does not take
    /// any. The copy *must* be explicitly completed via the `Sink::close` or `finish` methods. If
    /// it is not, the copy will be aborted.
    ///
    /// # Panics
    ///
    /// Panics if the statement contains parameters.
    #[instrument(
        name = "pg_client.copy_in",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_in<T, U>(&self, statement: &T) -> Result<CopyInSink<U>, PgError>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        self.inner.copy_in(statement).await.map_err(Into::into)
    }

    /// Executes a `COPY TO STDOUT` statement, returning a stream of the resulting data.
    ///
    /// PostgreSQL does not support parameters in `COPY` statements, so this method does not take
    /// any.
    ///
    /// # Panics
    ///
    /// Panics if the statement contains parameters.
    #[instrument(
        name = "pg_client.copy_out",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_out<T>(&self, statement: &T) -> Result<CopyOutStream, PgError>
    where
        T: ?Sized + ToStatement,
    {
        self.inner.copy_out(statement).await.map_err(Into::into)
    }

    /// Executes a sequence of SQL statements using the simple query protocol, returning the
    /// resulting rows.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the sequence
    /// will stop at that point. The simple query protocol returns the values in rows as strings
    /// rather than in their binary encodings, so the associated row type doesn't work with the
    /// `FromSql` trait. Rather than simply returning a list of the rows, this method returns a
    /// list of an enum which indicates either the completion of one of the commands, or a row of
    /// data. This preserves the framing between the separate statements in the request.
    ///
    /// # Warning
    ///
    /// Prepared statements should be use for any query which contains user-specified data, as they
    /// provided the functionality to safely embed that data in the request. Do not form statements
    /// via string concatenation and pass them to this method!
    #[instrument(
        name = "pg_client.simple_query",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, PgError> {
        self.inner.simple_query(query).await.map_err(Into::into)
    }

    /// Executes a sequence of SQL statements using the simple query protocol.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the sequence
    /// will stop at that point. This is intended for use when, for example, initializing a
    /// database schema.
    ///
    /// # Warning
    ///
    /// Prepared statements should be use for any query which contains user-specified data, as they
    /// provided the functionality to safely embed that data in the request. Do not form statements
    /// via string concatenation and pass them to this method!
    #[instrument(
        name = "pg_client.batch_execute",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn batch_execute(&self, query: &str) -> Result<(), PgError> {
        self.inner.batch_execute(query).await.map_err(Into::into)
    }

    /// Constructs a cancellation token that can later be used to request cancellation of a query
    /// running on the connection associated with this client.
    pub fn cancel_token(&self) -> CancelToken {
        self.inner.cancel_token()
    }

    /// Clears the client's type information cache.
    ///
    /// When user-defined types are used in a query, the client loads their definitions from the
    /// database and caches them for the lifetime of the client. If those definitions are changed
    /// in the database, this method can be used to flush the local cache and allow the new,
    /// updated definitions to be loaded.
    #[instrument(
        name = "pg_client.clear_type_cache",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn clear_type_cache(&self) {
        self.inner.clear_type_cache();
    }

    /// Determines if the connection to the server has already closed.
    ///
    /// In that case, all future queries will fail.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl fmt::Debug for InstrumentedClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedClient")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

pub struct InstrumentedTransaction<'a> {
    inner: Transaction<'a>,
    metadata: Arc<ConnectionMetadata>,
    tx_span: Span,
}

impl<'a> InstrumentedTransaction<'a> {
    fn new(inner: Transaction<'a>, metadata: Arc<ConnectionMetadata>, tx_span: Span) -> Self {
        Self {
            inner,
            metadata,
            tx_span,
        }
    }

    /// Like [`tokio_postgres::Transaction::prepare`](#method.prepare-1)
    /// but uses an existing statement from the cache if possible.
    #[instrument(
        name = "pg_transaction.prepare_cached",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .prepare_cached(query)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    #[instrument(
        name = "pg_transaction.prepare_typed_cached",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .prepare_typed_cached(query, types)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Consumes the transaction, committing all changes made within it.
    #[instrument(
        name = "pg_transaction.commit",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn commit(self) -> Result<(), PgError> {
        let span = current_span_for_instrument_at!("debug");

        let _ = &self;
        span.follows_from(&self.tx_span);

        let r = self
            .inner
            .commit()
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into);
        self.tx_span.record("db.transaction", "commit");
        r
    }

    /// Rolls the transaction back, discarding all changes made within it.
    ///
    /// This is equivalent to `Transaction`'s `Drop` implementation, but provides any error
    /// encountered to the caller.
    #[instrument(
        name = "pg_transaction.rollback",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn rollback(self) -> Result<(), PgError> {
        let span = current_span_for_instrument_at!("debug");

        let _ = &self;
        span.follows_from(&self.tx_span);

        let r = self
            .inner
            .rollback()
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into);
        self.tx_span.record("db.transaction", "rollback");
        r
    }

    /// Creates a new prepared statement.
    ///
    /// Prepared statements can be executed repeatedly, and may contain query parameters (indicated
    /// by `$1`, `$2`, etc), which are set when executed. Prepared statements can only be used with
    /// the connection that created them.
    #[instrument(
        name = "pg_transaction.prepare",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare(&self, query: &str) -> Result<Statement, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .prepare(query)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Like `prepare`, but allows the types of query parameters to be explicitly specified.
    ///
    /// The list of types may be smaller than the number of parameters - the types of the remaining
    /// parameters will be inferred. For example, `client.prepare_typed(query, &[])` is equivalent
    /// to `client.prepare(query)`.
    #[instrument(
        name = "pg_transaction.prepare_typed",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_typed(
        &self,
        query: &str,
        parameter_types: &[Type],
    ) -> Result<Statement, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .prepare_typed(query, parameter_types)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Executes a statement, returning a vector of the resulting rows.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_transaction.query",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            db.rows = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<PgRow>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        let r = self
            .inner
            .query(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|inner| PgRow { inner })
                    .collect::<Vec<_>>()
            })
            .map_err(Into::into);
        if let Ok(ref rows) = r {
            span.record("db.rows", rows.len());
        }
        r
    }

    /// Executes a statement which returns a single row, returning it.
    ///
    /// Returns an error if the query does not return exactly one row.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_transaction.query_one",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            db.rows = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_one(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<PgRow, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        let r = self
            .inner
            .query_one(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map(|inner| PgRow { inner })
            .map_err(Into::into);
        if r.is_ok() {
            span.record("db.rows", 1);
        }
        r
    }

    /// Executes a statements which returns zero or one rows, returning it.
    ///
    /// Returns an error if the query returns more than one row.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_transaction.query_opt",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            db.rows = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_opt(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<PgRow>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        let r = self
            .inner
            .query_opt(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map(|maybe| maybe.map(|inner| PgRow { inner }))
            .map_err(Into::into);
        if let Ok(ref maybe) = r {
            span.record(
                "db.rows",
                match maybe {
                    Some(_) => 1,
                    None => 0,
                },
            );
        }
        r
    }

    /// The maximally flexible version of [`query`].
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    ///
    /// [`query`]: #method.query
    #[instrument(
        name = "pg_transaction.query_raw",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_raw<P, I>(
        &self,
        statement: &str,
        params: I,
    ) -> Result<impl Stream<Item = Result<PgRow, PgError>> + use<P, I>, PgError>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .query_raw(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map(|row_stream| {
                row_stream
                    .map(|row_result| row_result.map(|inner| PgRow { inner }).map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Executes a statement, returning the number of rows modified.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// If the statement does not modify any rows (e.g. `SELECT`), 0 is returned.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_transaction.execute",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .execute(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// The maximally flexible version of [`execute`].
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    ///
    /// [`execute`]: #method.execute
    #[instrument(
        name = "pg_transaction.execute_raw",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn execute_raw<P, I>(&self, statement: &str, params: I) -> Result<u64, PgError>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .execute_raw(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Binds a statement to a set of parameters, creating a `Portal` which can be incrementally
    /// queried.
    ///
    /// Portals only last for the duration of the transaction in which they are created, and can
    /// only be used on the connection that created them.
    ///
    /// # Panics
    ///
    /// Panics if the number of parameters provided does not match the number expected.
    #[instrument(
        name = "pg_transaction.bind",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn bind<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Portal, PgError>
    where
        T: ?Sized + ToStatement,
    {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .bind(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// A maximally flexible version of [`bind`].
    ///
    /// [`bind`]: #method.bind
    #[instrument(
        name = "pg_transaction.bind_raw",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn bind_raw<P, T, I>(&self, statement: &T, params: I) -> Result<Portal, PgError>
    where
        T: ?Sized + ToStatement,
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .bind_raw(statement, params)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Continues execution of a portal, returning a stream of the resulting rows.
    ///
    /// Unlike `query`, portals can be incrementally evaluated by limiting the number of rows
    /// returned in each call to `query_portal`. If the requested number is negative or 0, all rows
    /// will be returned.
    #[instrument(
        name = "pg_transaction.query_portal",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_portal(
        &self,
        portal: &Portal,
        max_rows: i32,
    ) -> Result<Vec<PgRow>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .query_portal(portal, max_rows)
            .instrument(self.tx_span.clone())
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|inner| PgRow { inner })
                    .collect::<Vec<_>>()
            })
            .map_err(Into::into)
    }

    /// The maximally flexible version of [`query_portal`].
    ///
    /// [`query_portal`]: #method.query_portal
    #[instrument(
        name = "pg_transaction.query_portal_raw",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_portal_raw(
        &self,
        portal: &Portal,
        max_rows: i32,
    ) -> Result<impl Stream<Item = Result<PgRow, PgError>> + use<>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .query_portal_raw(portal, max_rows)
            .instrument(self.tx_span.clone())
            .await
            .map(|row_stream| {
                row_stream
                    .map(|row_result| row_result.map(|inner| PgRow { inner }).map_err(Into::into))
            })
            .map_err(Into::into)
    }

    /// Executes a `COPY FROM STDIN` statement, returning a sink used to write the copy data.
    ///
    /// PostgreSQL does not support parameters in `COPY` statements, so this method does not take
    /// any. The copy *must* be explicitly completed via the `Sink::close` or `finish` methods. If
    /// it is not, the copy will be aborted.
    ///
    /// # Panics
    ///
    /// Panics if the statement contains parameters.
    #[instrument(
        name = "pg_transaction.copy_in",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_in<T, U>(&self, statement: &T) -> Result<CopyInSink<U>, PgError>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .copy_in(statement)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Executes a `COPY TO STDOUT` statement, returning a stream of the resulting data.
    ///
    /// PostgreSQL does not support parameters in `COPY` statements, so this method does not take
    /// any.
    ///
    /// # Panics
    ///
    /// Panics if the statement contains parameters.
    #[instrument(
        name = "pg_transaction.copy_out",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_out<T>(&self, statement: &T) -> Result<CopyOutStream, PgError>
    where
        T: ?Sized + ToStatement,
    {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .copy_out(statement)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Executes a sequence of SQL statements using the simple query protocol, returning the
    /// resulting rows.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the sequence
    /// will stop at that point. The simple query protocol returns the values in rows as strings
    /// rather than in their binary encodings, so the associated row type doesn't work with the
    /// `FromSql` trait. Rather than simply returning a list of the rows, this method returns a
    /// list of an enum which indicates either the completion of one of the commands, or a row of
    /// data. This preserves the framing between the separate statements in the request.
    ///
    /// # Warning
    ///
    /// Prepared statements should be use for any query which contains user-specified data, as they
    /// provided the functionality to safely embed that data in the request. Do not form statements
    /// via string concatenation and pass them to this method!
    #[instrument(
        name = "pg_transaction.simple_query",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .simple_query(query)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Executes a sequence of SQL statements using the simple query protocol.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the sequence
    /// will stop at that point. This is intended for use when, for example, initializing a
    /// database schema.
    ///
    /// # Warning
    ///
    /// Prepared statements should be use for any query which contains user-specified data, as they
    /// provided the functionality to safely embed that data in the request. Do not form statements
    /// via string concatenation and pass them to this method!
    #[instrument(
        name = "pg_transaction.batch_execute",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn batch_execute(&self, query: &str) -> Result<(), PgError> {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.inner
            .batch_execute(query)
            .instrument(self.tx_span.clone())
            .await
            .map_err(Into::into)
    }

    /// Constructs a cancellation token that can later be used to request cancellation of a query
    /// running on the connection associated with this client.
    #[instrument(
        name = "pg_transaction.cancel_token",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn cancel_token(&self) -> CancelToken {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.tx_span.in_scope(|| self.inner.cancel_token())
    }

    /// Like `Client::transaction`, but creates a nested transaction via a savepoint.
    #[instrument(
        name = "pg_transaction.transaction",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            db.transaction = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn transaction(&mut self) -> Result<InstrumentedTransaction<'_>, PgError> {
        Ok(InstrumentedTransaction::new(
            self.inner
                .transaction()
                .instrument(self.tx_span.clone())
                .await?,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Like `Client::transaction`, but creates a nested transaction via a savepoint with the
    /// specified name.
    #[instrument(
        name = "pg_transaction.savepoint",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn savepoint<I>(&mut self, name: I) -> Result<InstrumentedTransaction<'_>, PgError>
    where
        I: Into<String>,
    {
        Ok(InstrumentedTransaction::new(
            self.inner
                .savepoint(name)
                .instrument(self.tx_span.clone())
                .await?,
            self.metadata.clone(),
            current_span_for_debug!(),
        ))
    }

    /// Returns a mutex-guarded reference to the underlying `Client`.
    #[instrument(
        name = "pg_transaction.client",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn client(&'a self) -> &'a Client {
        let span = current_span_for_instrument_at!("debug");

        span.follows_from(&self.tx_span);
        self.tx_span.in_scope(|| self.inner.client())
    }
}

impl fmt::Debug for InstrumentedTransaction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedTransaction")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

/// A row of data returned from the database by a query.
#[derive(Debug)]
pub struct PgRow {
    inner: Row,
}

impl PgRow {
    /// Returns information about the columns of data in the row.
    pub fn columns(&self) -> &[Column] {
        self.inner.columns()
    }

    /// Determines if the row contains no values.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of values in the row.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Deserializes a value from the row.
    ///
    /// The value can be specified either by its numeric index in the row, or by its column name.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds or if the value cannot be converted to the specified type.
    pub fn get<'a, I, T>(&'a self, idx: I) -> T
    where
        I: RowIndex + fmt::Display,
        T: FromSql<'a>,
    {
        self.inner.get(idx)
    }

    /// Like `Row::get`, but returns a `Result` rather than panicking.
    pub fn try_get<'a, I, T>(&'a self, idx: I) -> Result<T, PgError>
    where
        I: RowIndex + fmt::Display,
        T: FromSql<'a>,
    {
        self.inner.try_get(idx).map_err(Into::into)
    }
}

pub struct InstrumentedTransactionBuilder<'a> {
    inner: TransactionBuilder<'a>,
    metadata: Arc<ConnectionMetadata>,
}

impl<'a> InstrumentedTransactionBuilder<'a> {
    /// Sets the isolation level of the transaction.
    ///
    /// Like `tokio_postgres::TransactionBuilder::isolation_level`
    pub fn isolation_level(self, isolation_level: IsolationLevel) -> Self {
        Self {
            inner: self.inner.isolation_level(isolation_level),
            metadata: self.metadata,
        }
    }

    /// Sets the access mode of the transaction.
    ///
    /// Like `tokio_postgres::TransactionBuilder::read_only`
    pub fn read_only(self, read_only: bool) -> Self {
        Self {
            inner: self.inner.read_only(read_only),
            metadata: self.metadata,
        }
    }

    /// Sets the deferrability of the transaction.
    ///
    /// If the transaction is also serializable and read only, creation of the transaction may
    /// block, but when it completes the transaction is able to run with less overhead and a
    /// guarantee that it will not be aborted due to serialization failure.
    ///
    /// Like `tokio_postgres::TransactionBuilder::deferrable`
    pub fn deferrable(self, deferrable: bool) -> Self {
        Self {
            inner: self.inner.deferrable(deferrable),
            metadata: self.metadata,
        }
    }

    /// Begins the transaction.
    ///
    /// The transaction will roll back by default - use the commit method
    /// to commit it.
    ///
    /// Like `tokio_postgres::TransactionBuilder::start`
    #[instrument(
        name = "pg_transaction_builder.start",
        skip_all,
        level = "debug",
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.pool.max_size = %self.metadata.db_pool_max_size,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn start(self) -> Result<InstrumentedTransaction<'a>, PgError> {
        Ok(InstrumentedTransaction::new(
            self.inner.start().await?,
            self.metadata,
            current_span_for_debug!(),
        ))
    }
}

impl fmt::Debug for InstrumentedTransactionBuilder<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedTransactionBuilder")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}

/// A PostgreSQL transaction, backed by a client connection that can be shared by cloning for
/// concurrent access.
#[derive(Clone)]
pub struct PgSharedTransaction {
    inner: Arc<Mutex<PgOwnedTransaction>>,
    metadata: Arc<ConnectionMetadata>,
}

impl PgSharedTransaction {
    pub async fn create(pg_conn: InstrumentedClient) -> Result<Self, PgError> {
        let metadata = pg_conn.metadata.clone();
        let inner = PgOwnedTransaction::create(pg_conn).await?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            metadata,
        })
    }

    /// Consumes the transaction, committing all changes made within it, and returns the underlying
    /// client connection for reuse.
    ///
    /// # Runtime Safety Notes
    ///
    /// The only reasonable time to commit a transaction is when there are no other shared copies
    /// of the transaction active in the system. This is because a commit will consume `self` and
    /// therefore the internal locking mechanism needs to assert that there are no other potential
    /// copies that could race for a lock.
    ///
    /// The implication is that if there are other clones of the `Transaction` instance, then a
    /// runtime error of `PgError::TxnCommitNotExclusive` will be returned and the transaction will
    /// remain in an un-committed state (that is not committed and also not rolled back). It is the
    /// responsibility of the caller to ensure that there are no more copies before `commit` is
    /// called meaning that it's *highly likely* that `PgError::TxnCommitNotExclusive` represents a
    /// programmer error rather than a transient failure.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn commit_into_conn(self) -> Result<InstrumentedClient, PgError> {
        let mut owned_txn = Arc::try_unwrap(self.inner)
            .map_err(|arc| PgError::TxnCommitNotExclusive(Arc::strong_count(&arc)))?
            .into_inner();
        #[allow(clippy::expect_used)] // This expect could also be expressed with an `unreachable!`
        owned_txn
            .with_mut(|inner| inner.txn.take())
            .expect("txn is only consumed once with commit/rollback--this is an internal bug")
            .commit()
            .await?;
        let conn = owned_txn.into_heads().conn;

        Ok(conn)
    }

    /// Rolls the transaction back, discarding all changes made within it, and returns the
    /// underlying client connection for resuse.
    ///
    /// This is equivalent to `Transaction`'s `Drop` implementation, but provides any error
    /// encountered to the caller.
    ///
    /// # Runtime Safety Notes
    ///
    /// The only reasonable time to roll back a transaction is when there are no other shared
    /// copies of the transaction active in the system. This is because a rollback will consume
    /// `self` and therefore the internal locking mechanism needs to assert that there are no other
    /// potential copies that could race for a lock.
    //
    /// The implication is that if there are other clones of the `Transaction` instance, then a
    /// runtime error of `PgError::TxnRollbackNotExclusive` will be returned and the transaction
    /// will remain in an un-rolledback state (that is not committed and also not rolled back). It
    /// is the responsibility of the caller to ensure that there are no more copies before
    /// `rollback` is called meaning that it's *highly likely* that
    /// `PgError::TxnRollbackNotExclusive` represents a programmer error rather than a transient
    /// failure.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn rollback_into_conn(self) -> Result<InstrumentedClient, PgError> {
        let mut owned_txn = Arc::try_unwrap(self.inner)
            .map_err(|arc| PgError::TxnRollbackNotExclusive(Arc::strong_count(&arc)))?
            .into_inner();
        #[allow(clippy::expect_used)] // This expect could also be expressed with an `unreachable!`
        owned_txn
            .with_mut(|inner| inner.txn.take())
            .expect("txn is only consumed once with commit/rollback--this is an internal bug")
            .rollback()
            .await?;
        let conn = owned_txn.into_heads().conn;

        Ok(conn)
    }

    /// Like [`tokio_postgres::Transaction::prepare`](#method.prepare-1)
    /// but uses an existing statement from the cache if possible.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.prepare_cached(query).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn prepare_typed_cached(
        &self,
        query: &str,
        types: &[Type],
    ) -> Result<Statement, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.prepare_typed_cached(query, types).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Consumes the transaction, committing all changes made within it.
    ///
    /// # Runtime Safety Notes
    ///
    /// The only reasonable time to commit a transaction is when there are no other shared copies
    /// of the transaction active in the system. This is because a commit will consume `self` and
    /// therefore the internal locking mechanism needs to assert that there are no other potential
    /// copies that could race for a lock.
    ///
    /// The implication is that if there are other clones of the `Transaction` instance, then a
    /// runtime error of `PgError::TxnCommitNotExclusive` will be returned and the transaction will
    /// remain in an un-committed state (that is not committed and also not rolled back). It is the
    /// responsibility of the caller to ensure that there are no more copies before `commit` is
    /// called meaning that it's *highly likely* that `PgError::TxnCommitNotExclusive` represents a
    /// programmer error rather than a transient failure.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn commit(self) -> Result<(), PgError> {
        let _ = self.commit_into_conn().await?;
        Ok(())
    }

    /// Rolls the transaction back, discarding all changes made within it.
    ///
    /// This is equivalent to `Transaction`'s `Drop` implementation, but provides any error
    /// encountered to the caller.
    ///
    /// # Runtime Safety Notes
    ///
    /// The only reasonable time to roll back a transaction is when there are no other shared
    /// copies of the transaction active in the system. This is because a rollback will consume
    /// `self` and therefore the internal locking mechanism needs to assert that there are no other
    /// potential copies that could race for a lock.
    //
    /// The implication is that if there are other clones of the `Transaction` instance, then a
    /// runtime error of `PgError::TxnRollbackNotExclusive` will be returned and the transaction
    /// will remain in an un-rolledback state (that is not committed and also not rolled back). It
    /// is the responsibility of the caller to ensure that there are no more copies before
    /// `rollback` is called meaning that it's *highly likely* that
    /// `PgError::TxnRollbackNotExclusive` represents a programmer error rather than a transient
    /// failure.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn rollback(self) -> Result<(), PgError> {
        let _ = self.rollback_into_conn().await?;
        Ok(())
    }

    /// Creates a new prepared statement.
    ///
    /// Prepared statements can be executed repeatedly, and may contain query parameters (indicated
    /// by `$1`, `$2`, etc), which are set when executed. Prepared statements can only be used with
    /// the connection that created them.
    pub async fn prepare(&self, query: &str) -> Result<Statement, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.prepare(query).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Like `prepare`, but allows the types of query parameters to be explicitly specified.
    ///
    /// The list of types may be smaller than the number of parameters - the types of the remaining
    /// parameters will be inferred. For example, `client.prepare_typed(query, &[])` is equivalent
    /// to `client.prepare(query)`.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn prepare_typed(
        &self,
        query: &str,
        parameter_types: &[Type],
    ) -> Result<Statement, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.prepare_typed(query, parameter_types).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a statement, returning a vector of the resulting rows.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn query(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Vec<PgRow>, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.query(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a statement which returns a single row, returning it.
    ///
    /// Returns an error if the query does not return exactly one row.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn query_one(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<PgRow, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.query_one(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a statements which returns zero or one rows, returning it.
    ///
    /// Returns an error if the query returns more than one row.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn query_opt(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Option<PgRow>, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.query_opt(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a statement that returns zero rows.
    ///
    /// Returns an error if the query returns more than zero rows.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn query_none(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<(), PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => match txn.query_opt(statement, params).await? {
                None => Ok(()),
                Some(row) => Err(PgError::UnexpectedRow(Box::new(row))),
            },
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// The maximally flexible version of [`query`].
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    ///
    /// [`query`]: #method.query
    pub async fn query_raw<P, I>(
        &self,
        statement: &str,
        params: I,
    ) -> Result<impl Stream<Item = Result<PgRow, PgError>> + use<P, I>, PgError>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.query_raw(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a statement, returning the number of rows modified.
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// If the statement does not modify any rows (e.g. `SELECT`), 0 is returned.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn execute(
        &self,
        statement: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<u64, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.execute(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// The maximally flexible version of [`execute`].
    ///
    /// A statement may contain parameters, specified by `$n`, where `n` is the index of the
    /// parameter of the list provided, 1-indexed.
    ///
    /// The `statement` argument can either be a `Statement`, or a raw query string. If the same
    /// statement will be repeatedly executed (perhaps with different query parameters), consider
    /// preparing the statement up front with the `prepare` method.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    ///
    /// [`execute`]: #method.execute
    pub async fn execute_raw<P, I>(&self, statement: &str, params: I) -> Result<u64, PgError>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.execute_raw(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Binds a statement to a set of parameters, creating a `Portal` which can be incrementally
    /// queried.
    ///
    /// Portals only last for the duration of the transaction in which they are created, and can
    /// only be used on the connection that created them.
    ///
    /// # Panics
    ///
    /// - If the number of parameters provided does not match the number expected.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn bind<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Portal, PgError>
    where
        T: ?Sized + ToStatement,
    {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.bind(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// A maximally flexible version of [`bind`].
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    ///
    /// [`bind`]: #method.bind
    pub async fn bind_raw<P, T, I>(&self, statement: &T, params: I) -> Result<Portal, PgError>
    where
        T: ?Sized + ToStatement,
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.bind_raw(statement, params).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Continues execution of a portal, returning a stream of the resulting rows.
    ///
    /// Unlike `query`, portals can be incrementally evaluated by limiting the number of rows
    /// returned in each call to `query_portal`. If the requested number is negative or 0, all rows
    /// will be returned.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn query_portal(
        &self,
        portal: &Portal,
        max_rows: i32,
    ) -> Result<Vec<PgRow>, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.query_portal(portal, max_rows).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// The maximally flexible version of [`query_portal`].
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    ///
    /// [`query_portal`]: #method.query_portal
    pub async fn query_portal_raw(
        &self,
        portal: &Portal,
        max_rows: i32,
    ) -> Result<impl Stream<Item = Result<PgRow, PgError>> + use<>, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.query_portal_raw(portal, max_rows).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a `COPY FROM STDIN` statement, returning a sink used to write the copy data.
    ///
    /// PostgreSQL does not support parameters in `COPY` statements, so this method does not take
    /// any. The copy *must* be explicitly completed via the `Sink::close` or `finish` methods. If
    /// it is not, the copy will be aborted.
    ///
    /// # Panics
    ///
    /// - If the statement contains parameters.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn copy_in<T, U>(&self, statement: &T) -> Result<CopyInSink<U>, PgError>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.copy_in(statement).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a `COPY TO STDOUT` statement, returning a stream of the resulting data.
    ///
    /// PostgreSQL does not support parameters in `COPY` statements, so this method does not take
    /// any.
    ///
    /// # Panics
    ///
    /// - If the statement contains parameters.
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn copy_out<T>(&self, statement: &T) -> Result<CopyOutStream, PgError>
    where
        T: ?Sized + ToStatement,
    {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.copy_out(statement).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a sequence of SQL statements using the simple query protocol, returning the
    /// resulting rows.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the sequence
    /// will stop at that point. The simple query protocol returns the values in rows as strings
    /// rather than in their binary encodings, so the associated row type doesn't work with the
    /// `FromSql` trait. Rather than simply returning a list of the rows, this method returns a
    /// list of an enum which indicates either the completion of one of the commands, or a row of
    /// data. This preserves the framing between the separate statements in the request.
    ///
    /// # Warning
    ///
    /// Prepared statements should be use for any query which contains user-specified data, as they
    /// provided the functionality to safely embed that data in the request. Do not form statements
    /// via string concatenation and pass them to this method!
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.simple_query(query).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Executes a sequence of SQL statements using the simple query protocol.
    ///
    /// Statements should be separated by semicolons. If an error occurs, execution of the sequence
    /// will stop at that point. This is intended for use when, for example, initializing a
    /// database schema.
    ///
    /// # Warning
    ///
    /// Prepared statements should be use for any query which contains user-specified data, as they
    /// provided the functionality to safely embed that data in the request. Do not form statements
    /// via string concatenation and pass them to this method!
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn batch_execute(&self, query: &str) -> Result<(), PgError> {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.batch_execute(query).await,
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }

    /// Constructs a cancellation token that can later be used to request cancellation of a query
    /// running on the connection associated with this client.
    ///
    /// # Panics
    ///
    /// - If the internal transaction has already been consumed which is an internal correctness
    ///   bug
    pub async fn cancel_token(&self) -> CancelToken {
        match self.inner.lock().await.borrow_txn().as_ref() {
            Some(txn) => txn.cancel_token(),
            None => {
                unreachable!("txn is only consumed with commit/rollback--this is an internal bug")
            }
        }
    }
}

impl fmt::Debug for PgSharedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgSharedTransaction")
            .field("metadata", &self.metadata)
            .field("strong_count", &Arc::strong_count(&self.inner))
            .finish_non_exhaustive()
    }
}

#[self_referencing]
struct PgOwnedTransaction {
    conn: InstrumentedClient,
    #[borrows(mut conn)]
    #[covariant]
    txn: Option<InstrumentedTransaction<'this>>,
}

impl Debug for PgOwnedTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PgOwnedTransaction").finish_non_exhaustive()
    }
}

impl PgOwnedTransaction {
    async fn create(pg_conn: InstrumentedClient) -> Result<Self, PgError> {
        PgOwnedTransactionAsyncSendTryBuilder {
            conn: pg_conn,
            txn_builder: |pg_conn| {
                Box::pin(async move { Some(pg_conn.transaction().await).transpose() })
            },
        }
        .try_build()
        .await
    }
}

async fn test_connection_infallible_and_warm_up_pool_task(check_pool: PgPool) {
    let _result = check_pool.test_connection().await;
}

async fn evict_old_and_idle_pgpool_connections(
    pg_pool: Pool,
    check_interval: Duration,
    max_age: Duration,
    max_idle_time: Duration,
) {
    // Since the pool.retain we use in the loop below blocks the pool, we set a minimum
    // interval to avoid it from completely breaking db connections
    let min_check_interval = Duration::from_secs(30);

    let check_interval = if check_interval < min_check_interval {
        warn!(
            "pgpool configured with a too small eviction checking interval, setting it to min value ({} seconds)",
            min_check_interval.as_secs()
        );
        min_check_interval
    } else {
        check_interval
    };

    loop {
        tokio::time::sleep(check_interval).await;
        let connection_count = Rc::new(RefCell::new(0));
        let evicted_connections = Rc::new(RefCell::new(0));
        let oldest_used = Rc::new(RefCell::new(Duration::default()));
        let oldest_created = Rc::new(RefCell::new(Duration::default()));

        info!("Locking pool for eviction loop");
        pg_pool.retain(|_, metrics| {
            *connection_count.borrow_mut() += 1;
            if *oldest_used.borrow() < metrics.last_used() {
                *oldest_used.borrow_mut() = metrics.last_used()
            }
            if *oldest_created.borrow() < metrics.age() {
                *oldest_created.borrow_mut() = metrics.age()
            }
            if metrics.last_used() > max_idle_time {
                debug!("Evicting connection due to idle time");
                *evicted_connections.borrow_mut() += 1;
                false
            } else if metrics.age() > max_age {
                debug!("Evicting connection due to total age");
                *evicted_connections.borrow_mut() += 1;
                false
            } else {
                true
            }
        });
        info!(
            "Checked {} connection(s), evicted {}, oldest {:?}, least recently used {:?}",
            connection_count.borrow(),
            evicted_connections.borrow(),
            oldest_created.borrow(),
            oldest_used.borrow(),
        );
    }
}
