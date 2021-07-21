use bytes::Buf;
use deadpool::managed::Object;
use deadpool_postgres::{
    config::ConfigError, Config, Manager, ManagerConfig, Pool, PoolError, RecyclingMethod,
    Transaction, TransactionBuilder,
};
use std::{fmt, net::ToSocketAddrs, ops::DerefMut, sync::Arc};
use tokio_postgres::{
    types::{BorrowToSql, ToSql, Type},
    CancelToken, Client, CopyInSink, CopyOutStream, IsolationLevel, NoTls, Portal, Row, RowStream,
    SimpleQueryMessage, Statement, ToStatement,
};
use tracing::{
    field::{display, Empty},
    instrument, Instrument, Span,
};

pub use tokio_postgres::{error::SqlState, Error};

const MIGRATION_LOCK_NUMBER: i64 = 42;

#[derive(thiserror::Error, Debug)]
pub enum PgPoolError {
    #[error("pg pool config error: {0}")]
    DeadpoolConfig(#[from] ConfigError),
    #[error("pg pool error: {0}")]
    PoolError(#[from] PoolError),
    #[error("migration error: {0}")]
    Refinery(#[from] refinery::Error),
    #[error("tokio pg error: {0}")]
    TokioPg(#[from] tokio_postgres::Error),
    #[error("tokio task join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
    #[error("failed to resolve pg hostname")]
    ResolveHostname(std::io::Error),
    #[error("resolved hostname returned no entries")]
    ResolveHostnameNoEntries,
}

pub type PgPoolResult<T> = Result<T, PgPoolError>;
pub type PgTxn<'a> = InstrumentedTransaction<'a>;

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
    net_peer_ip: String,
    net_peer_port: u16,
    net_transport: &'static str,
}

impl PgPool {
    #[instrument(
        name = "pgpool.new",
        skip(settings),
        fields(
            db.system = Empty,
            db.connection_string = Empty,
            db.name = Empty,
            db.user = Empty,
            net.peer.ip = Empty,
            net.peer.port = Empty,
            net.transport = Empty,
        )
    )]
    pub async fn new(settings: &si_settings::Pg) -> PgPoolResult<Self> {
        let mut cfg = Config::new();
        cfg.hosts = Some(vec![settings.hostname.clone()]);
        cfg.port = Some(settings.port.clone());
        cfg.user = Some(settings.user.clone());
        cfg.password = Some(settings.password.clone());
        cfg.dbname = Some(settings.dbname.clone());
        cfg.application_name = Some(settings.application_name.clone());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        let pool = cfg.create_pool(NoTls)?;

        let resolving_hostname = format!("{}:{}", settings.hostname, settings.port);
        let net_peer_ip = tokio::task::spawn_blocking(move || {
            resolving_hostname
                .to_socket_addrs()
                .map_err(PgPoolError::ResolveHostname)
                .and_then(|mut iter| iter.next().ok_or(PgPoolError::ResolveHostnameNoEntries))
                .and_then(|socket_addr| Ok(socket_addr.ip().to_string()))
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
            net_peer_ip,
            net_peer_port: settings.port,
            net_transport: "ip_tcp",
        };

        let span = Span::current();
        span.record("db.system", &metadata.db_system);
        span.record(
            "db.connection_string",
            &metadata.db_connection_string.as_str(),
        );
        span.record("db.name", &metadata.db_name.as_str());
        span.record("db.user", &metadata.db_user.as_str());
        span.record("net.peer.ip", &metadata.net_peer_ip.as_str());
        span.record("net.peer.port", &metadata.net_peer_port);
        span.record("net.transport", &metadata.net_transport);

        // Warm up the pool and ensure that we can connect to the database. In practice, this pool
        // only gets created on a service start so this is a one-time cost with a nice fail-fast
        // approach.
        pool.get().await?;

        Ok(Self {
            pool,
            metadata: Arc::new(metadata),
        })
    }

    /// Retrieve object from pool or wait for one to become available.
    #[instrument(
        name = "pgpool.get",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn get(&self) -> PgPoolResult<InstrumentedClient> {
        let inner = self.pool.get().await?;

        Ok(InstrumentedClient {
            inner,
            metadata: self.metadata.clone(),
        })
    }

    #[instrument(
        name = "pgpool.migrate",
        skip(self, runner),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn migrate(&self, runner: refinery::Runner) -> PgPoolResult<()> {
        let mut conn = self.pool.get().await?;
        conn.query_one("SELECT pg_advisory_lock($1)", &[&MIGRATION_LOCK_NUMBER])
            .await?;
        let client = conn.deref_mut().deref_mut();
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

    #[instrument(skip(self))]
    pub async fn drop_and_create_public_schema(&self) -> PgPoolResult<()> {
        let conn = self.get().await?;
        conn.execute("DROP SCHEMA IF EXISTS public CASCADE", &[])
            .await?;
        conn.execute("CREATE SCHEMA public", &[]).await?;
        Ok(())
    }
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
        name = "instrumentedclient.prepare_cached",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, Error> {
        self.inner.prepare_cached(query).await
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    #[instrument(
        name = "instrumentedclient.prepare_typed_cached",
        skip(self, query, types),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Statement, tokio_postgres::Error> {
        self.inner.prepare_typed_cached(query, types).await
    }

    /// Begins a new database transaction.
    ///
    /// The transaction will roll back by default - use the `commit` method to commit it.
    #[instrument(
        name = "instrumentedclient.transaction",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.transaction = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn transaction(&mut self) -> Result<InstrumentedTransaction<'_>, Error> {
        Ok(InstrumentedTransaction::new(
            self.inner.transaction().await?,
            self.metadata.clone(),
            Span::current(),
        ))
    }

    /// Returns a builder for a transaction with custom settings.
    ///
    /// Unlike the `transaction` method, the builder can be used to control the transaction's
    /// isolation level and other
    /// attributes.
    #[instrument(
        name = "instrumentedclient.build_transaction",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
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
        name = "instrumentedclient.prepare",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare(&self, query: &str) -> Result<Statement, Error> {
        self.inner.prepare(query).await
    }

    /// Like `prepare`, but allows the types of query parameters to be explicitly specified.
    ///
    /// The list of types may be smaller than the number of parameters - the types of the remaining
    /// parameters will be inferred. For example, `client.prepare_typed(query, &[])` is equivalent
    /// to `client.prepare(query)`.
    #[instrument(
        name = "instrumentedclient.prepare_typed",
        skip(self, query, parameter_types),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Statement, Error> {
        self.inner.prepare_typed(query, parameter_types).await
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
        name = "instrumentedclient.query",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Vec<Row>, Error> {
        let r = self.inner.query(statement, params).await;
        if let Ok(ref rows) = r {
            Span::current().record("db.rows", &display(rows.len()));
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
        name = "instrumentedclient.query_one",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Row, Error> {
        let r = self.inner.query_one(statement, params).await;
        if let Ok(_) = r {
            Span::current().record("db.rows", &display(1));
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
        name = "instrumentedclient.query_opt",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Option<Row>, Error> {
        let r = self.inner.query_opt(statement, params).await;
        if let Ok(ref maybe) = r {
            Span::current().record(
                "db.rows",
                &display(match maybe {
                    Some(_) => 1,
                    None => 0,
                }),
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
        name = "instrumentedclient.query_raw",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_raw<P, I>(&self, statement: &str, params: I) -> Result<RowStream, Error>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner.query_raw(statement, params).await
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
        name = "instrumentedclient.execute",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<u64, Error> {
        self.inner.execute(statement, params).await
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
        name = "instrumentedclient.execute_raw",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn execute_raw<P, I>(&self, statement: &str, params: I) -> Result<u64, Error>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        self.inner.execute_raw(statement, params).await
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
        name = "instrumentedclient.copy_in",
        skip(self, statement),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_in<T, U>(&self, statement: &T) -> Result<CopyInSink<U>, Error>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        self.inner.copy_in(statement).await
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
        name = "instrumentedclient.copy_out",
        skip(self, statement),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_out<T>(&self, statement: &T) -> Result<CopyOutStream, Error>
    where
        T: ?Sized + ToStatement,
    {
        self.inner.copy_out(statement).await
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
        name = "instrumentedclient.simple_query",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, Error> {
        self.inner.simple_query(query).await
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
        name = "instrumentedclient.batch_execute",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn batch_execute(&self, query: &str) -> Result<(), Error> {
        self.inner.batch_execute(query).await
    }

    /// Constructs a cancellation token that can later be used to request cancellation of a query
    /// running on the connection associated with this client.
    #[instrument(
        name = "instrumentedclient.cancel_token",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
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
        name = "instrumentedclient.clear_type_cache",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn clear_type_cache(&self) {
        self.inner.clear_type_cache()
    }

    /// Determines if the connection to the server has already closed.
    ///
    /// In that case, all future queries will fail.
    #[instrument(
        name = "instrumentedclient.is_closed",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
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
        name = "instrumentedtransaction.prepare_cached",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare_cached(&self, query: &str) -> Result<Statement, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .prepare_cached(query)
            .instrument(self.tx_span.clone())
            .await
    }

    /// Like [`tokio_postgres::Transaction::prepare_typed`](#method.prepare_typed-1)
    /// but uses an existing statement from the cache if possible.
    #[instrument(
        name = "instrumentedtransaction.prepare_typed_cached",
        skip(self, query, types),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Statement, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .prepare_typed_cached(query, types)
            .instrument(self.tx_span.clone())
            .await
    }

    /// Consumes the transaction, committing all changes made within it.
    #[instrument(
        name = "instrumentedtransaction.commit",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn commit(self) -> Result<(), Error> {
        Span::current().follows_from(&self.tx_span);
        let r = self.inner.commit().instrument(self.tx_span.clone()).await;
        self.tx_span.record("db.transaction", &display("commit"));
        r
    }

    /// Rolls the transaction back, discarding all changes made within it.
    ///
    /// This is equivalent to `Transaction`'s `Drop` implementation, but provides any error
    /// encountered to the caller.
    #[instrument(
        name = "instrumentedtransaction.rollback",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn rollback(self) -> Result<(), Error> {
        Span::current().follows_from(&self.tx_span);
        let r = self.inner.rollback().instrument(self.tx_span.clone()).await;
        self.tx_span.record("db.transaction", &display("rollback"));
        r
    }

    /// Creates a new prepared statement.
    ///
    /// Prepared statements can be executed repeatedly, and may contain query parameters (indicated
    /// by `$1`, `$2`, etc), which are set when executed. Prepared statements can only be used with
    /// the connection that created them.
    #[instrument(
        name = "instrumentedtransaction.prepare",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn prepare(&self, query: &str) -> Result<Statement, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .prepare(query)
            .instrument(self.tx_span.clone())
            .await
    }

    /// Like `prepare`, but allows the types of query parameters to be explicitly specified.
    ///
    /// The list of types may be smaller than the number of parameters - the types of the remaining
    /// parameters will be inferred. For example, `client.prepare_typed(query, &[])` is equivalent
    /// to `client.prepare(query)`.
    #[instrument(
        name = "instrumentedtransaction.prepare_typed",
        skip(self, query, parameter_types),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Statement, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .prepare_typed(query, parameter_types)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.query",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Vec<Row>, Error> {
        Span::current().follows_from(&self.tx_span);
        let r = self
            .inner
            .query(statement, params)
            .instrument(self.tx_span.clone())
            .await;
        if let Ok(ref rows) = r {
            Span::current().record("db.rows", &display(rows.len()));
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
        name = "instrumentedtransaction.query_one",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Row, Error> {
        Span::current().follows_from(&self.tx_span);
        let r = self
            .inner
            .query_one(statement, params)
            .instrument(self.tx_span.clone())
            .await;
        if let Ok(_) = r {
            Span::current().record("db.rows", &display(1));
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
        name = "instrumentedtransaction.query_opt",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<Option<Row>, Error> {
        Span::current().follows_from(&self.tx_span);
        let r = self
            .inner
            .query_opt(statement, params)
            .instrument(self.tx_span.clone())
            .await;
        if let Ok(ref maybe) = r {
            Span::current().record(
                "db.rows",
                &display(match maybe {
                    Some(_) => 1,
                    None => 0,
                }),
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
        name = "instrumentedtransaction.query_raw",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_raw<P, I>(&self, statement: &str, params: I) -> Result<RowStream, Error>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .query_raw(statement, params)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.execute",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
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
    ) -> Result<u64, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .execute(statement, params)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.execute_raw",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = statement,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn execute_raw<P, I>(&self, statement: &str, params: I) -> Result<u64, Error>
    where
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .execute_raw(statement, params)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.bind",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn bind<T>(
        &self,
        statement: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Portal, Error>
    where
        T: ?Sized + ToStatement,
    {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .bind(statement, params)
            .instrument(self.tx_span.clone())
            .await
    }

    /// A maximally flexible version of [`bind`].
    ///
    /// [`bind`]: #method.bind
    #[instrument(
        name = "instrumentedtransaction.bind_raw",
        skip(self, statement, params),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn bind_raw<P, T, I>(&self, statement: &T, params: I) -> Result<Portal, Error>
    where
        T: ?Sized + ToStatement,
        P: BorrowToSql,
        I: IntoIterator<Item = P>,
        I::IntoIter: ExactSizeIterator,
    {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .bind_raw(statement, params)
            .instrument(self.tx_span.clone())
            .await
    }

    /// Continues execution of a portal, returning a stream of the resulting rows.
    ///
    /// Unlike `query`, portals can be incrementally evaluated by limiting the number of rows
    /// returned in each call to `query_portal`. If the requested number is negative or 0, all rows
    /// will be returned.
    #[instrument(
        name = "instrumentedtransaction.query_portal",
        skip(self, portal, max_rows),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_portal(&self, portal: &Portal, max_rows: i32) -> Result<Vec<Row>, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .query_portal(portal, max_rows)
            .instrument(self.tx_span.clone())
            .await
    }

    /// The maximally flexible version of [`query_portal`].
    ///
    /// [`query_portal`]: #method.query_portal
    #[instrument(
        name = "instrumentedtransaction.query_portal_raw",
        skip(self, portal, max_rows),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn query_portal_raw(
        &self,
        portal: &Portal,
        max_rows: i32,
    ) -> Result<RowStream, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .query_portal_raw(portal, max_rows)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.copy_in",
        skip(self, statement),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_in<T, U>(&self, statement: &T) -> Result<CopyInSink<U>, Error>
    where
        T: ?Sized + ToStatement,
        U: Buf + 'static + Send,
    {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .copy_in(statement)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.copy_out",
        skip(self, statement),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn copy_out<T>(&self, statement: &T) -> Result<CopyOutStream, Error>
    where
        T: ?Sized + ToStatement,
    {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .copy_out(statement)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.simple_query",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn simple_query(&self, query: &str) -> Result<Vec<SimpleQueryMessage>, Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .simple_query(query)
            .instrument(self.tx_span.clone())
            .await
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
        name = "instrumentedtransaction.batch_execute",
        skip(self, query),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.statement = query,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn batch_execute(&self, query: &str) -> Result<(), Error> {
        Span::current().follows_from(&self.tx_span);
        self.inner
            .batch_execute(query)
            .instrument(self.tx_span.clone())
            .await
    }

    /// Constructs a cancellation token that can later be used to request cancellation of a query
    /// running on the connection associated with this client.
    #[instrument(
        name = "instrumentedtransaction.cancel_token",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn cancel_token(&self) -> CancelToken {
        Span::current().follows_from(&self.tx_span);
        self.tx_span.in_scope(|| self.inner.cancel_token())
    }

    /// Like `Client::transaction`, but creates a nested transaction via a savepoint.
    #[instrument(
        name = "instrumentedtransaction.transaction",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            db.transaction = Empty,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn transaction(&mut self) -> Result<InstrumentedTransaction<'_>, Error> {
        Ok(InstrumentedTransaction::new(
            self.inner
                .transaction()
                .instrument(self.tx_span.clone())
                .await?,
            self.metadata.clone(),
            Span::current(),
        ))
    }

    /// Like `Client::transaction`, but creates a nested transaction via a savepoint with the specified name.
    #[instrument(
        name = "instrumentedtransaction.savepoint",
        skip(self, name),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn savepoint<I>(&mut self, name: I) -> Result<InstrumentedTransaction<'_>, Error>
    where
        I: Into<String>,
    {
        Ok(InstrumentedTransaction::new(
            self.inner
                .savepoint(name)
                .instrument(self.tx_span.clone())
                .await?,
            self.metadata.clone(),
            Span::current(),
        ))
    }

    /// Returns a reference to the underlying `Client`.
    #[instrument(
        name = "instrumentedtransaction.client",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub fn client(&self) -> &Client {
        Span::current().follows_from(&self.tx_span);
        self.tx_span.in_scope(|| self.inner.client())
    }
}

impl<'a> fmt::Debug for InstrumentedTransaction<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedTransaction")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
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
    #[instrument(skip(self, isolation_level))]
    pub fn isolation_level(self, isolation_level: IsolationLevel) -> Self {
        Self {
            inner: self.inner.isolation_level(isolation_level),
            metadata: self.metadata,
        }
    }

    /// Sets the access mode of the transaction.
    ///
    /// Like `tokio_postgres::TransactionBuilder::read_only`
    #[instrument(skip(self, read_only))]
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
    #[instrument(skip(self, deferrable))]
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
        name = "instrumentedtransactionbuilder.start",
        skip(self),
        fields(
            db.system = %self.metadata.db_system,
            db.connection_string = %self.metadata.db_connection_string,
            db.name = %self.metadata.db_name,
            db.user = %self.metadata.db_user,
            net.peer.ip = %self.metadata.net_peer_ip,
            net.peer.port = %self.metadata.net_peer_port,
            net.transport = %self.metadata.net_transport,
        )
    )]
    pub async fn start(self) -> Result<InstrumentedTransaction<'a>, Error> {
        Ok(InstrumentedTransaction::new(
            self.inner.start().await?,
            self.metadata,
            Span::current(),
        ))
    }
}

impl<'a> fmt::Debug for InstrumentedTransactionBuilder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InstrumentedTransactionBuilder")
            .field("metadata", &self.metadata)
            .finish_non_exhaustive()
    }
}
