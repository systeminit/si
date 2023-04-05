use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use si_data_nats::{NatsClient, NatsError, NatsTxn};
use si_data_pg::{InstrumentedClient, PgError, PgPool, PgPoolError, PgPoolResult, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::{Client as VeritechClient, EncryptionKey};

use crate::{
    job::{
        processor::{JobQueueProcessor, JobQueueProcessorError},
        producer::JobProducer,
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
    /// The prefix for channels names when communicating with Council
    council_subject_prefix: String,
    /// The path where available packages can be found
    pkgs_path: Option<PathBuf>,
}

impl ServicesContext {
    /// Constructs a new instance of a `ServicesContext`.
    pub fn new(
        pg_pool: PgPool,
        nats_conn: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: Arc<EncryptionKey>,
        council_subject_prefix: String,
        pkgs_path: Option<PathBuf>,
    ) -> Self {
        Self {
            pg_pool,
            nats_conn,
            job_processor,
            veritech,
            encryption_key,
            council_subject_prefix,
            pkgs_path,
        }
    }

    /// Consumes and returns [`DalContextBuilder`].
    pub fn into_builder(self) -> DalContextBuilder {
        DalContextBuilder {
            services_context: self,
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
    pub fn encryption_key(&self) -> &EncryptionKey {
        &self.encryption_key
    }

    /// Builds and returns a new [`Connections`].
    pub async fn connections(&self) -> PgPoolResult<Connections> {
        let pg_conn = self.pg_pool.get().await?;
        let nats_conn = self.nats_conn.clone();
        let job_processor = self.job_processor.clone();
        Ok(Connections::new(pg_conn, nats_conn, job_processor))
    }
}

/// A context type which holds references to underlying services, transactions, and context for DAL objects.
#[derive(Clone, Debug)]
pub struct DalContext {
    /// A reference to a [`ServicesContext`] which has handles to common core services.
    services_context: ServicesContext,
    /// A reference to a set of atomically related transactions.
    txns: Transactions,
    /// A suitable tenancy for the consuming DAL objects.
    tenancy: Tenancy,
    /// A suitable [`Visibility`] scope for the consuming DAL objects.
    visibility: Visibility,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
}

impl DalContext {
    /// Takes a reference to a [`ServicesContext`] and returns a builder to construct a
    /// `DalContext`.
    pub fn builder(services_context: ServicesContext) -> DalContextBuilder {
        DalContextBuilder { services_context }
    }

    /// Consumes context, commiting transactions, returning new context with new transaction
    pub async fn commit_and_continue(self) -> Result<Self, TransactionsError> {
        let (builder, conns, request_ctx) = self.commit_into_parts().await?;
        builder.build_with_conns(request_ctx, conns).await
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// underlying connections.
    pub async fn commit_into_conns(self) -> Result<Connections, TransactionsError> {
        self.txns.commit_into_conns().await
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// the context parts which can be used to build a new context.
    pub async fn commit_into_parts(
        self,
    ) -> Result<(DalContextBuilder, Connections, RequestContext), TransactionsError> {
        let conns = self.txns.commit_into_conns().await?;
        let builder = self.services_context.into_builder();
        let request_ctx = RequestContext {
            tenancy: self.tenancy,
            visibility: self.visibility,
            history_actor: self.history_actor,
        };

        Ok((builder, conns, request_ctx))
    }

    /// Consumes all inner transactions and committing all changes made within them.
    pub async fn commit(self) -> Result<(), TransactionsError> {
        let _ = self.commit_into_conns().await?;
        Ok(())
    }

    pub fn services_context(&self) -> ServicesContext {
        self.services_context.clone()
    }

    /// Rolls all inner transactions back, discarding all changes made within them, and returns
    /// underlying connections.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback_into_conns(self) -> Result<Connections, TransactionsError> {
        self.txns.rollback_into_conns().await
    }

    /// Rolls all inner transactions back, discarding all changes made within them, and returns the
    /// context parts which can be used to build a new context.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback_into_parts(
        self,
    ) -> Result<(DalContextBuilder, Connections, RequestContext), TransactionsError> {
        let conns = self.txns.rollback_into_conns().await?;
        let builder = self.services_context.into_builder();
        let request_ctx = RequestContext {
            tenancy: self.tenancy,
            visibility: self.visibility,
            history_actor: self.history_actor,
        };

        Ok((builder, conns, request_ctx))
    }

    /// Rolls all inner transactions back, discarding all changes made within them.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback(self) -> Result<(), TransactionsError> {
        let _ = self.rollback_into_conns().await?;
        Ok(())
    }

    /// Updates this context with all new values from a [`RequestContext`].
    pub fn update_from_request_context(&mut self, request_ctx: RequestContext) {
        self.tenancy = request_ctx.tenancy;
        self.visibility = request_ctx.visibility;
        self.history_actor = request_ctx.history_actor;
    }

    /// Clones a new context from this one with the same metadata, but in different transactions
    pub async fn clone_with_new_transactions(&self) -> Result<Self, TransactionsError> {
        let mut other = self.clone();
        other.txns = self
            .services_context
            .connections()
            .await?
            .start_txns()
            .await?;
        Ok(other)
    }

    /// Clones a new context from this one with all new values from a  [`RequestContext`].
    pub fn clone_from_request_context(&self, request_ctx: RequestContext) -> Self {
        let mut new = self.clone();
        new.update_from_request_context(request_ctx);
        new
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

    /// Updates this context with a new [`Visibility`].
    pub fn update_visibility(&mut self, visibility: Visibility) {
        self.visibility = visibility;
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

    pub async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        self.txns().job_processor.enqueue_job(job, self).await
    }

    pub async fn enqueue_blocking_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        self.txns()
            .job_processor
            .enqueue_blocking_job(job, self)
            .await
    }

    /// Gets the dal context's txns.
    pub fn txns(&self) -> &Transactions {
        &self.txns
    }

    pub fn job_processor(&self) -> Box<dyn JobQueueProcessor + Send + Sync> {
        self.services_context.job_processor.clone()
    }

    /// Gets a reference to the DAL context's Postgres pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.services_context.pg_pool
    }

    /// Gets the dal context's pg txn.
    pub fn pg_txn(&self) -> &PgTxn {
        &self.txns.pg_txn
    }

    /// Gets a reference to the DAL context's NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }

    /// Gets the dal context's nats txn.
    pub fn nats_txn(&self) -> &NatsTxn {
        &self.txns.nats_txn
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

    /// Gets a reference to the council's communication channel subject prefix
    pub fn council_subject_prefix(&self) -> &str {
        &self.services_context.council_subject_prefix
    }

    /// Determines if a standard model object matches the tenancy of the current context and
    /// is in the same visibility.
    pub async fn check_tenancy<T: StandardModel>(
        &self,
        object: &T,
    ) -> Result<bool, TransactionsError> {
        let is_in_our_tenancy = self
            .tenancy()
            .check(self.pg_txn(), object.tenancy())
            .await?;

        Ok(is_in_our_tenancy)
    }

    /// Copies every single row from `Self::builtin()` to our tenancy on head change-set
    /// Needed to remove universal tenancy while packages aren't a thing
    #[instrument(skip_all)]
    pub async fn import_builtins(&self) -> Result<(), TransactionsError> {
        self.txns()
            .pg()
            .execute("SELECT import_builtins_v1($1)", &[self.tenancy()])
            .await?;
        Ok(())
    }
}

impl From<DalContext> for Transactions {
    fn from(value: DalContext) -> Self {
        value.txns
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

impl RequestContext {
    /// Get a reference to the request context's tenancy.
    #[must_use]
    pub fn tenancy(&self) -> &Tenancy {
        &self.tenancy
    }

    /// Get the request context's visibility.
    #[must_use]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    /// Get a reference to the request context's history actor.
    #[must_use]
    pub fn history_actor(&self) -> &HistoryActor {
        &self.history_actor
    }
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

    /// Builds and returns a new [`RequestContext`] using the default [`Visibility`].
    pub fn build_head(self) -> RequestContext {
        RequestContext {
            tenancy: self.tenancy,
            visibility: Visibility::new_head(false),
            history_actor: self.history_actor,
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
}

impl DalContextBuilder {
    /// Contructs and returns a new [`DalContext`] using the given [`RequestContext`] and
    /// an existing [`Connections`].
    pub async fn build_with_conns(
        &self,
        request_context: RequestContext,
        conns: Connections,
    ) -> Result<DalContext, TransactionsError> {
        let txns = conns.start_txns().await?;

        Ok(DalContext {
            services_context: self.services_context.clone(),
            txns,
            tenancy: request_context.tenancy,
            visibility: request_context.visibility,
            history_actor: request_context.history_actor,
        })
    }

    /// Contructs and returns a new [`DalContext`] using a default [`RequestContext`] and
    /// the given [`Transactions`].
    pub fn build_default_with_txns(&self, txns: Transactions) -> DalContext {
        self.build_with_txns(RequestContext::default(), txns)
    }

    /// Contructs and returns a new [`DalContext`] using the given [`RequestContext`] and
    /// an existing [`Transactions`].
    pub fn build_with_txns(
        &self,
        request_context: RequestContext,
        txns: Transactions,
    ) -> DalContext {
        DalContext {
            services_context: self.services_context.clone(),
            txns,
            tenancy: request_context.tenancy,
            visibility: request_context.visibility,
            history_actor: request_context.history_actor,
        }
    }

    /// Contructs and returns a new [`DalContext`] using a default [`RequestContext`].
    pub async fn build_default(&self) -> Result<DalContext, TransactionsError> {
        self.build(RequestContext::default()).await
    }

    /// Contructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build(
        &self,
        request_context: RequestContext,
    ) -> Result<DalContext, TransactionsError> {
        let conns = self.connections().await?;
        self.build_with_conns(request_context, conns).await
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
}

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

    /// Consumes all inner transactions and committing all changes made within them.
    pub async fn commit(self) -> Result<(), TransactionsError> {
        let _ = self.commit_into_conns().await?;
        Ok(())
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
