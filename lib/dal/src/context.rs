use std::{collections::HashMap, collections::HashSet, mem, path::PathBuf, sync::Arc};

use content_store::{PgStore, StoreError};
use futures::Future;
use rebaser_client::ClientError as RebaserClientError;
use rebaser_client::Config as RebaserClientConfig;
use rebaser_client::ReplyRebaseMessage;
use serde::{Deserialize, Serialize};
use si_crypto::SymmetricCryptoService;
use si_data_nats::{NatsClient, NatsError, NatsTxn};
use si_data_pg::{InstrumentedClient, PgError, PgPool, PgPoolError, PgPoolResult, PgTxn};
use si_layer_cache::layer_cache::LayerCacheDependencies;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};
use tokio::time::Instant;
use ulid::Ulid;
use veritech_client::{Client as VeritechClient, CycloneEncryptionKey};

use crate::workspace_snapshot::conflict::Conflict;
use crate::workspace_snapshot::update::Update;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::workspace_snapshot::WorkspaceSnapshotId;
use crate::{
    change_set_pointer::{ChangeSetPointer, ChangeSetPointerId},
    job::{
        processor::{JobQueueProcessor, JobQueueProcessorError},
        producer::{BlockingJobError, BlockingJobResult, JobProducer},
        queue::JobQueue,
    },
    workspace_snapshot::WorkspaceSnapshotError,
    AttributeValueId, ComponentId, HistoryActor, StandardModel, Tenancy, TenancyError, Visibility,
    WorkspacePk, WorkspaceSnapshot,
};
use crate::{ChangeSetPk, Workspace};

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
    encryption_key: Arc<CycloneEncryptionKey>,
    /// The path where available packages can be found
    pkgs_path: Option<PathBuf>,
    /// The URL of the module index
    module_index_url: Option<String>,
    /// A service that can encrypt and decrypt values with a set of symmetric keys
    symmetric_crypto_service: SymmetricCryptoService,
    /// Config for the the rebaser service
    rebaser_config: RebaserClientConfig,
    /// Content store
    content_store_pg_pool: PgPool,
    /// Dependencies for the layer cache (sled and postgres)
    layer_cache_dependencies: LayerCacheDependencies,
}

impl ServicesContext {
    /// Constructs a new instance of a `ServicesContext`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pg_pool: PgPool,
        nats_conn: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        veritech: VeritechClient,
        encryption_key: Arc<CycloneEncryptionKey>,
        pkgs_path: Option<PathBuf>,
        module_index_url: Option<String>,
        symmetric_crypto_service: SymmetricCryptoService,
        rebaser_config: RebaserClientConfig,
        content_store_pg_pool: PgPool,
        layer_cache_dependencies: LayerCacheDependencies,
    ) -> Self {
        Self {
            pg_pool,
            nats_conn,
            job_processor,
            veritech,
            encryption_key,
            pkgs_path,
            module_index_url,
            symmetric_crypto_service,
            rebaser_config,
            content_store_pg_pool,
            layer_cache_dependencies,
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
    pub fn encryption_key(&self) -> Arc<CycloneEncryptionKey> {
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

    /// Gets a reference to the rebaser client configuration
    pub fn rebaser_config(&self) -> &RebaserClientConfig {
        &self.rebaser_config
    }

    /// Gets a reference to the content store pg pool
    pub fn content_store_pg_pool(&self) -> &PgPool {
        &self.content_store_pg_pool
    }

    pub fn layer_cache_dependencies(&self) -> &LayerCacheDependencies {
        &self.layer_cache_dependencies
    }

    /// Builds and returns a new [`content_store::PgStore`]
    pub async fn content_store(&self) -> content_store::StoreResult<PgStore> {
        PgStore::new(self.content_store_pg_pool().clone()).await
    }

    /// Builds and returns a new [`Connections`].
    pub async fn connections(&self) -> PgPoolResult<Connections> {
        let pg_conn = self.pg_pool.get().await?;
        let nats_conn = self.nats_conn.clone();
        let job_processor = self.job_processor.clone();
        let rebaser_config = self.rebaser_config.clone();

        Ok(Connections::new(
            pg_conn,
            nats_conn,
            job_processor,
            rebaser_config,
        ))
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

    async fn commit(
        self,
        tenancy: &Tenancy,
        rebase_request: Option<RebaseRequest>,
    ) -> Result<(Self, Option<Conflicts>), TransactionsError> {
        match self {
            Self::Connections(_) => {
                trace!("no active transactions present when commit was called, taking no action");
                Ok((self, None))
            }
            Self::Transactions(txns) => {
                let (conns, conflicts) = txns.commit_into_conns(tenancy, rebase_request).await?;
                Ok((Self::Connections(conns), conflicts))
            }
            Self::Invalid => Err(TransactionsError::TxnCommit),
        }
    }

    async fn blocking_commit(
        self,
        tenancy: &Tenancy,
        rebase_request: Option<RebaseRequest>,
    ) -> Result<(Self, Option<Conflicts>), TransactionsError> {
        match self {
            Self::Connections(_) => {
                trace!("no active transactions present when commit was called, taking no action");
                Ok((self, None))
            }
            Self::Transactions(txns) => {
                let (conns, conflicts) = txns
                    .blocking_commit_into_conns(tenancy, rebase_request)
                    .await?;
                Ok((Self::Connections(conns), conflicts))
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

pub enum DalContextError {}

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
    /// this context. Useful for builtin migrations, since we don't care about attribute values propagation then.
    no_dependent_values: bool,
    /// The content-addressable [`store`](content_store::Store) used by the "dal".
    ///
    /// This should be configurable in the future, but for now, the only kind of store used is the
    /// [`PgStore`](content_store::PgStore).
    content_store: Arc<Mutex<PgStore>>,
    /// The workspace snapshot for this context
    workspace_snapshot: Option<Arc<WorkspaceSnapshot>>,
    /// The change set pointer for this context
    change_set_pointer: Option<ChangeSetPointer>,
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

    pub async fn get_workspace_default_change_set_id(
        &self,
    ) -> Result<ChangeSetPointerId, TransactionsError> {
        let workspace = Workspace::get_by_pk(
            self,
            &self.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE),
        )
        .await
        // use a proper error
        .map_err(|err| TransactionsError::ChangeSet(err.to_string()))?;

        let cs_id = workspace
            .map(|workspace| workspace.default_change_set_id())
            .unwrap_or(ChangeSetPointerId::NONE);

        Ok(cs_id)
    }

    pub async fn update_snapshot_to_visibility(&mut self) -> Result<(), TransactionsError> {
        let change_set_id = match self.change_set_id() {
            ChangeSetPointerId::NONE => self.get_workspace_default_change_set_id().await?,
            other => other,
        };

        let change_set_pointer = ChangeSetPointer::find(self, change_set_id)
            .await
            .map_err(|err| TransactionsError::ChangeSet(err.to_string()))?
            .ok_or(TransactionsError::ChangeSetPointerNotFound(
                self.change_set_id(),
            ))?;

        let workspace_snapshot =
            WorkspaceSnapshot::find_for_change_set(self, change_set_pointer.id)
                .await
                .map_err(|err| TransactionsError::WorkspaceSnapshot(err.to_string()))?;

        self.set_change_set_pointer(change_set_pointer)?;
        self.set_workspace_snapshot(workspace_snapshot);

        Ok(())
    }

    pub async fn update_snapshot_to_visibility_no_editing_change_set(
        &mut self,
    ) -> Result<(), TransactionsError> {
        let change_set_id = match self.change_set_id() {
            ChangeSetPointerId::NONE => self.get_workspace_default_change_set_id().await?,
            other => other,
        };

        let change_set_pointer = ChangeSetPointer::find(self, change_set_id)
            .await
            .map_err(|err| TransactionsError::ChangeSet(err.to_string()))?
            .ok_or(TransactionsError::ChangeSetPointerNotFound(
                self.change_set_id(),
            ))?;

        let workspace_snapshot =
            WorkspaceSnapshot::find_for_change_set(self, change_set_pointer.id)
                .await
                .map_err(|err| TransactionsError::WorkspaceSnapshot(err.to_string()))?;

        self.change_set_pointer = Some(change_set_pointer);
        self.set_workspace_snapshot(workspace_snapshot);

        Ok(())
    }

    pub async fn write_snapshot(&self) -> Result<Option<WorkspaceSnapshotId>, TransactionsError> {
        if let Some(snapshot) = &self.workspace_snapshot {
            let vector_clock_id = self.change_set_pointer()?.vector_clock_id();

            Ok(Some(snapshot.write(self, vector_clock_id).await.map_err(
                |err| TransactionsError::WorkspaceSnapshot(err.to_string()),
            )?))
        } else {
            Ok(None)
        }
    }

    fn get_rebase_request(
        &self,
        onto_workspace_snapshot_id: WorkspaceSnapshotId,
    ) -> Result<RebaseRequest, TransactionsError> {
        let vector_clock_id = self.change_set_pointer()?.vector_clock_id();
        Ok(RebaseRequest {
            onto_workspace_snapshot_id,
            // the vector clock id of the current change set is just the id
            // of the current change set
            to_rebase_change_set_id: self.change_set_id(),
            onto_vector_clock_id: vector_clock_id,
        })
    }

    pub async fn do_rebase_request(
        &self,
        rebase_request: RebaseRequest,
    ) -> Result<Option<Conflicts>, TransactionsError> {
        rebase(
            &self.tenancy,
            self.services_context.nats_conn.clone(),
            self.services_context().rebaser_config.clone(),
            rebase_request,
        )
        .await
    }

    async fn commit_internal(
        &self,
        rebase_request: Option<RebaseRequest>,
    ) -> Result<Option<Conflicts>, TransactionsError> {
        let conflicts = if self.blocking {
            self.blocking_commit_internal(rebase_request).await?
        } else {
            let mut guard = self.conns_state.lock().await;
            let (new_guard, conflicts) = guard.take().commit(&self.tenancy, rebase_request).await?;
            *guard = new_guard;

            conflicts
        };

        Ok(conflicts)
    }

    async fn blocking_commit_internal(
        &self,
        rebase_request: Option<RebaseRequest>,
    ) -> Result<Option<Conflicts>, TransactionsError> {
        let mut guard = self.conns_state.lock().await;

        let (new_guard, conflicts) = guard
            .take()
            .blocking_commit(&self.tenancy, rebase_request)
            .await?;
        *guard = new_guard;

        Ok(conflicts)
    }

    pub fn to_builder(&self) -> DalContextBuilder {
        DalContextBuilder {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            no_dependent_values: self.no_dependent_values,
        }
    }

    /// Consumes all inner transactions and committing all changes made within them.
    pub async fn commit(&self) -> Result<Option<Conflicts>, TransactionsError> {
        let rebase_request = match self.write_snapshot().await? {
            Some(workspace_snapshot_id) => Some(self.get_rebase_request(workspace_snapshot_id)?),
            None => None,
        };

        Ok(if self.blocking {
            self.blocking_commit_internal(rebase_request).await?
        } else {
            self.commit_internal(rebase_request).await?
        })
    }

    pub async fn commit_no_rebase(&self) -> Result<(), TransactionsError> {
        if self.blocking {
            self.blocking_commit_internal(None).await?;
        } else {
            self.commit_internal(None).await?;
        }

        Ok(())
    }

    pub fn change_set_pointer(&self) -> Result<&ChangeSetPointer, TransactionsError> {
        match self.change_set_pointer.as_ref() {
            Some(csp_ref) => Ok(csp_ref),
            None => Err(TransactionsError::ChangeSetPointerNotSet),
        }
    }

    /// Fetch the change set pointer for the current change set visibility
    /// Should only be called by DalContextBuilder or by ourselves if changing visibility or
    /// refetching after a commit
    pub fn set_change_set_pointer(
        &mut self,
        change_set_pointer: ChangeSetPointer,
    ) -> Result<&ChangeSetPointer, TransactionsError> {
        // "fork" a new change set pointer for this dal context "edit session". This gives us a new
        // Ulid generator and new vector clock id so that concurrent editing conflicts can be
        // resolved by the rebaser. This change set pointer is not persisted to the database (the
        // rebaser will persist a new one if it can)
        self.change_set_pointer = Some(
            change_set_pointer
                .editing_changeset()
                .map_err(|err| TransactionsError::ChangeSet(err.to_string()))?,
        );

        self.change_set_pointer()
    }

    pub fn set_workspace_snapshot(&mut self, workspace_snapshot: WorkspaceSnapshot) {
        self.workspace_snapshot = Some(Arc::new(workspace_snapshot));
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
    pub async fn blocking_commit(&self) -> Result<Option<Conflicts>, TransactionsError> {
        info!("blocking_commit");
        let rebase_request = match self.write_snapshot().await? {
            Some(workspace_snapshot_id) => Some(self.get_rebase_request(workspace_snapshot_id)?),
            None => None,
        };

        self.blocking_commit_internal(rebase_request).await
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

    /// Updates this context with a new [`Visibility`], specific to the new engine.
    pub async fn update_visibility_v2(
        &mut self,
        change_set_v2: &ChangeSetPointer,
    ) -> Result<(), TransactionsError> {
        self.update_visibility(Visibility::new(
            ChangeSetPk::from(Ulid::from(change_set_v2.id)),
            None,
        ));
        self.update_snapshot_to_visibility().await?;
        Ok(())
    }

    pub async fn update_visibility_v2_no_editing_change_set(
        &mut self,
        change_set_v2: &ChangeSetPointer,
    ) -> Result<(), TransactionsError> {
        self.update_visibility(Visibility::new(
            ChangeSetPk::from(Ulid::from(change_set_v2.id)),
            None,
        ));
        self.update_snapshot_to_visibility_no_editing_change_set()
            .await?;
        Ok(())
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

    pub async fn parent_is_head(&self) -> bool {
        if let Some(workspace_pk) = self.tenancy.workspace_pk() {
            let workspace = Workspace::get_by_pk(self, &workspace_pk)
                .await
                .unwrap()
                .unwrap();
            workspace.default_change_set_id()
                == self
                    .change_set_pointer()
                    .unwrap()
                    .base_change_set_id
                    .unwrap()
        } else {
            false
        }
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

    pub async fn enqueue_dependencies_update_component(
        &self,
        component_id: ComponentId,
    ) -> Result<(), TransactionsError> {
        self.txns()
            .await?
            .enqueue_dependencies_update_component(
                *self.tenancy(),
                self.visibility().change_set_pk,
                component_id,
            )
            .await;
        Ok(())
    }

    pub async fn enqueue_job(
        &self,
        job: Box<dyn JobProducer + Sync + Send>,
    ) -> Result<(), TransactionsError> {
        self.txns().await?.job_queue.enqueue_job(job).await;
        Ok(())
    }

    // pub async fn enqueue_fix(&self, job: Box<FixesJob>) -> Result<(), TransactionsError> {
    //     self.txns().await?.job_queue.enqueue_job(job).await;
    //     Ok(())
    // }
    //

    // pub async fn enqueue_refresh(&self, job: Box<RefreshJob>) -> Result<(), TransactionsError> {
    //     self.txns().await?.job_queue.enqueue_job(job).await;
    //     Ok(())
    // }

    pub async fn enqueue_dependent_values_update(
        &self,
        ids: Vec<AttributeValueId>,
    ) -> Result<(), TransactionsError> {
        self.txns()
            .await?
            .job_queue
            .enqueue_dependent_values_update(
                self.visibility().change_set_pk,
                self.access_builder(),
                ids,
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
    pub fn encryption_key(&self) -> &CycloneEncryptionKey {
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

    /// Gets a reference to the content store.
    pub fn content_store(&self) -> &Arc<Mutex<PgStore>> {
        &self.content_store
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
    #[instrument(level = "debug", skip_all)]
    pub async fn import_builtins(&self) -> Result<(), TransactionsError> {
        // TODO(nick,zack,jacob): restore the ability to "import builtins" via the graph work.
        // let source_workspace_pk = WorkspacePk::NONE;
        // self.txns()
        //     .await?
        //     .pg()
        //     .execute(
        //         "SELECT import_builtins_v1($1, $2)",
        //         &[self.tenancy(), &source_workspace_pk],
        //     )
        //     .await?;
        Ok(())
    }

    // NOTE(nick,zack,jacob): likely a temporary func to get the change set id from the visibility.
    pub fn change_set_id(&self) -> ChangeSetPointerId {
        ChangeSetPointerId::from(Ulid::from(self.visibility.change_set_pk))
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
    /// Constructs and returns a new [`DalContext`] using a default [`RequestContext`].
    pub async fn build_default(&self) -> Result<DalContext, TransactionsError> {
        let conns = self.services_context.connections().await?;
        // should we move this into Connections?
        let content_store = self.services_context.content_store().await?;

        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new_empty(),
            visibility: Visibility::new_head(false),
            history_actor: HistoryActor::SystemInit,
            content_store: Arc::new(Mutex::new(content_store)),
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set_pointer: None,
        })
    }

    /// Constructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build_head(
        &self,
        access_builder: AccessBuilder,
    ) -> Result<DalContext, TransactionsError> {
        let conns = self.services_context.connections().await?;
        let content_store = self.services_context.content_store().await?;

        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: access_builder.tenancy,
            history_actor: access_builder.history_actor,
            visibility: Visibility::new_head(false),
            no_dependent_values: self.no_dependent_values,
            content_store: Arc::new(Mutex::new(content_store)),
            workspace_snapshot: None,
            change_set_pointer: None,
        })
    }

    /// Constructs and returns a new [`DalContext`] using a [`RequestContext`].
    pub async fn build(
        &self,
        request_context: RequestContext,
    ) -> Result<DalContext, TransactionsError> {
        let conns = self.services_context.connections().await?;
        let content_store = self.services_context.content_store().await?;
        let mut ctx = DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: request_context.tenancy,
            visibility: request_context.visibility,
            history_actor: request_context.history_actor,
            no_dependent_values: self.no_dependent_values,
            content_store: Arc::new(Mutex::new(content_store)),
            workspace_snapshot: None,
            change_set_pointer: None,
        };

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
    #[error("change set error: {0}")]
    ChangeSet(String),
    #[error("change set pointer not found for change set id: {0}")]
    ChangeSetPointerNotFound(ChangeSetPointerId),
    #[error("Change set pointer not set on DalContext")]
    ChangeSetPointerNotSet,
    #[error(transparent)]
    JobQueueProcessor(#[from] JobQueueProcessorError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error("rebase of snapshot {0} change set id {1} failed {2}")]
    RebaseFailed(WorkspaceSnapshotId, ChangeSetPointerId, String),
    #[error(transparent)]
    RebaserClient(#[from] RebaserClientError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
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
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(String),
}

/// A type which holds ownership over connections that can be used to start transactions.
#[derive(Debug)]
pub struct Connections {
    pg_conn: InstrumentedClient,
    nats_conn: NatsClient,
    rebaser_config: RebaserClientConfig,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
}

impl Connections {
    /// Builds a new [`Connections`].
    #[must_use]
    pub fn new(
        pg_conn: InstrumentedClient,
        nats_conn: NatsClient,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        rebaser_config: RebaserClientConfig,
    ) -> Self {
        Self {
            pg_conn,
            nats_conn,
            rebaser_config,
            job_processor,
        }
    }

    /// Starts and returns a [`Transactions`].
    pub async fn start_txns(self) -> Result<Transactions, TransactionsError> {
        let pg_txn = PgTxn::create(self.pg_conn).await?;
        let nats_txn = self.nats_conn.transaction();
        let job_processor = self.job_processor;
        let rebaser_config = self.rebaser_config;

        Ok(Transactions::new(
            pg_txn,
            nats_txn,
            job_processor,
            rebaser_config,
        ))
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
    /// Rebaser client
    rebaser_config: RebaserClientConfig,
    job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
    job_queue: JobQueue,
    #[allow(clippy::type_complexity)]
    dependencies_update_component:
        Arc<Mutex<HashMap<(Tenancy, ChangeSetPk), HashSet<ComponentId>>>>,
}

#[derive(Clone, Debug)]
pub struct RebaseRequest {
    pub to_rebase_change_set_id: ChangeSetPointerId,
    pub onto_workspace_snapshot_id: WorkspaceSnapshotId,
    pub onto_vector_clock_id: VectorClockId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Conflicts {
    conflicts_found: Vec<Conflict>,
    updates_found_and_skipped: Vec<Update>,
}

// TODO(nick): we need to determine the long term vision for tenancy-scoped subjects. We're leaking the tenancy into
// the connection state functions. I believe it is fine for now since rebasing is a very specific use case, but we may
// not want it long term.
async fn rebase(
    tenancy: &Tenancy,
    nats: NatsClient,
    rebaser_config: RebaserClientConfig,
    rebase_request: RebaseRequest,
) -> Result<Option<Conflicts>, TransactionsError> {
    let start = Instant::now();

    // TODO(nick): make this cleaner.
    let workspace_id = tenancy.workspace_pk().unwrap_or(WorkspacePk::NONE).into();
    let rebaser_client = rebaser_client::Client::new(nats, rebaser_config, workspace_id);

    info!("got client and requesting rebase: {:?}", start.elapsed());
    let response = rebaser_client
        .request_rebase(
            rebase_request.to_rebase_change_set_id.into(),
            rebase_request.onto_workspace_snapshot_id.into(),
            rebase_request.onto_vector_clock_id.into(),
        )
        .await?;
    info!("got response from rebaser: {:?}", start.elapsed());

    match response {
        ReplyRebaseMessage::Success { .. } => Ok(None),
        ReplyRebaseMessage::Error { message } => Err(TransactionsError::RebaseFailed(
            rebase_request.onto_workspace_snapshot_id,
            rebase_request.to_rebase_change_set_id,
            message,
        )),
        ReplyRebaseMessage::ConflictsFound {
            conflicts_found,
            updates_found_and_skipped,
        } => {
            let conflicts = Conflicts {
                conflicts_found: serde_json::from_value(conflicts_found)?,
                updates_found_and_skipped: serde_json::from_value(updates_found_and_skipped)?,
            };

            Ok(Some(conflicts))
        }
    }
}

impl Transactions {
    /// Creates and returns a new `Transactions` instance.
    fn new(
        pg_txn: PgTxn,
        nats_txn: NatsTxn,
        job_processor: Box<dyn JobQueueProcessor + Send + Sync>,
        rebaser_config: RebaserClientConfig,
    ) -> Self {
        Self {
            pg_txn,
            nats_txn,
            rebaser_config,
            job_processor,
            job_queue: JobQueue::new(),
            dependencies_update_component: Default::default(),
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
        fields()
    )]
    pub async fn commit_into_conns(
        self,
        tenancy: &Tenancy,
        rebase_request: Option<RebaseRequest>,
    ) -> Result<(Connections, Option<Conflicts>), TransactionsError> {
        let pg_conn = self.pg_txn.commit_into_conn().await?;
        let nats_conn = self.nats_txn.commit_into_conn().await?;

        let conflicts = if let Some(rebase_request) = rebase_request {
            let start = Instant::now();
            let conflicts = rebase(
                tenancy,
                nats_conn.clone(),
                self.rebaser_config.clone(),
                rebase_request,
            )
            .await?;
            info!("rebase took: {:?}", start.elapsed());
            conflicts
        } else {
            None
        };

        self.job_processor.process_queue(self.job_queue).await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor, self.rebaser_config);

        Ok((conns, conflicts))
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// underlying connections. Blocking until all queued jobs have reported as finishing.
    #[instrument(
        name = "transactions.blocking_commit_into_conns",
        level = "info",
        skip_all,
        fields()
    )]
    pub async fn blocking_commit_into_conns(
        self,
        tenancy: &Tenancy,
        rebase_request: Option<RebaseRequest>,
    ) -> Result<(Connections, Option<Conflicts>), TransactionsError> {
        let pg_conn = self.pg_txn.commit_into_conn().await?;
        let nats_conn = self.nats_txn.commit_into_conn().await?;

        let conflicts = if let Some(rebase_request) = rebase_request {
            info!("rebase request");
            rebase(
                tenancy,
                nats_conn.clone(),
                self.rebaser_config.clone(),
                rebase_request,
            )
            .await?
        } else {
            None
        };

        self.job_processor
            .blocking_process_queue(self.job_queue)
            .await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor, self.rebaser_config);

        Ok((conns, conflicts))
    }

    /// Rolls all inner transactions back, discarding all changes made within them, and returns
    /// underlying connections.
    ///
    /// This is equivalent to the transaction's `Drop` implementations, but provides any error
    /// encountered to the caller.
    pub async fn rollback_into_conns(self) -> Result<Connections, TransactionsError> {
        let pg_conn = self.pg_txn.rollback_into_conn().await?;
        let nats_conn = self.nats_txn.rollback_into_conn().await?;
        let conns = Connections::new(pg_conn, nats_conn, self.job_processor, self.rebaser_config);

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

    pub async fn enqueue_dependencies_update_component(
        &self,
        tenancy: Tenancy,
        change_set_pk: ChangeSetPk,
        component_id: ComponentId,
    ) {
        self.dependencies_update_component
            .lock()
            .await
            .entry((tenancy, change_set_pk))
            .or_default()
            .insert(component_id);
    }

    #[instrument(level = "info", skip_all)]
    async fn run_dependencies_update_component(&self) -> Result<(), TransactionsError> {
        for ((tenancy, change_set_pk), component_ids) in
            std::mem::take(&mut *self.dependencies_update_component.lock().await)
        {
            for component_id in component_ids {
                let visibility = Visibility::new(change_set_pk, None);
                self.pg()
                    .execute(
                        "SELECT attribute_value_dependencies_update_component_v1($1, $2, $3)",
                        &[&tenancy, &visibility, &component_id],
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
