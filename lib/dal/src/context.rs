use std::collections::HashSet;
use std::{fmt, mem, path::PathBuf, sync::Arc};

use futures::Future;
use serde::{Deserialize, Serialize};
use si_crypto::SymmetricCryptoService;
use si_crypto::VeritechEncryptionKey;
use si_data_nats::{NatsClient, NatsError, NatsTxn};
use si_data_pg::{InstrumentedClient, PgError, PgPool, PgPoolError, PgPoolResult, PgTxn};
use si_events::rebase_batch_address::RebaseBatchAddress;
use si_events::WorkspaceSnapshotAddress;
use si_layer_cache::activities::rebase::RebaseStatus;
use si_layer_cache::activities::ActivityPayload;
use si_layer_cache::activities::ActivityPayloadDiscriminants;
use si_layer_cache::db::LayerDb;
use si_layer_cache::event::LayeredEventMetadata;
use si_layer_cache::LayerDbError;
use si_runtime::DedicatedExecutor;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use tokio::time::Instant;
use veritech_client::Client as VeritechClient;

use crate::feature_flags::FeatureFlagService;
use crate::jetstream_streams::JetstreamStreams;
use crate::job::definition::AttributeValueBasedJobIdentifier;
use crate::layer_db_types::ContentTypes;
use crate::slow_rt::SlowRuntimeError;
use crate::workspace_snapshot::graph::detect_updates::Update;
use crate::workspace_snapshot::graph::{RebaseBatch, WorkspaceSnapshotGraph};
use crate::workspace_snapshot::DependentValueRoot;
use crate::{
    change_set::{ChangeSet, ChangeSetId},
    job::{
        definition::ActionJob,
        processor::{JobQueueProcessor, JobQueueProcessorError},
        producer::{BlockingJobError, BlockingJobResult, JobProducer},
        queue::JobQueue,
    },
    workspace_snapshot::WorkspaceSnapshotError,
    AttributeValueId, HistoryActor, StandardModel, Tenancy, TenancyError, Visibility, WorkspacePk,
    WorkspaceSnapshot,
};
use crate::{slow_rt, EncryptedSecret, Workspace, WorkspaceError};

pub type DalLayerDb = LayerDb<ContentTypes, EncryptedSecret, WorkspaceSnapshotGraph, RebaseBatch>;

/// A context type which contains handles to common core service dependencies.
///
/// These services are typically used by most DAL objects, such as a database connection pool, a
/// function execution client, etc.
#[derive(Clone)]
pub struct ServicesContext {
    /// A PostgreSQL connection pool.
    pg_pool: PgPool,
    /// A connected NATS client
    nats_conn: NatsClient,
    /// NATS Jetstream streams that we need to publish to or consume from.
    jetstream_streams: JetstreamStreams,
    /// A connected job processor client
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    /// A Veritech client, connected via a NATS connection.
    veritech: VeritechClient,
    /// A key for re-recrypting messages to the function execution system.
    encryption_key: Arc<VeritechEncryptionKey>,
    /// The path where available packages can be found
    pkgs_path: Option<PathBuf>,
    /// The URL of the module index
    module_index_url: Option<String>,
    /// A service that can encrypt and decrypt values with a set of symmetric keys
    symmetric_crypto_service: SymmetricCryptoService,
    /// The layer db
    layer_db: DalLayerDb,
    /// The service that stores feature flags
    feature_flag_service: FeatureFlagService,
    /// Dedicated executor for running CPU-intensive tasks
    compute_executor: DedicatedExecutor,
}

impl ServicesContext {
    /// Constructs a new instance of a `ServicesContext`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pg_pool: PgPool,
        nats_conn: NatsClient,
        jetstream_streams: JetstreamStreams,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: Arc<VeritechEncryptionKey>,
        pkgs_path: Option<PathBuf>,
        module_index_url: Option<String>,
        symmetric_crypto_service: SymmetricCryptoService,
        layer_db: DalLayerDb,
        feature_flag_service: FeatureFlagService,
        compute_executor: DedicatedExecutor,
    ) -> Self {
        Self {
            pg_pool,
            nats_conn,
            jetstream_streams,
            job_processor,
            veritech,
            encryption_key,
            pkgs_path,
            module_index_url,
            symmetric_crypto_service,
            layer_db,
            feature_flag_service,
            compute_executor,
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

    /// Gets a reference to the NATS Jetstream streams' contexts.
    pub fn jetstream_streams(&self) -> &JetstreamStreams {
        &self.jetstream_streams
    }

    /// Gets a reference to the Veritech client.
    pub fn veritech(&self) -> &VeritechClient {
        &self.veritech
    }

    pub fn job_processor(&self) -> Box<dyn JobQueueProcessor + Send + Sync> {
        self.job_processor.clone()
    }

    /// Gets a reference to the encryption key.
    pub fn encryption_key(&self) -> Arc<VeritechEncryptionKey> {
        self.encryption_key.clone()
    }

    /// Get a reference to the module index url
    pub fn module_index_url(&self) -> Option<&str> {
        self.module_index_url.as_deref()
    }

    /// Get a reference to the symmetric encryption service
    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoService {
        &self.symmetric_crypto_service
    }

    /// Gets a reference to the Layer Db
    pub fn layer_db(&self) -> &DalLayerDb {
        &self.layer_db
    }

    /// Get a reference to the feature flags service
    pub fn feature_flags_service(&self) -> &FeatureFlagService {
        &self.feature_flag_service
    }

    /// Gets a reference to the dedicated compute executor
    pub fn compute_executor(&self) -> &DedicatedExecutor {
        &self.compute_executor
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

    #[allow(clippy::panic)]
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

    async fn start_txns(self) -> TransactionsResult<Self> {
        match self {
            Self::Invalid => Err(TransactionsError::TxnStart("invalid")),
            Self::Connections(conns) => Ok(Self::Transactions(conns.start_txns().await?)),
            Self::Transactions(_) => Err(TransactionsError::TxnStart("transactions")),
        }
    }

    async fn commit(
        self,
        tenancy: &Tenancy,
        layer_db: &DalLayerDb,
        rebase_request: Option<RebaseRequest>,
    ) -> TransactionsResult<Self> {
        let conns = match self {
            Self::Connections(conns) => {
                // We need to rebase and wait for the rebaser to update the change set
                // pointer, even if we are not in a "transactions" state
                if let Some(rebase_request) = rebase_request {
                    rebase(tenancy, layer_db, rebase_request).await?;
                }

                trace!("no active transactions present when commit was called");
                Ok(Self::Connections(conns))
            }
            Self::Transactions(txns) => {
                let conns = txns
                    .commit_into_conns(tenancy, layer_db, rebase_request)
                    .await?;
                Ok(Self::Connections(conns))
            }
            Self::Invalid => Err(TransactionsError::TxnCommit),
        }?;

        Ok(conns)
    }

    async fn blocking_commit(
        self,
        tenancy: &Tenancy,
        layer_db: &DalLayerDb,
        rebase_request: Option<RebaseRequest>,
    ) -> TransactionsResult<Self> {
        match self {
            Self::Connections(conns) => {
                trace!("no active transactions present when commit was called, but we will still attempt rebase");

                // Even if there are no open dal transactions, we may have written to the layer db
                // and we need to perform a rebase if one is requested
                if let Some(rebase_request) = rebase_request {
                    rebase(tenancy, layer_db, rebase_request).await?;
                }

                Ok(Self::Connections(conns))
            }
            Self::Transactions(txns) => {
                let conns = txns
                    .blocking_commit_into_conns(tenancy, layer_db, rebase_request)
                    .await?;
                Ok(Self::Connections(conns))
            }
            Self::Invalid => Err(TransactionsError::TxnCommit),
        }
    }

    async fn rollback(self) -> TransactionsResult<Self> {
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

pub enum DalContextError {}

/// A context type which holds references to underlying services, transactions, and context for DAL objects.
#[derive(Clone)] // NOTE: don't auto-derive a `Debug` implementation on this type!
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
    /// this context. Useful for builtin migrations, since we don't care about attribute values propagation then.
    no_dependent_values: bool,
    /// The workspace snapshot for this context
    workspace_snapshot: Option<Arc<WorkspaceSnapshot>>,
    /// The change set for this context
    change_set: Option<ChangeSet>,
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

    pub async fn get_workspace_default_change_set_id(&self) -> TransactionsResult<ChangeSetId> {
        let workspace_pk = self
            .tenancy()
            .workspace_pk_opt()
            .unwrap_or(WorkspacePk::NONE);
        let workspace = Workspace::get_by_pk(self, &workspace_pk)
            .await?
            .ok_or(TransactionsError::WorkspaceNotFound(workspace_pk))?;
        Ok(workspace.default_change_set_id())
    }

    pub async fn get_workspace_token(&self) -> Result<Option<String>, TransactionsError> {
        let workspace_pk = self
            .tenancy()
            .workspace_pk_opt()
            .unwrap_or(WorkspacePk::NONE);
        let workspace = Workspace::get_by_pk(self, &workspace_pk)
            .await?
            .ok_or(TransactionsError::WorkspaceNotFound(workspace_pk))?;
        Ok(workspace.token())
    }

    pub async fn get_workspace(&self) -> Result<Workspace, TransactionsError> {
        let workspace_pk = self.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE);
        let workspace = Workspace::get_by_pk(self, &workspace_pk)
            .await?
            .ok_or(TransactionsError::WorkspaceNotFound(workspace_pk))?;

        Ok(workspace)
    }

    /// Update the context to use the most recent snapshot pointed to by the current `ChangeSetId`.
    pub async fn update_snapshot_to_visibility(&mut self) -> TransactionsResult<()> {
        let change_set = ChangeSet::find(self, self.change_set_id())
            .await
            .map_err(|err| TransactionsError::ChangeSet(err.to_string()))?
            .ok_or(TransactionsError::ChangeSetNotFound(self.change_set_id()))?;

        let workspace_snapshot = WorkspaceSnapshot::find_for_change_set(self, change_set.id)
            .await
            .map_err(|err| TransactionsError::WorkspaceSnapshot(Box::new(err)))?;

        self.set_change_set(change_set)?;
        self.set_workspace_snapshot(workspace_snapshot);
        Ok(())
    }

    pub async fn write_snapshot(
        &self,
    ) -> Result<Option<WorkspaceSnapshotAddress>, TransactionsError> {
        if let Some(snapshot) = &self.workspace_snapshot {
            Ok(Some(snapshot.write(self).await.map_err(|err| {
                TransactionsError::WorkspaceSnapshot(Box::new(err))
            })?))
        } else {
            Ok(None)
        }
    }

    pub async fn do_rebase_request(&self, rebase_request: RebaseRequest) -> TransactionsResult<()> {
        rebase(&self.tenancy, &self.layer_db(), rebase_request).await?;
        Ok(())
    }

    async fn commit_internal(
        &self,
        rebase_request: Option<RebaseRequest>,
    ) -> TransactionsResult<()> {
        if self.blocking {
            self.blocking_commit_internal(rebase_request).await?;
        } else {
            let mut guard = self.conns_state.lock().await;
            *guard = guard
                .take()
                .commit(&self.tenancy, &self.layer_db(), rebase_request)
                .await?;
        };

        Ok(())
    }

    async fn blocking_commit_internal(
        &self,
        rebase_request: Option<RebaseRequest>,
    ) -> TransactionsResult<()> {
        let mut guard = self.conns_state.lock().await;
        *guard = guard
            .take()
            .blocking_commit(&self.tenancy, &self.layer_db(), rebase_request)
            .await?;

        Ok(())
    }

    pub fn to_builder(&self) -> DalContextBuilder {
        DalContextBuilder {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            no_dependent_values: self.no_dependent_values,
        }
    }

    pub async fn write_rebase_batch(
        &self,
        rebase_batch: RebaseBatch,
    ) -> TransactionsResult<RebaseBatchAddress> {
        let layer_db = self.layer_db().clone();
        let events_tenancy = self.events_tenancy();
        let events_actor = self.events_actor();

        let rebase_batch_address = slow_rt::spawn(async move {
            let (rebase_batch_address, _) = layer_db
                .rebase_batch()
                .write(Arc::new(rebase_batch), None, events_tenancy, events_actor)
                .await?;

            Ok::<RebaseBatchAddress, TransactionsError>(rebase_batch_address)
        })?
        .await??;

        Ok(rebase_batch_address)
    }

    pub async fn write_current_rebase_batch(
        &self,
    ) -> Result<Option<RebaseBatchAddress>, TransactionsError> {
        Ok(if let Some(snapshot) = &self.workspace_snapshot {
            if let Some(rebase_batch) = snapshot.current_rebase_batch().await.map_err(Box::new)? {
                Some(self.write_rebase_batch(rebase_batch).await?)
            } else {
                None
            }
        } else {
            None
        })
    }

    /// Consumes all inner transactions and committing all changes made within them.
    pub async fn commit(&self) -> TransactionsResult<()> {
        let rebase_request = self
            .write_current_rebase_batch()
            .await?
            .map(|rebase_batch_address| {
                RebaseRequest::new(self.change_set_id(), rebase_batch_address, None)
            });

        if self.blocking {
            self.blocking_commit_internal(rebase_request).await
        } else {
            self.commit_internal(rebase_request).await
        }
    }

    pub async fn commit_no_rebase(&self) -> TransactionsResult<()> {
        if self.blocking {
            self.blocking_commit_internal(None).await?;
        } else {
            self.commit_internal(None).await?;
        }

        Ok(())
    }

    pub fn change_set_id(&self) -> ChangeSetId {
        self.visibility.change_set_id
    }

    pub fn change_set(&self) -> TransactionsResult<&ChangeSet> {
        match self.change_set.as_ref() {
            Some(csp_ref) => Ok(csp_ref),
            None => Err(TransactionsError::ChangeSetNotSet),
        }
    }

    pub fn layer_db(&self) -> DalLayerDb {
        self.services_context().layer_db().clone()
    }

    /// Fetch the change set for the current change set visibility
    /// Should only be called by DalContextBuilder or by ourselves if changing visibility or
    /// refetching after a commit
    pub fn set_change_set(&mut self, change_set: ChangeSet) -> TransactionsResult<&ChangeSet> {
        // "fork" a new change set for this dal context "edit session". This gives us a new
        // Ulid generator and new vector clock id so that concurrent editing conflicts can be
        // resolved by the rebaser. This change set is not persisted to the database (the
        // rebaser will persist a new one if it can)
        self.change_set = Some(change_set);
        self.change_set()
    }

    pub fn set_workspace_snapshot(
        &mut self,
        workspace_snapshot: impl Into<Arc<WorkspaceSnapshot>>,
    ) {
        self.workspace_snapshot = Some(workspace_snapshot.into())
    }

    /// Fetch the workspace snapshot for the current visibility
    pub fn workspace_snapshot(&self) -> Result<Arc<WorkspaceSnapshot>, WorkspaceSnapshotError> {
        match &self.workspace_snapshot {
            Some(workspace_snapshot) => Ok(workspace_snapshot.clone()),
            None => Err(WorkspaceSnapshotError::WorkspaceSnapshotNotFetched),
        }
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

    pub fn symmetric_crypto_service(&self) -> &SymmetricCryptoService {
        self.services_context.symmetric_crypto_service()
    }

    /// Consumes all inner transactions, committing all changes made within them, and
    /// blocks until all queued jobs have reported as finishing.
    pub async fn blocking_commit(&self) -> TransactionsResult<()> {
        let rebase_request = self
            .write_current_rebase_batch()
            .await?
            .map(|rebase_batch_address| {
                RebaseRequest::new(self.change_set_id(), rebase_batch_address, None)
            });

        info!("rebase_request: {:?}", rebase_request);

        self.blocking_commit_internal(rebase_request).await
    }

    pub async fn blocking_commit_no_rebase(&self) -> TransactionsResult<()> {
        self.blocking_commit_internal(None).await?;
        Ok(())
    }

    /// Rolls all inner transactions back, discarding all changes made within them.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback(&self) -> TransactionsResult<()> {
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
        ctx.update_visibility_deprecated(visibility);

        fun(ctx).await
    }

    /// Updates this context with a new [`Visibility`].
    pub fn update_visibility_deprecated(&mut self, visibility: Visibility) {
        self.visibility = visibility;
    }

    /// Updates this context with a new [`Visibility`], specific to the new engine.
    pub async fn update_visibility_and_snapshot_to_visibility(
        &mut self,
        change_set_id: ChangeSetId,
    ) -> TransactionsResult<()> {
        self.update_visibility_deprecated(Visibility::new(change_set_id));
        self.update_snapshot_to_visibility().await?;
        Ok(())
    }

    /// Clones a new context from this one with a new [`Visibility`].
    pub fn clone_with_new_visibility(&self, visibility: Visibility) -> Self {
        let mut new = self.clone();
        new.update_visibility_deprecated(visibility);
        new
    }

    async fn workspace(&self) -> TransactionsResult<Workspace> {
        Ok(Workspace::get_by_pk_or_error(self, self.tenancy.workspace_pk()?).await?)
    }

    pub async fn parent_is_head(&self) -> TransactionsResult<bool> {
        let change_set = self.change_set()?;
        let base_change_set_id = change_set
            .base_change_set_id
            .ok_or(TransactionsError::NoBaseChangeSet(change_set.id))?;
        Ok(self.workspace().await?.default_change_set_id() == base_change_set_id)
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

    /// Clones a new context from this one with a "head" [`Visibility`] (default [`ChangeSet`] for
    /// the workspace).
    pub async fn clone_with_head(&self) -> TransactionsResult<Self> {
        let mut new = self.clone();
        let default_change_set_id = new.get_workspace_default_change_set_id().await?;
        new.update_visibility_and_snapshot_to_visibility(default_change_set_id)
            .await?;
        Ok(new)
    }

    /// Clones a new context from this one with a "base" [`Visibility`].
    ///
    /// _Warning:_ this only works if the current [`ChangeSet`] is not an editing [`ChangeSet`].
    pub async fn clone_with_base(&self) -> TransactionsResult<Self> {
        let change_set = self.change_set()?;
        let base_change_set_id = change_set
            .base_change_set_id
            .ok_or(TransactionsError::NoBaseChangeSet(change_set.id))?;

        let mut new = self.clone();
        new.update_visibility_and_snapshot_to_visibility(base_change_set_id)
            .await?;
        Ok(new)
    }

    pub async fn enqueue_action(&self, job: Box<ActionJob>) -> TransactionsResult<()> {
        self.txns().await?.job_queue.enqueue_job(job).await;
        Ok(())
    }

    /// Add the node ids to the workspace snapshot graph and enqueue a dependent values update.
    /// This update will only be run on commit if blocking_commit is used. If commit is used, the
    /// DVU debouncer will run the job. Note that the DVU debouncer might still pick up the job
    /// before blocking_commit does, so blocking_commit might do extra work.
    pub async fn add_dependent_values_and_enqueue(
        &self,
        ids: Vec<impl Into<si_events::ulid::Ulid>>,
    ) -> Result<(), WorkspaceSnapshotError> {
        for id in ids {
            self.workspace_snapshot()?
                .add_dependent_value_root(DependentValueRoot::Unfinished(id.into()))
                .await?;
        }

        self.enqueue_dependent_values_update().await?;

        Ok(())
    }

    /// Adds a dependent values update job to the queue. Most users will instead want to use
    /// [`Self::add_dependent_values_and_enqueue`] which will add the values that need to be
    /// processed to the graph, and enqueue the job.
    pub async fn enqueue_dependent_values_update(&self) -> TransactionsResult<()> {
        // The values that the DVU job will process are part of the snapshot now
        let empty_vec: Vec<ulid::Ulid> = vec![];
        self.txns()
            .await?
            .job_queue
            .enqueue_attribute_value_job(
                self.change_set_id(),
                self.access_builder(),
                AttributeValueBasedJobIdentifier::DependentValuesUpdate,
                empty_vec,
            )
            .await;

        Ok(())
    }

    #[instrument(
    level="info",
    skip_all,
    fields(
            si.change_set.id = Empty,
            si.attribute_value_id,
        ),
    )]
    pub async fn enqueue_compute_validations(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> TransactionsResult<()> {
        let span = Span::current();
        span.record("si.change_set.id", &self.change_set_id().to_string());

        self.txns()
            .await?
            .job_queue
            .enqueue_attribute_value_job(
                self.change_set_id(),
                self.access_builder(),
                AttributeValueBasedJobIdentifier::ComputeValidation,
                vec![attribute_value_id],
            )
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
    pub fn encryption_key(&self) -> &VeritechEncryptionKey {
        &self.services_context.encryption_key
    }

    /// Gets a reference to the dal context's tenancy.
    pub fn tenancy(&self) -> &Tenancy {
        &self.tenancy
    }

    /// Gets the version of tenancy used by the layerdb/si-events crate
    pub fn events_tenancy(&self) -> si_events::Tenancy {
        si_events::Tenancy {
            change_set_id: self.change_set_id().into(),
            workspace_pk: self
                .tenancy()
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .into(),
        }
    }

    /// Gets the version of the "actor" (UserPk) used by the layerdb/si-events-crate
    pub fn events_actor(&self) -> si_events::Actor {
        match self.history_actor() {
            HistoryActor::User(user_pk) => si_events::Actor::User((*user_pk).into()),
            HistoryActor::SystemInit => si_events::Actor::System,
        }
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
    pub async fn check_tenancy<T: StandardModel>(&self, object: &T) -> TransactionsResult<bool> {
        let is_in_our_tenancy = self
            .tenancy()
            .check(self.txns().await?.pg(), object.tenancy())
            .await?;

        Ok(is_in_our_tenancy)
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

/// A request context builder which requires a [`Visibility`] to be completed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
#[derive(Clone)]
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

impl fmt::Debug for DalContextBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DalContextBuilder")
            .field("blocking", &self.blocking)
            .field("no_dependent_values", &self.no_dependent_values)
            .finish_non_exhaustive()
    }
}

impl DalContextBuilder {
    /// Constructs and returns a new [`DalContext`] using a default [`RequestContext`].
    pub async fn build_default(&self) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new_empty(),
            visibility: Visibility::new_head_fake(),
            history_actor: HistoryActor::SystemInit,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
        })
    }

    /// Constructs and returns a new [`DalContext`] from a [`WorkspacePk`] and [`ChangeSetId`] as
    /// the system user.
    pub async fn build_for_change_set_as_system(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        let mut ctx = DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new(workspace_pk),
            visibility: Visibility::new(change_set_id),
            history_actor: HistoryActor::SystemInit,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
        };

        ctx.update_snapshot_to_visibility().await?;

        Ok(ctx)
    }

    /// Constructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build_head(
        &self,
        access_builder: AccessBuilder,
    ) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        let mut ctx = DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: access_builder.tenancy,
            history_actor: access_builder.history_actor,
            visibility: Visibility::new_head_fake(),
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
        };

        // TODO(nick): there's a chicken and egg problem here. We want a dal context to get the
        // workspace's default change set id, but we are going to use a dummy visibility to do so.
        // We should probably just use the pg connection directly or derive the default change set
        // id through other means.
        let default_change_set_id = ctx.get_workspace_default_change_set_id().await?;
        ctx.update_visibility_and_snapshot_to_visibility(default_change_set_id)
            .await?;

        Ok(ctx)
    }

    /// Constructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build(&self, request_context: RequestContext) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        let mut ctx = DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: request_context.tenancy,
            visibility: request_context.visibility,
            history_actor: request_context.history_actor,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
        };

        if ctx.history_actor() != &HistoryActor::SystemInit {
            let user_workspaces: HashSet<WorkspacePk> = Workspace::list_for_user(&ctx)
                .await?
                .iter()
                .map(Workspace::pk)
                .copied()
                .collect();
            if let Some(workspace_pk) = request_context.tenancy.workspace_pk_opt() {
                let workspace_has_change_set =
                    Workspace::has_change_set(&ctx, request_context.visibility.change_set_id)
                        .await?;
                // We want to make sure that *BOTH* the Workspace requested is one that the user has
                // access to, *AND* that the Change Set requested is one of the Change Sets for _that_
                // workspace.
                if !(user_workspaces.contains(&workspace_pk) && workspace_has_change_set) {
                    return Err(TransactionsError::BadWorkspaceAndChangeSet);
                }
            }
        }

        ctx.update_snapshot_to_visibility().await?;

        Ok(ctx)
    }

    /// Gets a reference to the PostgreSQL connection pool.
    pub fn pg_pool(&self) -> &PgPool {
        &self.services_context.pg_pool
    }

    /// Gets a reference to the NATS connection.
    pub fn nats_conn(&self) -> &NatsClient {
        &self.services_context.nats_conn
    }

    /// Returns the location on disk where packages are stored (if one was provided)
    pub async fn pkgs_path(&self) -> Option<&PathBuf> {
        self.services_context.pkgs_path.as_ref()
    }

    /// Gets a reference to the [`ServicesContext`].
    pub fn services_context(&self) -> &ServicesContext {
        &self.services_context
    }

    /// Gets a reference to the LayerDb.
    pub fn layer_db(&self) -> &DalLayerDb {
        &self.services_context.layer_db
    }

    /// Gets a reference to the compute [`DedicatedExecutor`].
    pub fn compute_executor(&self) -> &DedicatedExecutor {
        &self.services_context.compute_executor
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
#[derive(Debug, Error, EnumDiscriminants)]
pub enum TransactionsError {
    #[error("expected a {0:?} activity, but received a {1:?}")]
    BadActivity(ActivityPayloadDiscriminants, ActivityPayloadDiscriminants),
    /// Intentionally a bit vague as its used when either the user in question doesn't have access
    /// to the requested Workspace, or the Change Set requested isn't part of the Workspace that
    /// was specified.
    #[error("Bad Workspace & Change Set")]
    BadWorkspaceAndChangeSet,
    #[error("change set error: {0}")]
    ChangeSet(String),
    #[error("change set not found for change set id: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("change set not set on DalContext")]
    ChangeSetNotSet,
    #[error(transparent)]
    JobQueueProcessor(#[from] JobQueueProcessorError),
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error(transparent)]
    LayerDb(#[from] LayerDbError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("no base change set for change set: {0}")]
    NoBaseChangeSet(ChangeSetId),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error("rebase of batch {0} for change set id {1} failed: {2}")]
    RebaseFailed(RebaseBatchAddress, ChangeSetId, String),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("slow rt error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error(transparent)]
    Tenancy(#[from] TenancyError),
    #[error("Unable to acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("cannot commit transactions on invalid connections state")]
    TxnCommit,
    #[error("cannot rollback transactions on invalid connections state")]
    TxnRollback,
    #[error("cannot start transactions without connections; state={0}")]
    TxnStart(&'static str),
    #[error("workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("workspace not found by pk: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("workspace not set on DalContext")]
    WorkspaceNotSet,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
}

pub type TransactionsResult<T> = Result<T, TransactionsError>;

impl From<WorkspaceError> for TransactionsError {
    fn from(err: WorkspaceError) -> Self {
        TransactionsError::Workspace(Box::new(err))
    }
}

impl TransactionsError {
    pub fn is_unmigrated_snapshot_error(&self) -> bool {
        match self {
            TransactionsError::WorkspaceSnapshot(boxed_err) => matches!(
                boxed_err.as_ref(),
                WorkspaceSnapshotError::WorkspaceSnapshotNotMigrated(_)
            ),
            _ => false,
        }
    }
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
    pub async fn start_txns(self) -> TransactionsResult<Transactions> {
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
    job_queue: JobQueue,
}

#[derive(Clone, Debug)]
pub struct RebaseRequest {
    pub to_rebase_change_set_id: ChangeSetId,
    pub rebase_batch_address: RebaseBatchAddress,
    pub from_change_set_id: Option<ChangeSetId>,
}

impl RebaseRequest {
    pub fn new(
        to_rebase_change_set_id: ChangeSetId,
        rebase_batch_address: RebaseBatchAddress,
        from_change_set_id: Option<ChangeSetId>,
    ) -> Self {
        Self {
            to_rebase_change_set_id,
            rebase_batch_address,
            from_change_set_id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Updates {
    pub updates_found: Vec<Update>,
}

// TODO(nick): we need to determine the long term vision for tenancy-scoped subjects. We're leaking the tenancy into
// the connection state functions. I believe it is fine for now since rebasing is a very specific use case, but we may
// not want it long term.
#[instrument(
    level="info",
    skip_all,
    fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
            si.conflicts = Empty,
            si.updates = Empty,
            si.conflicts.count = Empty,
            si.updates.count = Empty,
        ),
    )]
async fn rebase(
    tenancy: &Tenancy,
    layer_db: &DalLayerDb,
    rebase_request: RebaseRequest,
) -> TransactionsResult<()> {
    let start = Instant::now();
    let span = Span::current();

    let metadata = LayeredEventMetadata::new(
        si_events::Tenancy::new(
            tenancy
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .into(),
            rebase_request.to_rebase_change_set_id.into(),
        ),
        si_events::Actor::System,
    );
    span.record(
        "si.change_set.id",
        rebase_request.to_rebase_change_set_id.to_string(),
    );
    span.record(
        "si.workspace.id",
        tenancy
            .workspace_pk_opt()
            .unwrap_or(WorkspacePk::NONE)
            .to_string(),
    );
    info!("requesting rebase: {:?}", start.elapsed());
    let rebase_finished_activity = layer_db
        .activity()
        .rebase()
        .rebase_and_wait(
            rebase_request.to_rebase_change_set_id.into(),
            rebase_request.from_change_set_id.map(Into::into),
            rebase_request.rebase_batch_address,
            metadata,
        )
        .await?;
    info!("got response from rebaser: {:?}", start.elapsed());
    debug!(
        "rebaser response payload: {:?}",
        rebase_finished_activity.payload
    );
    match rebase_finished_activity.payload {
        ActivityPayload::RebaseFinished(rebase_finished) => match rebase_finished.status() {
            RebaseStatus::Success {
                updates_performed: _,
            } => {
                //span.record("si.updates", updates_performed);
                //span.record("si.updates.count", updates.updates_found.len().to_string());
                Ok(())
            }
            RebaseStatus::Error { message } => Err(TransactionsError::RebaseFailed(
                rebase_request.rebase_batch_address,
                rebase_request.to_rebase_change_set_id,
                message.to_string(),
            )),
        },
        p => Err(TransactionsError::BadActivity(
            ActivityPayloadDiscriminants::RebaseFinished,
            p.into(),
        )),
    }
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
            job_queue: JobQueue::new(),
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
    #[instrument(
        name = "transactions.commit_into_conns",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn commit_into_conns(
        self,
        tenancy: &Tenancy,
        layer_db: &DalLayerDb,
        rebase_request: Option<RebaseRequest>,
    ) -> TransactionsResult<Connections> {
        let span = Span::current();
        span.record(
            "si.workspace.id",
            tenancy
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .to_string(),
        );
        let pg_conn = self.pg_txn.commit_into_conn().await?;

        if let Some(rebase_request) = rebase_request {
            span.record(
                "si.change_set.id",
                rebase_request.to_rebase_change_set_id.to_string(),
            );

            // remove the dependent value job since it will be handled by the rebaser
            let _ = self
                .job_queue
                .take_dependent_values_for_change_set(rebase_request.to_rebase_change_set_id)
                .await;
            rebase(tenancy, layer_db, rebase_request).await?;
        };

        let nats_conn = self.nats_txn.commit_into_conn().await?;
        self.job_processor.process_queue(self.job_queue).await?;

        Ok(Connections::new(pg_conn, nats_conn, self.job_processor))
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// underlying connections. Blocking until all queued jobs have reported as finishing.
    #[instrument(
        name = "transactions.blocking_commit_into_conns",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
        )
    )]
    pub async fn blocking_commit_into_conns(
        self,
        tenancy: &Tenancy,
        layer_db: &DalLayerDb,
        rebase_request: Option<RebaseRequest>,
    ) -> TransactionsResult<Connections> {
        let span = Span::current();
        span.record(
            "si.workspace.id",
            tenancy
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .to_string(),
        );
        let pg_conn = self.pg_txn.commit_into_conn().await?;

        if let Some(rebase_request) = rebase_request {
            span.record(
                "si.change_set.id",
                rebase_request.to_rebase_change_set_id.to_string(),
            );
            rebase(tenancy, layer_db, rebase_request).await?;
        }

        let nats_conn = self.nats_txn.commit_into_conn().await?;

        self.job_processor
            .blocking_process_queue(self.job_queue)
            .await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor);

        Ok(conns)
    }

    /// Rolls all inner transactions back, discarding all changes made within them, and returns
    /// underlying connections.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback_into_conns(self) -> TransactionsResult<Connections> {
        let pg_conn = self.pg_txn.rollback_into_conn().await?;
        let nats_conn = self.nats_txn.rollback_into_conn().await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor);

        Ok(conns)
    }

    /// Rolls all inner transactions back, discarding all changes made within them.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback(self) -> TransactionsResult<()> {
        let _ = self.rollback_into_conns().await?;
        Ok(())
    }
}
