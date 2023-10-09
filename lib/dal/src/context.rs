use std::{mem, path::PathBuf, sync::Arc};

use futures::Future;
use serde::{Deserialize, Serialize};
use si_data_nats::{NatsClient, NatsError, NatsTxn};
use si_data_pg::{InstrumentedClient, PgError, PgPool, PgPoolError, PgPoolResult, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use veritech_client::{Client as VeritechClient, EncryptionKey};

use crate::{
    job::{
        processor::{JobQueueProcessor, JobQueueProcessorError},
        producer::{BlockingJobError, BlockingJobResult, JobProducer},
    },
    HistoryActor, StandardModel, Tenancy, TenancyError, Visibility,
};

/// A context type which contains handles to common core service dependencies.
///
/// These services are typically used by most DAL objects, such as a database connection pool, a
/// function execution client, etc.
#[derive(Clone, Debug)]
pub struct ServicesContext {
    /// A PostgreSQL connection pool.
    pg_pool: PgPool,
    /// A connected NATS client
    nats_conn: NatsClient,
    /// A connected job processor client
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    /// A Veritech client, connected via a NATS connection.
    veritech: VeritechClient,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<EncryptionKey>,
    /// The path where available packages can be found
    pkgs_path: Option<PathBuf>,
    /// The URL of the module index
    module_index_url: Option<String>,
}

impl ServicesContext {
    /// Constructs a new instance of a `ServicesContext`.
    pub fn new(
        pg_pool: PgPool,
        nats_conn: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: Arc<EncryptionKey>,
        pkgs_path: Option<PathBuf>,
        module_index_url: Option<String>,
    ) -> Self {
        Self {
            pg_pool,
            nats_conn,
            job_processor,
            veritech,
            encryption_key,
            pkgs_path,
            module_index_url,
        }
    }

    /// Consumes and returns [`DalContextBuilder`].
    pub fn into_builder(self, blocking: bool) -> DalContextBuilder {
        DalContextBuilder {
            services_context: self,
            blocking,
            no_dependent_values: false,
        }
    }

    /// Gets a reference to the Postgres pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.pg_pool
    }

    /// Gets a reference to the NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.nats_conn
    }

    /// Gets a reference to the Veritech client.
    pub fn veritech(&self) -> &VeritechClient {
        &self.veritech
    }

    pub fn job_processor(&self) -> Box<dyn JobQueueProcessor + Send + Sync> {
        self.job_processor.clone()
    }

    /// Gets a reference to the encryption key.
    pub fn encryption_key(&self) -> Arc<EncryptionKey> {
        self.encryption_key.clone()
    }

    /// Builds and returns a new [`Connections`].
    pub async fn connections(&self) -> PgPoolResult<Connections> {
        let pg_conn = self.pg_pool.get().await?;
        let nats_conn = self.nats_conn.clone();
        let job_processor = self.job_processor.clone();
        Ok(Connections::new(pg_conn, nats_conn, job_processor))
    }
}

#[remain::sorted]
#[derive(Debug)]
enum ConnectionState {
    Connections(Connections),
    Invalid,
    Transactions(Transactions),
}

impl ConnectionState {
    fn new_from_conns(conns: Connections) -> Self {
        Self::Connections(conns)
    }

    fn take(&mut self) -> Self {
        mem::replace(self, Self::Invalid)
    }

    fn is_conns(&self) -> bool {
        matches!(self, Self::Connections(_))
    }

    fn txns(&mut self) -> &mut Transactions {
        match self {
            Self::Transactions(txns) => txns,
            _ => {
                // The caller of this method has already ensured that we can only be in the
                // Transactions state (remember, this type is internal to DalContext)
                panic!("caller must ensure state is txns--this is an internal bug");
            }
        }
    }

    async fn start_txns(self) -> Result<Self, TransactionsError> {
        match self {
            Self::Invalid => Err(TransactionsError::TxnStart("invalid")),
            Self::Connections(conns) => Ok(Self::Transactions(conns.start_txns().await?)),
            Self::Transactions(_) => Err(TransactionsError::TxnStart("transactions")),
        }
    }

    async fn commit(self) -> Result<Self, TransactionsError> {
        match self {
            Self::Connections(_) => {
                trace!("no active transactions present when commit was called, taking no action");
                Ok(self)
            }
            Self::Transactions(txns) => {
                let conns = txns.commit_into_conns().await?;
                Ok(Self::Connections(conns))
            }
            Self::Invalid => Err(TransactionsError::TxnCommit),
        }
    }

    async fn blocking_commit(self) -> Result<Self, TransactionsError> {
        match self {
            Self::Connections(_) => {
                trace!("no active transactions present when commit was called, taking no action");
                Ok(self)
            }
            Self::Transactions(txns) => {
                let conns = txns.blocking_commit_into_conns().await?;
                Ok(Self::Connections(conns))
            }
            Self::Invalid => Err(TransactionsError::TxnCommit),
        }
    }

    async fn rollback(self) -> Result<Self, TransactionsError> {
        match self {
            Self::Connections(_) => {
                trace!("no active transactions present when rollback was called, taking no action");
                Ok(self)
            }
            Self::Transactions(txns) => {
                let conns = txns.rollback_into_conns().await?;
                Ok(Self::Connections(conns))
            }
            Self::Invalid => Err(TransactionsError::TxnRollback),
        }
    }
}

/// A context type which holds references to underlying services, transactions, and context for DAL objects.
#[derive(Clone, Debug)]
pub struct DalContext {
    /// A reference to a [`ServicesContext`] which has handles to common core services.
    services_context: ServicesContext,
    /// A reference to a set of atomically related transactions.
    conns_state: Arc<Mutex<ConnectionState>>,
    /// A suitable tenancy for the consuming DAL objects.
    tenancy: Tenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
    /// Determines if regular commits block until the jobs get executed.
    /// This is useful to ensure child jobs of blocking jobs also block so there is no race-condition in the DAL.
    /// And also for SDF routes to block the HTTP request until the jobs get executed, so SDF tests don't race.
    blocking: bool,
    /// Determines if we should not enqueue dependent value update jobs for attribute updates in
    /// this context
    no_dependent_values: bool,
}

impl DalContext {
    /// Takes a reference to a [`ServicesContext`] and returns a builder to construct a
    /// `DalContext`.
    pub fn builder(services_context: ServicesContext, blocking: bool) -> DalContextBuilder {
        DalContextBuilder {
            services_context,
            blocking,
            no_dependent_values: false,
        }
    }

    /// Consumes all inner transactions and committing all changes made within them.
    pub async fn commit(&self) -> Result<(), TransactionsError> {
        if self.blocking {
            self.blocking_commit().await?;
        } else {
            let mut guard = self.conns_state.lock().await;
            *guard = guard.take().commit().await?;
        }

        Ok(())
    }

    pub fn blocking(&self) -> bool {
        self.blocking
    }

    pub fn no_dependent_values(&self) -> bool {
        self.no_dependent_values
    }

    pub fn services_context(&self) -> ServicesContext {
        self.services_context.clone()
    }

    /// Consumes all inner transactions, committing all changes made within them, and
    /// blocks until all queued jobs have reported as finishing.
    pub async fn blocking_commit(&self) -> Result<(), TransactionsError> {
        let mut guard = self.conns_state.lock().await;

        *guard = guard.take().blocking_commit().await?;

        Ok(())
    }

    /// Rolls all inner transactions back, discarding all changes made within them.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback(&self) -> Result<(), TransactionsError> {
        let mut guard = self.conns_state.lock().await;

        *guard = guard.take().rollback().await?;

        Ok(())
    }

    /// Updates this context with a new [`HistoryActor`].
    pub fn update_history_actor(&mut self, history_actor: HistoryActor) {
        self.history_actor = history_actor;
    }

    /// Clones a new context from this one with a new [`HistoryActor`].
    pub fn clone_with_new_history_actor(&self, history_actor: HistoryActor) -> Self {
        let mut new = self.clone();
        new.update_history_actor(history_actor);
        new
    }

    /// Updates this context with a new [`Visibility`].
    pub fn update_access_builder(&mut self, access_builder: AccessBuilder) {
        self.tenancy = access_builder.tenancy;
        self.history_actor = access_builder.history_actor;
    }

    /// Runs a block of code with a custom [`Visibility`] DalContext using the same transactions
    pub async fn run_with_visibility<F, Fut, R>(&self, visibility: Visibility, fun: F) -> R
    where
        F: FnOnce(DalContext) -> Fut,
        Fut: Future<Output = R>,
    {
        let mut ctx = self.clone();
        ctx.update_visibility(visibility);

        fun(ctx).await
    }

    /// Updates this context with a new [`Visibility`].
    pub fn update_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    /// Runs a block of code with "deleted" [`Visibility`] DalContext using the same transactions
    pub async fn run_with_deleted_visibility<F, Fut, R>(&self, fun: F) -> R
    where
        F: FnOnce(DalContext) -> Fut,
        Fut: Future<Output = R>,
    {
        self.run_with_visibility(
            Visibility::new_change_set(self.visibility().change_set_pk, true),
            fun,
        )
        .await
    }

    /// Mutates [`self`](DalContext) with a "deleted" [`Visibility`].
    pub fn update_with_deleted_visibility(&mut self) {
        self.update_visibility(Visibility::new_change_set(
            self.visibility().change_set_pk,
            true,
        ));
    }

    /// Mutates [`self`](DalContext) with a "non-deleted" [`Visibility`].
    pub fn update_without_deleted_visibility(&mut self) {
        self.update_visibility(Visibility::new_change_set(
            self.visibility().change_set_pk,
            false,
        ));
    }

    /// Clones a new context from this one without deleted visibility.
    pub fn clone_without_deleted_visibility(&self) -> Self {
        let mut ctx = self.clone();
        ctx.update_without_deleted_visibility();
        ctx
    }

    /// Clones a new context from this one with a new [`Visibility`].
    pub fn clone_with_new_visibility(&self, visibility: Visibility) -> Self {
        let mut new = self.clone();
        new.update_visibility(visibility);
        new
    }

    /// Clones a new context from this one [`Visibility`] that allows querying deleted values.
    pub fn clone_with_delete_visibility(&self) -> Self {
        self.clone_with_new_visibility(Visibility::new_change_set(
            self.visibility().change_set_pk,
            true,
        ))
    }

    /// Updates this context with a new [`Tenancy`]
    pub fn update_tenancy(&mut self, tenancy: Tenancy) {
        self.tenancy = tenancy;
    }

    /// Clones a new context from this one with a new [`Tenancy`] and [`Tenancy`].
    pub fn clone_with_new_tenancy(&self, tenancy: Tenancy) -> Self {
        let mut new = self.clone();
        new.update_tenancy(tenancy);
        new
    }

    /// Updates this context with a head [`Visibility`].
    pub fn update_to_head(&mut self) {
        self.visibility = Visibility::new_head(false);
    }

    /// Clones a new context from this one with a head [`Visibility`].
    pub fn clone_with_head(&self) -> Self {
        let mut new = self.clone();
        new.update_to_head();
        new
    }

    pub async fn enqueue_job(
        &self,
        job: Box<dyn JobProducer + Send + Sync>,
    ) -> Result<(), TransactionsError> {
        self.txns()
            .await?
            .job_processor
            .enqueue_job(job, self)
            .await;
        Ok(())
    }

    /// Similar to `enqueue_job`, except that instead of waiting to flush the job to
    /// the processing system on `commit`, the job is immediately flushed, and the
    /// processor is expected to not return until the job has finished. Returns the
    /// result of executing the job.
    pub async fn block_on_job(&self, job: Box<dyn JobProducer + Send + Sync>) -> BlockingJobResult {
        self.txns()
            .await
            .map_err(|err| BlockingJobError::Transactions(err.to_string()))?
            .job_processor
            .block_on_job(job)
            .await
    }

    /// Gets the dal context's txns.
    pub async fn txns(&self) -> Result<MappedMutexGuard<'_, Transactions>, TransactionsError> {
        let mut guard = self.conns_state.lock().await;

        let conns_state = guard.take();

        if conns_state.is_conns() {
            // If we are Connections, then we need to start Transactions
            *guard = conns_state.start_txns().await?;
        } else {
            // Otherwise, we return the state back to the guard--it's Transactions under normal
            // circumstances, and Invalid if something went wrong with a previous Transactions
            *guard = conns_state;
        }

        Ok(MutexGuard::map(guard, |cs| cs.txns()))
    }

    pub fn job_processor(&self) -> Box<dyn JobQueueProcessor + Send + Sync> {
        self.services_context.job_processor.clone()
    }

    /// Gets a reference to the DAL context's Postgres pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.services_context.pg_pool
    }

    /// Gets a reference to the DAL context's NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }

    /// Gets a reference to the DAL context's Veritech client.
    pub fn veritech(&self) -> &VeritechClient {
        &self.services_context.veritech
    }

    /// Gets a reference to the DAL context's encryption key.
    pub fn encryption_key(&self) -> &EncryptionKey {
        &self.services_context.encryption_key
    }

    /// Gets a reference to the dal context's tenancy.
    pub fn tenancy(&self) -> &Tenancy {
        &self.tenancy
    }

    /// Gets the dal context's visibility.
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Gets a reference to the dal context's history actor.
    pub fn history_actor(&self) -> &HistoryActor {
        &self.history_actor
    }

    /// Gets an optional reference to the dal context's pkgs path
    pub fn pkgs_path(&self) -> Option<&PathBuf> {
        self.services_context.pkgs_path.as_ref()
    }

    /// Gets an optional reference to the module index service's url
    pub fn module_index_url(&self) -> Option<&str> {
        self.services_context.module_index_url.as_deref()
    }

    /// Determines if a standard model object matches the tenancy of the current context and
    /// is in the same visibility.
    pub async fn check_tenancy<T: StandardModel>(
        &self,
        object: &T,
    ) -> Result<bool, TransactionsError> {
        let is_in_our_tenancy = self
            .tenancy()
            .check(self.txns().await?.pg(), object.tenancy())
            .await?;

        Ok(is_in_our_tenancy)
    }

    /// Copies every single row from `Self::builtin()` to our tenancy on head change-set
    /// Needed to remove universal tenancy while packages aren't a thing
    #[instrument(skip_all)]
    pub async fn import_builtins(&self) -> Result<(), TransactionsError> {
        self.txns()
            .await?
            .pg()
            .execute("SELECT import_builtins_v1($1)", &[self.tenancy()])
            .await?;
        Ok(())
    }

    pub fn access_builder(&self) -> AccessBuilder {
        AccessBuilder::new(self.tenancy, self.history_actor)
    }
}

/// A context which represents a suitable tenancies, visibilities, etc. for consumption by a set
/// of DAL objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestContext {
    /// A suitable tenancy for the consuming DAL objects.
    pub tenancy: Tenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    pub visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    pub history_actor: HistoryActor,
}

impl Default for RequestContext {
    /// Builds a new [`RequestContext`] with no tenancy (only usable for managing objects that live outside of the standard model)
    /// and a head [`Visibility`] and the given [`HistoryActor`].
    fn default() -> Self {
        Self {
            tenancy: Tenancy::new_empty(),
            visibility: Visibility::new_head(false),
            history_actor: HistoryActor::SystemInit,
        }
    }
}

/// A request context builder which requires a [`Visibility`] to be completed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessBuilder {
    /// A suitable tenancy for the consuming DAL objects.
    tenancy: Tenancy,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
}

impl AccessBuilder {
    /// Constructs a new instance given a tenancy and a [`HistoryActor`].
    pub fn new(tenancy: Tenancy, history_actor: HistoryActor) -> Self {
        Self {
            tenancy,
            history_actor,
        }
    }

    /// Builds and returns a new [`RequestContext`] using the given [`Visibility`].
    pub fn build(self, visibility: Visibility) -> RequestContext {
        RequestContext {
            tenancy: self.tenancy,
            visibility,
            history_actor: self.history_actor,
        }
    }

    /// Gets a reference to the dal context's tenancy.
    pub fn tenancy(&self) -> &Tenancy {
        &self.tenancy
    }

    /// Gets a reference to the dal context's history actor.
    pub fn history_actor(&self) -> &HistoryActor {
        &self.history_actor
    }
}

impl From<DalContext> for AccessBuilder {
    fn from(ctx: DalContext) -> Self {
        Self::new(ctx.tenancy, ctx.history_actor)
    }
}

/// A builder for a [`DalContext`].
#[derive(Clone, Debug)]
pub struct DalContextBuilder {
    /// A [`ServicesContext`] which has handles to common core services.
    services_context: ServicesContext,
    /// Determines if regular commits block until the jobs get executed.
    /// This is useful to ensure child jobs of blocking jobs also block so there is no race-condition in the DAL.
    /// And also for SDF routes to block the HTTP request until the jobs get executed, so SDF tests don't race.
    blocking: bool,
    /// Determines if we should not enqueue dependent value update jobs for attribute value
    /// changes.
    no_dependent_values: bool,
}

impl DalContextBuilder {
    /// Contructs and returns a new [`DalContext`] using a default [`RequestContext`].
    pub async fn build_default(&self) -> Result<DalContext, TransactionsError> {
        let conns = self.connections().await?;
        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new_empty(),
            visibility: Visibility::new_head(false),
            history_actor: HistoryActor::SystemInit,
            no_dependent_values: self.no_dependent_values,
        })
    }

    /// Contructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build_head(
        &self,
        access_builder: AccessBuilder,
    ) -> Result<DalContext, TransactionsError> {
        let conns = self.connections().await?;
        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: access_builder.tenancy,
            history_actor: access_builder.history_actor,
            visibility: Visibility::new_head(false),
            no_dependent_values: self.no_dependent_values,
        })
    }

    /// Contructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build(
        &self,
        request_context: RequestContext,
    ) -> Result<DalContext, TransactionsError> {
        let conns = self.connections().await?;
        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: request_context.tenancy,
            visibility: request_context.visibility,
            history_actor: request_context.history_actor,
            no_dependent_values: self.no_dependent_values,
        })
    }

    /// Gets a reference to the PostgreSQL connection pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.services_context.pg_pool
    }

    /// Gets a reference to the NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }

    pub fn job_processor(&self) -> Box<dyn JobQueueProcessor + Send + Sync> {
        self.services_context.job_processor.clone()
    }

    /// Builds and returns a new [`Connections`].
    pub async fn connections(&self) -> PgPoolResult<Connections> {
        self.services_context.connections().await
    }

    /// Returns the location on disk where packages are stored (if one was provided)
    pub async fn pkgs_path(&self) -> Option<&PathBuf> {
        self.services_context.pkgs_path.as_ref()
    }

    /// Gets a reference to the [`ServicesContext`].
    pub fn services_context(&self) -> &ServicesContext {
        &self.services_context
    }

    /// Set blocking flag
    pub fn set_blocking(&mut self) {
        self.blocking = true;
    }

    pub fn set_no_dependent_values(&mut self) {
        self.no_dependent_values = true;
    }
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum TransactionsError {
    #[error(transparent)]
    JobQueueProcessor(#[from] JobQueueProcessorError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Tenancy(#[from] TenancyError),
    #[error("cannot commit transactions on invalid connections state")]
    TxnCommit,
    #[error("cannot rollback transactions on invalid connections state")]
    TxnRollback,
    #[error("cannot start transactions without connections; state={0}")]
    TxnStart(&'static str),
}

/// A type which holds ownership over connections that can be used to start transactions.
#[derive(Debug)]
pub struct Connections {
    pg_conn: InstrumentedClient,
    nats_conn: NatsClient,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
}

impl Connections {
    /// Builds a new [`Connections`].
    #[must_use]
    pub fn new(
        pg_conn: InstrumentedClient,
        nats_conn: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    ) -> Self {
        Self {
            pg_conn,
            nats_conn,
            job_processor,
        }
    }

    /// Starts and returns a [`Transactions`].
    pub async fn start_txns(self) -> Result<Transactions, TransactionsError> {
        let pg_txn = PgTxn::create(self.pg_conn).await?;
        let nats_txn = self.nats_conn.transaction();
        let job_processor = self.job_processor;

        Ok(Transactions::new(pg_txn, nats_txn, job_processor))
    }

    /// Gets a reference to a PostgreSQL connection.
    pub fn pg_conn(&self) -> &InstrumentedClient {
        &self.pg_conn
    }

    /// Gets a reference to a NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.nats_conn
    }
}

// A set of atomically-related transactions.
//
// Ideally, all of these inner transactions would be committed or rolled back together, hence the
// API methods.
#[derive(Clone, Debug)]
pub struct Transactions {
    /// A PostgreSQL transaction.
    pg_txn: PgTxn,
    /// A NATS transaction.
    nats_txn: NatsTxn,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
}

impl Transactions {
    /// Creates and returns a new `Transactions` instance.
    fn new(
        pg_txn: PgTxn,
        nats_txn: NatsTxn,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    ) -> Self {
        Self {
            pg_txn,
            nats_txn,
            job_processor,
        }
    }

    /// Gets a reference to the PostgreSQL transaction.
    pub fn pg(&self) -> &PgTxn {
        &self.pg_txn
    }

    /// Gets a reference to the NATS transaction.
    pub fn nats(&self) -> &NatsTxn {
        &self.nats_txn
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// underlying connections.
    pub async fn commit_into_conns(self) -> Result<Connections, TransactionsError> {
        let pg_conn = self.pg_txn.commit_into_conn().await?;
        let nats_conn = self.nats_txn.commit_into_conn().await?;
        self.job_processor.process_queue().await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor);

        Ok(conns)
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// underlying connections. Blocking until all queued jobs have reported as finishing.
    pub async fn blocking_commit_into_conns(self) -> Result<Connections, TransactionsError> {
        let pg_conn = self.pg_txn.commit_into_conn().await?;
        let nats_conn = self.nats_txn.commit_into_conn().await?;
        self.job_processor.blocking_process_queue().await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor);

        Ok(conns)
    }

    /// Rolls all inner transactions back, discarding all changes made within them, and returns
    /// underlying connections.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback_into_conns(self) -> Result<Connections, TransactionsError> {
        let pg_conn = self.pg_txn.rollback_into_conn().await?;
        let nats_conn = self.nats_txn.rollback_into_conn().await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor);

        Ok(conns)
    }

    /// Rolls all inner transactions back, discarding all changes made within them.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback(self) -> Result<(), TransactionsError> {
        let _ = self.rollback_into_conns().await?;
        Ok(())
    }
}
