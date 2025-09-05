use std::{
    collections::HashSet,
    fmt,
    mem,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use concurrent_extensions::ConcurrentExtensions;
use futures::{
    Future,
    future::BoxFuture,
};
use rebaser_client::{
    RebaserClient,
    RequestId,
    api_types::enqueue_updates_response::{
        EnqueueUpdatesResponse,
        RebaseStatus,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use si_crypto::{
    SymmetricCryptoService,
    VeritechEncryptionKey,
};
use si_data_nats::{
    NatsClient,
    NatsError,
    NatsTxn,
    jetstream,
};
use si_data_pg::{
    InstrumentedClient,
    PgError,
    PgPool,
    PgPoolError,
    PgPoolResult,
    PgTxn,
};
use si_db::{
    HistoryActor,
    SiDbContext,
    SiDbTransactions,
    SiDbTransactionsError,
    Tenancy,
    Visibility,
};
use si_events::{
    AuthenticationMethod,
    EventSessionId,
    RebaseBatchAddressKind,
    WorkspaceSnapshotAddress,
    audit_log::AuditLogKind,
    change_batch::{
        ChangeBatch,
        ChangeBatchAddress,
    },
    rebase_batch_address::RebaseBatchAddress,
    split_snapshot_rebase_batch_address::SplitSnapshotRebaseBatchAddress,
    workspace_snapshot::Change,
};
use si_id::{
    ActionId,
    ComponentId,
    ManagementPrototypeId,
    ViewId,
};
use si_layer_cache::{
    LayerDbError,
    activities::ActivityPayloadDiscriminants,
    db::LayerDb,
};
use si_runtime::DedicatedExecutor;
use si_split_graph::SuperGraph;
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use telemetry_utils::metric;
use thiserror::Error;
use tokio::{
    sync::{
        MappedMutexGuard,
        Mutex,
        MutexGuard,
    },
    time,
};
use tokio_util::task::TaskTracker;
use veritech_client::Client as VeritechClient;

use crate::{
    AttributeValueId,
    ChangeSetError,
    EncryptedSecret,
    Workspace,
    WorkspaceError,
    WorkspacePk,
    WorkspaceSnapshot,
    audit_logging::{
        self,
        AuditLoggingError,
    },
    change_set::{
        ChangeSet,
        ChangeSetId,
    },
    feature_flags::FeatureFlagService,
    jetstream_streams::JetstreamStreams,
    job::{
        consumer::DalJob,
        processor::{
            JobQueueProcessor,
            JobQueueProcessorError,
        },
        producer::{
            BlockingJobError,
            BlockingJobResult,
        },
        queue::JobQueue,
    },
    layer_db_types::ContentTypes,
    slow_rt::{
        self,
        SlowRuntimeError,
    },
    workspace_snapshot::{
        DependentValueRoot,
        WorkspaceSnapshotError,
        WorkspaceSnapshotResult,
        WorkspaceSnapshotSelector,
        dependent_value_root::DependentValueRootError,
        graph::{
            RebaseBatch,
            WorkspaceSnapshotGraph,
        },
        split_snapshot::{
            SplitRebaseBatchVCurrent,
            SplitSnapshot,
            SubGraphVCurrent,
        },
    },
};

pub type DalLayerDb = LayerDb<
    ContentTypes,
    EncryptedSecret,
    WorkspaceSnapshotGraph,
    RebaseBatch,
    SubGraphVCurrent,
    SuperGraph,
    SplitRebaseBatchVCurrent,
>;

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
    /// A Rebaser client, connected via a NATS connection.
    rebaser: RebaserClient,
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
        rebaser: RebaserClient,
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
            rebaser,
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

    /// Gets a reference to the Rebaser client.
    pub fn rebaser(&self) -> &RebaserClient {
        &self.rebaser
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

    fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid)
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

    async fn start_txns(self) -> Result<Self, SiDbTransactionsError> {
        match self {
            Self::Invalid => Err(SiDbTransactionsError::TxnStart("invalid")),
            Self::Connections(conns) => Ok(Self::Transactions(conns.start_txns().await?)),
            Self::Transactions(_) => Err(SiDbTransactionsError::TxnStart("transactions")),
        }
    }

    async fn commit(self, maybe_rebase: DelayedRebaseWithReply<'_>) -> TransactionsResult<Self> {
        let conns = match self {
            Self::Connections(conns) => {
                // We need to rebase and wait for the rebaser to update the change set
                // pointer, even if we are not in a "transactions" state
                if let DelayedRebaseWithReply::WithUpdates {
                    rebaser,
                    workspace_pk,
                    change_set_id,
                    updates_address,
                    event_session_id,
                } = maybe_rebase
                {
                    rebase_with_reply(
                        rebaser,
                        workspace_pk,
                        change_set_id,
                        updates_address,
                        event_session_id,
                    )
                    .await?;
                }

                trace!("no active transactions present when commit was called");
                Ok(Self::Connections(conns))
            }
            Self::Transactions(txns) => {
                let conns = txns.commit_into_conns(maybe_rebase).await?;
                Ok(Self::Connections(conns))
            }
            Self::Invalid => Err(TransactionsError::TxnCommit),
        }?;

        Ok(conns)
    }

    async fn blocking_commit(
        self,
        maybe_rebase: DelayedRebaseWithReply<'_>,
    ) -> TransactionsResult<Self> {
        match self {
            Self::Connections(conns) => {
                trace!(
                    "no active transactions present when commit was called, but we will still attempt rebase"
                );

                // Even if there are no open dal transactions, we may have written to the layer db
                // and we need to perform a rebase if one is requested
                if let DelayedRebaseWithReply::WithUpdates {
                    rebaser,
                    workspace_pk,
                    change_set_id,
                    updates_address,
                    event_session_id,
                } = maybe_rebase
                {
                    rebase_with_reply(
                        rebaser,
                        workspace_pk,
                        change_set_id,
                        updates_address,
                        event_session_id,
                    )
                    .await?;
                }

                Ok(Self::Connections(conns))
            }
            Self::Transactions(txns) => {
                let conns = txns.blocking_commit_into_conns(maybe_rebase).await?;
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
    workspace_snapshot: Option<WorkspaceSnapshotSelector>,
    /// The change set for this context
    change_set: Option<ChangeSet>,
    /// The event session identifier
    event_session_id: EventSessionId,
    /// The request ulid coming from the client
    request_ulid: Option<ulid::Ulid>,
    /// The authentication method used
    authentication_method: AuthenticationMethod,
    /// A type cache of data which saves on constant re-fetching
    cache: ConcurrentExtensions,
}

#[async_trait]
impl SiDbContext for DalContext {
    type Transactions = self::Transactions;

    fn history_actor(&self) -> &HistoryActor {
        self.history_actor()
    }

    async fn txns(
        &self,
    ) -> Result<MappedMutexGuard<'_, Self::Transactions>, SiDbTransactionsError> {
        self.txns_internal().await
    }

    fn tenancy(&self) -> &Tenancy {
        self.tenancy()
    }

    fn visibility(&self) -> &Visibility {
        self.visibility()
    }

    fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id()
    }
}

impl SiDbTransactions for Transactions {
    fn pg(&self) -> &PgTxn {
        self.pg()
    }

    fn nats(&self) -> &NatsTxn {
        self.nats()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct WorkspaceDefaultChangeSetId {
    default_change_set_id: ChangeSetId,
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
        if let Some(cached) = self.cache.get::<WorkspaceDefaultChangeSetId>() {
            return Ok(cached.default_change_set_id);
        }

        let workspace_pk = self.tenancy().workspace_pk()?;
        let default_change_set_id = Workspace::get_by_pk(self, workspace_pk)
            .await?
            .default_change_set_id();

        self.cache.insert(WorkspaceDefaultChangeSetId {
            default_change_set_id,
        });

        Ok(default_change_set_id)
    }

    pub async fn get_workspace_token(&self) -> Result<Option<String>, TransactionsError> {
        let workspace_pk = self
            .tenancy()
            .workspace_pk_opt()
            .unwrap_or(WorkspacePk::NONE);
        let workspace = Workspace::get_by_pk(self, workspace_pk).await?;
        Ok(workspace.token())
    }

    pub async fn get_workspace(&self) -> Result<Workspace, TransactionsError> {
        Ok(Workspace::get_by_pk(self, self.tenancy.workspace_pk()?).await?)
    }

    pub async fn get_workspace_or_builtin(&self) -> Result<Workspace, TransactionsError> {
        let workspace_pk = self.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE);
        let workspace = Workspace::get_by_pk(self, workspace_pk).await?;

        Ok(workspace)
    }

    /// Update the context to use the most recent snapshot pointed to by the current [`ChangeSetId`].
    /// Note: This does not guarantee that the [`ChangeSetId`] is contained within the [`WorkspacePk`]
    /// for the current [`DalContext`]
    pub async fn update_snapshot_to_visibility(&mut self) -> TransactionsResult<()> {
        let change_set = ChangeSet::get_by_id_across_workspaces(self, self.change_set_id()).await?;
        let workspace = self.get_workspace().await?;

        self.workspace_snapshot = Some(
            workspace
                .snapshot_for_change_set(self, change_set.id)
                .await?,
        );

        self.set_change_set(change_set)?;

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

    pub async fn run_rebase_with_reply(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddressKind,
    ) -> TransactionsResult<()> {
        rebase_with_reply(
            self.rebaser(),
            workspace_pk,
            change_set_id,
            updates_address,
            self.event_session_id,
        )
        .await
    }

    pub async fn run_async_rebase_from_change_set(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddressKind,
        from_change_set_id: ChangeSetId,
    ) -> TransactionsResult<RequestId> {
        self.rebaser()
            .enqueue_updates_from_change_set(
                workspace_pk,
                change_set_id,
                updates_address,
                from_change_set_id,
                self.event_session_id,
            )
            .await
            .map_err(Into::into)
    }

    pub async fn run_rebase_from_change_set_with_reply(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddressKind,
        from_change_set_id: ChangeSetId,
    ) -> TransactionsResult<(
        RequestId,
        BoxFuture<'static, Result<EnqueueUpdatesResponse, rebaser_client::ClientError>>,
    )> {
        self.rebaser()
            .enqueue_updates_from_change_set_with_reply(
                workspace_pk,
                change_set_id,
                updates_address,
                from_change_set_id,
                self.event_session_id,
            )
            .await
            .map_err(Into::into)
    }

    async fn commit_internal(
        &self,
        rebase_batch: Option<RebaseBatchAddressKind>,
    ) -> TransactionsResult<()> {
        let maybe_rebase = match rebase_batch {
            Some(updates_address) => DelayedRebaseWithReply::WithUpdates {
                rebaser: self.rebaser(),
                workspace_pk: self.workspace_pk()?,
                change_set_id: self.change_set_id(),
                updates_address,
                event_session_id: self.event_session_id,
            },
            None => {
                // Since we are not rebasing, we need to write the final message and flush all
                // pending audit logs.
                self.bless_audit_logs_infallible_wrapper().await;
                DelayedRebaseWithReply::NoUpdates
            }
        };

        if self.blocking {
            self.blocking_commit_internal(maybe_rebase).await?;
        } else {
            let mut guard = self.conns_state.lock().await;
            *guard = guard.take().commit(maybe_rebase).await?;
        };

        Ok(())
    }

    async fn blocking_commit_internal(
        &self,
        maybe_rebase: DelayedRebaseWithReply<'_>,
    ) -> TransactionsResult<()> {
        let mut guard = self.conns_state.lock().await;
        *guard = guard.take().blocking_commit(maybe_rebase).await?;

        Ok(())
    }

    pub fn to_builder(&self) -> DalContextBuilder {
        DalContextBuilder {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            no_dependent_values: self.no_dependent_values,
        }
    }

    #[instrument(name = "context.write_change_batch", level = "debug", skip_all)]
    pub async fn write_change_batch(
        &self,
        changes: Vec<Change>,
    ) -> TransactionsResult<ChangeBatchAddress> {
        let layer_db = self.layer_db().clone();
        let events_tenancy = self.events_tenancy();
        let events_actor = self.events_actor();

        let change_batch_address = slow_rt::spawn(async move {
            let (change_batch_address, _) = layer_db.change_batch().write(
                Arc::new(ChangeBatch::new(changes)),
                None,
                events_tenancy,
                events_actor,
            )?;

            Ok::<ChangeBatchAddress, TransactionsError>(change_batch_address)
        })?
        .await??;

        Ok(change_batch_address)
    }

    #[instrument(name = "context.write_rebase_batch", level = "debug", skip_all)]
    pub async fn write_legacy_rebase_batch(
        &self,
        rebase_batch: RebaseBatch,
    ) -> TransactionsResult<RebaseBatchAddress> {
        let layer_db = self.layer_db().clone();
        let events_tenancy = self.events_tenancy();
        let events_actor = self.events_actor();

        let rebase_batch_address = slow_rt::spawn(async move {
            let (rebase_batch_address, _) = layer_db.rebase_batch().write(
                Arc::new(rebase_batch),
                None,
                events_tenancy,
                events_actor,
            )?;

            Ok::<RebaseBatchAddress, TransactionsError>(rebase_batch_address)
        })?
        .await??;

        Ok(rebase_batch_address)
    }

    #[instrument(
        name = "context.write_split_snapshot_rebase_batch",
        level = "debug",
        skip_all
    )]
    pub async fn write_split_snapshot_rebase_batch(
        &self,
        rebase_batch: SplitRebaseBatchVCurrent,
    ) -> TransactionsResult<SplitSnapshotRebaseBatchAddress> {
        let layer_db = self.layer_db().clone();
        let events_tenancy = self.events_tenancy();
        let events_actor = self.events_actor();

        let rebase_batch_address = slow_rt::spawn(async move {
            let (rebase_batch_address, _) = layer_db.split_snapshot_rebase_batch().write(
                Arc::new(rebase_batch),
                None,
                events_tenancy,
                events_actor,
            )?;

            Ok::<SplitSnapshotRebaseBatchAddress, TransactionsError>(rebase_batch_address)
        })?
        .await??;

        Ok(rebase_batch_address)
    }

    #[instrument(name = "context.write_current_rebase_batch", level = "debug", skip_all)]
    async fn write_current_rebase_batch(
        &self,
    ) -> Result<Option<RebaseBatchAddressKind>, TransactionsError> {
        Ok(match self.workspace_snapshot.as_ref() {
            Some(WorkspaceSnapshotSelector::LegacySnapshot(legacy_snapshot)) => {
                match legacy_snapshot
                    .current_rebase_batch()
                    .await
                    .map_err(Box::new)?
                {
                    Some(rebase_batch) => Some(RebaseBatchAddressKind::Legacy(
                        self.write_legacy_rebase_batch(rebase_batch).await?,
                    )),
                    None => None,
                }
            }
            Some(WorkspaceSnapshotSelector::SplitSnapshot(split_snapshot)) => {
                match split_snapshot
                    .current_rebase_batch()
                    .await
                    .map_err(Box::new)?
                {
                    Some(rebase_batch) => Some(RebaseBatchAddressKind::Split(
                        self.write_split_snapshot_rebase_batch(rebase_batch).await?,
                    )),
                    None => None,
                }
            }
            None => None,
        })
    }

    pub async fn detect_changes_from_head(&self) -> WorkspaceSnapshotResult<Vec<Change>> {
        let head_change_set_id = self.get_workspace_default_change_set_id().await?;

        match &self.workspace_snapshot()? {
            WorkspaceSnapshotSelector::LegacySnapshot(workspace_snapshot) => {
                let head_snapshot =
                    WorkspaceSnapshot::find_for_change_set(self, head_change_set_id).await?;
                head_snapshot.detect_changes(workspace_snapshot).await
            }
            WorkspaceSnapshotSelector::SplitSnapshot(split_snapshot) => {
                let head_snapshot =
                    SplitSnapshot::find_for_change_set(self, head_change_set_id).await?;
                head_snapshot.detect_changes(split_snapshot).await
            }
        }
    }

    /// Consumes all inner transactions and committing all changes made within them.
    #[instrument(name = "context.commit", level = "info", skip_all)]
    pub async fn commit(&self) -> TransactionsResult<()> {
        let rebase_batch = self.write_current_rebase_batch().await?;
        self.commit_internal(rebase_batch).await
    }

    #[instrument(name = "context.commit_no_rebase", level = "info", skip_all)]
    pub async fn commit_no_rebase(&self) -> TransactionsResult<()> {
        self.commit_internal(None).await
    }

    pub fn workspace_pk(&self) -> TransactionsResult<WorkspacePk> {
        self.tenancy.workspace_pk().map_err(Into::into)
    }

    pub fn change_set_id(&self) -> ChangeSetId {
        self.visibility.change_set_id
    }

    pub fn authentication_method(&self) -> AuthenticationMethod {
        self.authentication_method
    }

    pub fn change_set(&self) -> TransactionsResult<&ChangeSet> {
        match self.change_set.as_ref() {
            Some(csp_ref) => Ok(csp_ref),
            None => Err(TransactionsError::ChangeSetNotSet),
        }
    }

    pub fn event_session_id(&self) -> EventSessionId {
        self.event_session_id
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

    pub fn set_workspace_split_snapshot(&mut self, snapshot: impl Into<Arc<SplitSnapshot>>) {
        self.workspace_snapshot = Some(WorkspaceSnapshotSelector::SplitSnapshot(snapshot.into()));
    }

    pub fn set_workspace_snapshot(
        &mut self,
        workspace_snapshot: impl Into<Arc<WorkspaceSnapshot>>,
    ) {
        self.workspace_snapshot = Some(WorkspaceSnapshotSelector::LegacySnapshot(
            workspace_snapshot.into(),
        ));
    }

    /// Fetch the workspace snapshot for the current visibility
    pub fn workspace_snapshot(&self) -> Result<WorkspaceSnapshotSelector, WorkspaceSnapshotError> {
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
        let maybe_rebase = match self.write_current_rebase_batch().await? {
            Some(updates_address) => DelayedRebaseWithReply::WithUpdates {
                rebaser: self.rebaser(),
                workspace_pk: self.workspace_pk()?,
                change_set_id: self.change_set_id(),
                updates_address,
                event_session_id: self.event_session_id,
            },
            None => {
                // Since we are not rebasing, we need to write the final message and flush all
                // pending audit logs.
                self.bless_audit_logs_infallible_wrapper().await;
                DelayedRebaseWithReply::NoUpdates
            }
        };

        self.blocking_commit_internal(maybe_rebase).await
    }

    pub async fn blocking_commit_no_rebase(&self) -> TransactionsResult<()> {
        self.blocking_commit_internal(DelayedRebaseWithReply::NoUpdates)
            .await?;
        Ok(())
    }

    /// Start with a new connection state in this context, with new pg pool and
    /// nats connections, and an empty job queue.
    pub async fn restart_connections(&mut self) -> TransactionsResult<()> {
        self.conns_state = Arc::new(Mutex::new(ConnectionState::new_from_conns(
            self.services_context().connections().await?,
        )));

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

    pub async fn is_head(&self) -> TransactionsResult<bool> {
        Ok(self.get_workspace_default_change_set_id().await? == self.change_set_id())
    }

    pub async fn parent_is_head(&self) -> TransactionsResult<bool> {
        let change_set = self.change_set()?;
        let base_change_set_id = change_set
            .base_change_set_id
            .ok_or(TransactionsError::NoBaseChangeSet(change_set.id))?;

        Ok(self.get_workspace_default_change_set_id().await? == base_change_set_id)
    }

    /// Updates this context with a new [`Tenancy`]
    pub fn update_tenancy(&mut self, tenancy: Tenancy) {
        // Bust cache as we're updating tenancy (i.e. workspace_pk)
        self.cache.remove::<WorkspaceDefaultChangeSetId>();

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

    pub async fn enqueue_action_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        action_id: ActionId,
    ) -> TransactionsResult<()> {
        self.txns()
            .await?
            .job_queue
            .enqueue_action_job(workspace_id, change_set_id, action_id)
            .await;
        Ok(())
    }

    /// Add the node ids to the workspace snapshot graph and enqueue a dependent values update.
    /// This update will only be run on commit if blocking_commit is used. If commit is used, the
    /// DVU debouncer will run the job. Note that the DVU debouncer might still pick up the job
    /// before blocking_commit does, so blocking_commit might do extra work.
    pub async fn add_dependent_values_and_enqueue(
        &self,
        ids: Vec<impl Into<si_events::ulid::Ulid>>,
    ) -> Result<(), DependentValueRootError> {
        for id in ids {
            DependentValueRoot::add_dependent_value_root(
                self,
                DependentValueRoot::Unfinished(id.into()),
            )
            .await?;
        }

        self.enqueue_dependent_values_update().await?;

        Ok(())
    }

    /// Adds a dependent values update job to the queue. Most users will instead want to use
    /// [`Self::add_dependent_values_and_enqueue`] which will add the values that need to be
    /// processed to the graph, and enqueue the job.
    async fn enqueue_dependent_values_update(&self) -> TransactionsResult<()> {
        self.txns()
            .await?
            .job_queue
            .enqueue_dependent_values_update_job(self.workspace_pk()?, self.change_set_id())
            .await;

        Ok(())
    }

    #[instrument(
        name = "dal_context.enqueue_compute_validations",
        level = "info",
        skip_all,
        fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
        ),
    )]
    pub async fn enqueue_compute_validations(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> TransactionsResult<()> {
        let span = current_span_for_instrument_at!("info");

        span.record("si.change_set.id", self.change_set_id().to_string());
        span.record("si.workspace.id", self.workspace_pk()?.to_string());

        self.txns()
            .await?
            .job_queue
            .enqueue_validation_job(
                self.workspace_pk()?,
                self.change_set_id(),
                attribute_value_id,
            )
            .await;

        Ok(())
    }

    pub async fn enqueue_management_func(
        &self,
        prototype_id: ManagementPrototypeId,
        component_id: ComponentId,
        view_id: ViewId,
        request_ulid: ulid::Ulid,
    ) -> TransactionsResult<()> {
        self.txns()
            .await?
            .job_queue
            .enqueue_management_func_job(
                self.workspace_pk()?,
                self.change_set_id(),
                prototype_id,
                component_id,
                view_id,
                // TODO(nick): make this required.
                Some(request_ulid),
            )
            .await;

        Ok(())
    }

    /// Similar to `enqueue_job`, except that instead of waiting to flush the job to
    /// the processing system on `commit`, the job is immediately flushed, and the
    /// processor is expected to not return until the job has finished. Returns the
    /// result of executing the job.
    pub async fn block_on_job(&self, job: Box<dyn DalJob>) -> BlockingJobResult {
        self.txns()
            .await
            .map_err(|err| BlockingJobError::Transactions(err.to_string()))?
            .job_processor
            .block_on_job(job)
            .await
    }

    /// Gets the dal context's txns.
    pub async fn txns(&self) -> Result<MappedMutexGuard<'_, Transactions>, TransactionsError> {
        // This is just to convert error types
        Ok(self.txns_internal().await?)
    }

    // TODO instead of doing this error juke, we should move Transactions to a common place
    // shared by dal and si-db
    async fn txns_internal(
        &self,
    ) -> Result<MappedMutexGuard<'_, Transactions>, SiDbTransactionsError> {
        let mut guard = self.conns_state.lock().await;

        let conns_state = guard.take();

        if conns_state.is_conns() {
            // If we are Connections, then we need to start Transactions
            *guard = conns_state.start_txns().await?;
        } else if conns_state.is_invalid() {
            return Err(SiDbTransactionsError::ConnStateInvalid);
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

    // Gets a reference to the DAL context's Rebaser client.
    //
    // **NOTE**: Internal API
    fn rebaser(&self) -> &RebaserClient {
        &self.services_context.rebaser
    }

    /// Gets a reference to the DAL context's compute executor.
    pub fn compute_executor(&self) -> &DedicatedExecutor {
        &self.services_context.compute_executor
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
            change_set_id: self.change_set_id(),
            workspace_pk: self
                .tenancy()
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE),
        }
    }

    /// Gets the version of the "actor" (UserPk) used by the layerdb/si-events-crate
    pub fn events_actor(&self) -> si_events::Actor {
        match self.history_actor() {
            HistoryActor::User(user_pk) => si_events::Actor::User(*user_pk),
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

    pub fn access_builder(&self) -> AccessBuilder {
        AccessBuilder::new(
            self.tenancy,
            self.history_actor,
            self.request_ulid,
            self.authentication_method,
        )
    }

    /// Returns a new [`jetstream::Context`].
    pub fn jetstream_context(&self) -> jetstream::Context {
        jetstream::new(self.nats_conn().to_owned())
    }

    /// Convenience wrapper around [`audit_logging::write`].
    #[instrument(name = "dal_context.write_audit_log", level = "debug", skip_all)]
    pub async fn write_audit_log(
        &self,
        kind: AuditLogKind,
        entity_name: String,
    ) -> TransactionsResult<()> {
        Ok(audit_logging::write(self, kind, entity_name, None).await?)
    }

    /// Convenience wrapper around [`audit_logging::write`] that writes to HEAD.
    #[instrument(
        name = "dal_context.write_audit_log_to_head",
        level = "debug",
        skip_all
    )]
    pub async fn write_audit_log_to_head(
        &self,
        kind: AuditLogKind,
        entity_name: String,
    ) -> TransactionsResult<()> {
        let head_change_set_id = self.get_workspace_default_change_set_id().await?;
        Ok(audit_logging::write(self, kind, entity_name, Some(head_change_set_id)).await?)
    }

    /// Convenience wrapper around [`audit_logging::write_final_message`].
    #[instrument(
        name = "dal_context.audit_log_write_final_message",
        level = "debug",
        skip_all
    )]
    async fn write_audit_log_final_message(&self) -> TransactionsResult<()> {
        Ok(audit_logging::write_final_message(self).await?)
    }

    /// Convenience wrapper around [`audit_logging::publish_pending`].
    #[instrument(
        name = "dal_context.publish_pending_audit_logs",
        level = "debug",
        skip_all
    )]
    pub async fn publish_pending_audit_logs(
        &self,
        tracker: Option<TaskTracker>,
        override_event_session_id: Option<si_events::EventSessionId>,
    ) -> TransactionsResult<()> {
        Ok(audit_logging::publish_pending(self, tracker, override_event_session_id).await?)
    }

    #[instrument(
        name = "dal_context.blest_audit_logs_infallible_wrapper",
        level = "debug",
        skip_all
    )]
    async fn bless_audit_logs_infallible_wrapper(&self) {
        match self.write_audit_log_final_message().await {
            Ok(()) => match self.publish_pending_audit_logs(None, None).await {
                Ok(()) => {}
                Err(err) => error!(si.error.message = ?err, "unable to publish pending audit logs"),
            },
            Err(err) => error!(si.error.message = ?err, "unable to write final audit log"),
        }
    }

    pub fn request_ulid(&self) -> Option<ulid::Ulid> {
        self.request_ulid
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
    /// A potentially suitable ulid generated by the front end
    pub request_ulid: Option<ulid::Ulid>,
    /// Authentication method of the actor
    pub authentication_method: AuthenticationMethod,
}

/// A request context builder which requires a [`Visibility`] to be completed.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccessBuilder {
    /// A suitable tenancy for the consuming DAL objects.
    tenancy: Tenancy,
    /// A suitable [`HistoryActor`] for the consuming DAL objects.
    history_actor: HistoryActor,
    /// A potentially suitable ulid generated by the front end
    request_ulid: Option<ulid::Ulid>,
    /// The authentication method for the [`history_actor`]
    /// None means it's the [`HistoryActor::System`]
    authentication_method: AuthenticationMethod,
}

impl AccessBuilder {
    /// Constructs a new instance given a tenancy and a [`HistoryActor`].
    pub fn new(
        tenancy: Tenancy,
        history_actor: HistoryActor,
        request_ulid: Option<ulid::Ulid>,
        authentication_method: AuthenticationMethod,
    ) -> Self {
        Self {
            tenancy,
            history_actor,
            request_ulid,
            authentication_method,
        }
    }

    /// Builds and returns a new [`RequestContext`] using the given [`Visibility`].
    pub fn build(self, visibility: Visibility) -> RequestContext {
        RequestContext {
            tenancy: self.tenancy,
            history_actor: self.history_actor,
            request_ulid: self.request_ulid,
            authentication_method: self.authentication_method,
            visibility,
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
    pub async fn build_default(
        &self,
        request_ulid: Option<ulid::Ulid>,
    ) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new_empty(),
            visibility: Visibility::new_head_fake(),
            history_actor: HistoryActor::SystemInit,
            request_ulid,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
            event_session_id: EventSessionId::new(),
            authentication_method: AuthenticationMethod::System,
            cache: Default::default(),
        })
    }

    /// Constructs and returns a new [`DalContext`] with no home workspace or change set.
    /// For admin-ish requests that are workspace-independent.
    pub async fn build_without_workspace(
        &self,
        history_actor: HistoryActor,
        request_ulid: Option<ulid::Ulid>,
        authentication_method: AuthenticationMethod,
    ) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        Ok(DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new_empty(),
            visibility: Visibility::new_head_fake(),
            history_actor,
            request_ulid,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
            event_session_id: EventSessionId::new(),
            authentication_method,
            cache: Default::default(),
        })
    }

    /// Constructs and returns a new [`DalContext`] from a [`WorkspacePk`] and [`ChangeSetId`] as
    /// the system user.
    pub async fn build_for_change_set_as_system(
        &self,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        request_ulid: Option<ulid::Ulid>,
    ) -> TransactionsResult<DalContext> {
        let conns = self.services_context.connections().await?;

        let mut ctx = DalContext {
            services_context: self.services_context.clone(),
            blocking: self.blocking,
            conns_state: Arc::new(Mutex::new(ConnectionState::new_from_conns(conns))),
            tenancy: Tenancy::new(workspace_pk),
            visibility: Visibility::new(change_set_id),
            history_actor: HistoryActor::SystemInit,
            request_ulid,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
            event_session_id: EventSessionId::new(),
            authentication_method: AuthenticationMethod::System,
            cache: Default::default(),
        };

        ctx.update_snapshot_to_visibility().await?;

        Ok(ctx)
    }

    /// Constructs and returns a new [`DalContext`] using an [`AccessBuilder`].
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
            request_ulid: access_builder.request_ulid,
            visibility: Visibility::new_head_fake(),
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
            event_session_id: EventSessionId::new(),
            authentication_method: access_builder.authentication_method,
            cache: Default::default(),
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
            request_ulid: request_context.request_ulid,
            no_dependent_values: self.no_dependent_values,
            workspace_snapshot: None,
            change_set: None,
            event_session_id: EventSessionId::new(),
            authentication_method: request_context.authentication_method,
            cache: Default::default(),
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
    #[error("audit logging error: {0}")]
    AuditLogging(#[from] Box<AuditLoggingError>),
    #[error("expected a {0:?} activity, but received a {1:?}")]
    BadActivity(ActivityPayloadDiscriminants, ActivityPayloadDiscriminants),
    /// Intentionally a bit vague as its used when either the user in question doesn't have access
    /// to the requested Workspace, or the Change Set requested isn't part of the Workspace that
    /// was specified.
    #[error("Bad Workspace & Change Set")]
    BadWorkspaceAndChangeSet,
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("change set not set on DalContext")]
    ChangeSetNotSet,
    #[error("transactions cannot be run when connection state invalid")]
    ConnStateInvalid,
    #[error("job queue processor error: {0}")]
    JobQueueProcessor(#[from] Box<JobQueueProcessorError>),
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] Box<LayerDbError>),
    #[error("nats error: {0}")]
    Nats(#[from] NatsError),
    #[error("no base change set for change set: {0}")]
    NoBaseChangeSet(ChangeSetId),
    #[error("pg error: {0}")]
    Pg(#[from] Box<PgError>),
    #[error("pg pool error: {0}")]
    PgPool(#[from] Box<PgPoolError>),
    #[error("rebase of batch {0} for change set id {1} failed: {2}")]
    RebaseFailed(RebaseBatchAddressKind, ChangeSetId, String),
    #[error("rebaser client error: {0}")]
    Rebaser(#[from] Box<rebaser_client::ClientError>),
    #[error("rebaser reply deadline elapsed; waited={0:?}, request_id={1}")]
    RebaserReplyDeadlineElasped(Duration, RequestId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] Box<si_db::Error>),
    #[error("slow rt error: {0}")]
    SlowRuntime(#[from] Box<SlowRuntimeError>),
    #[error("unable to acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("cannot commit transactions on invalid connections state")]
    TxnCommit,
    #[error("cannot rollback transactions on invalid connections state")]
    TxnRollback,
    #[error("cannot start transactions without connections; state={0}")]
    TxnStart(&'static str),
    #[error("workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("workspace not set on DalContext")]
    WorkspaceNotSet,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
}

pub type TransactionsResult<T> = Result<T, TransactionsError>;

impl From<AuditLoggingError> for TransactionsError {
    fn from(value: AuditLoggingError) -> Self {
        Box::new(value).into()
    }
}

impl From<JobQueueProcessorError> for TransactionsError {
    fn from(value: JobQueueProcessorError) -> Self {
        Box::new(value).into()
    }
}

impl From<LayerDbError> for TransactionsError {
    fn from(value: LayerDbError) -> Self {
        Box::new(value).into()
    }
}

impl From<PgError> for TransactionsError {
    fn from(value: PgError) -> Self {
        Box::new(value).into()
    }
}

impl From<PgPoolError> for TransactionsError {
    fn from(value: PgPoolError) -> Self {
        Box::new(value).into()
    }
}

impl From<rebaser_client::ClientError> for TransactionsError {
    fn from(value: rebaser_client::ClientError) -> Self {
        Box::new(value).into()
    }
}

impl From<si_db::Error> for TransactionsError {
    fn from(value: si_db::Error) -> Self {
        Box::new(value).into()
    }
}

impl From<SlowRuntimeError> for TransactionsError {
    fn from(value: SlowRuntimeError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceError> for TransactionsError {
    fn from(err: WorkspaceError) -> Self {
        Box::new(err).into()
    }
}

impl From<ChangeSetError> for TransactionsError {
    fn from(err: ChangeSetError) -> Self {
        Box::new(err).into()
    }
}

impl From<SiDbTransactionsError> for TransactionsError {
    fn from(err: SiDbTransactionsError) -> Self {
        match err {
            SiDbTransactionsError::Pg(err) => TransactionsError::Pg(Box::new(err)),
            SiDbTransactionsError::TxnStart(state) => TransactionsError::TxnStart(state),
            SiDbTransactionsError::ConnStateInvalid => TransactionsError::ConnStateInvalid,
        }
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
    pub async fn start_txns(self) -> Result<Transactions, PgError> {
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
            job_queue: JobQueue::default(),
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

    pub fn job_queue(&self) -> &JobQueue {
        &self.job_queue
    }

    /// Consumes all inner transactions, committing all changes made within them, and returns
    /// underlying connections.
    #[instrument(name = "transactions.commit_into_conns", level = "info", skip_all)]
    pub async fn commit_into_conns(
        self,
        maybe_rebase: DelayedRebaseWithReply<'_>,
    ) -> TransactionsResult<Connections> {
        let pg_conn = self.pg_txn.commit_into_conn().await?;

        if let DelayedRebaseWithReply::WithUpdates {
            rebaser,
            workspace_pk,
            change_set_id,
            updates_address,
            event_session_id,
        } = maybe_rebase
        {
            // remove the dependent value job since it will be handled by the rebaser
            self.job_queue.clear_dependent_values_jobs().await;
            rebase_with_reply(
                rebaser,
                workspace_pk,
                change_set_id,
                updates_address,
                event_session_id,
            )
            .await?;
        }

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
        maybe_rebase: DelayedRebaseWithReply<'_>,
    ) -> TransactionsResult<Connections> {
        let span = current_span_for_instrument_at!("info");

        let pg_conn = self.pg_txn.commit_into_conn().await?;

        if let DelayedRebaseWithReply::WithUpdates {
            rebaser,
            workspace_pk,
            change_set_id,
            updates_address,
            event_session_id,
        } = maybe_rebase
        {
            span.record("si.change_set.id", change_set_id.to_string());
            span.record("si.workspace.id", workspace_pk.to_string());
            rebase_with_reply(
                rebaser,
                workspace_pk,
                change_set_id,
                updates_address,
                event_session_id,
            )
            .await?;
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

/// The madness needs to end soon.
///
/// We are *obsessed* with possibly submitting work to the Rebaser in this module. This type
/// attempts to stick *one* data type through the context world for the moment in the hopes this
/// will make future refactoring a little easier.
#[derive(Debug)]
enum DelayedRebaseWithReply<'a> {
    NoUpdates,
    WithUpdates {
        rebaser: &'a RebaserClient,
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        updates_address: RebaseBatchAddressKind,
        event_session_id: EventSessionId,
    },
}

#[instrument(
    level="info",
    skip_all,
    fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = %workspace_pk,
            si.rebaser.updates_address = %updates_address,
        ),
    )]
#[inline]
async fn rebase_with_reply(
    rebaser: &RebaserClient,
    workspace_pk: WorkspacePk,
    change_set_id: ChangeSetId,
    updates_address: RebaseBatchAddressKind,
    event_session_id: EventSessionId,
) -> TransactionsResult<()> {
    let timeout = Duration::from_secs(60);
    let metric_label = format!("{workspace_pk}:{change_set_id}");
    metric!(counter.dal.rebase_requested = 1, label = metric_label);
    let (request_id, reply_fut) = rebaser
        .enqueue_updates_with_reply(
            workspace_pk,
            change_set_id,
            updates_address,
            event_session_id,
        )
        .await?;

    let reply_fut = reply_fut.instrument(info_span!(
        "rebaser_client.await_response",
        si.workspace.id = %workspace_pk,
        si.change_set.id = %change_set_id,
    ));

    // Wait on response from Rebaser after request has processed
    let reply = time::timeout(timeout, reply_fut)
        .await
        .map_err(|_elapsed| {
            TransactionsError::RebaserReplyDeadlineElasped(timeout, request_id)
        })??;

    metric!(counter.dal.rebase_requested = -1, label = metric_label);

    match &reply.status {
        RebaseStatus::Success { .. } => Ok(()),
        // Return a specific error if the Rebaser reports that it failed to process the request
        RebaseStatus::Error { message } => Err(TransactionsError::RebaseFailed(
            updates_address,
            change_set_id,
            message.clone(),
        )),
    }
}
