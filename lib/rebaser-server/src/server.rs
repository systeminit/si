use std::{
    collections::{HashMap, HashSet},
    future::IntoFuture,
    sync::Arc,
};

use dal::feature_flags::FeatureFlagService;
use dal::{
    ChangeSetStatus, DalContext, DalContextBuilder, DalLayerDb, JobQueueProcessor, NatsProcessor,
    ServicesContext,
};
use futures::StreamExt;
use si_crypto::{
    CryptoConfig, CycloneEncryptionKey, SymmetricCryptoService, SymmetricCryptoServiceConfig,
};
use si_data_nats::{async_nats::jetstream, NatsClient, NatsConfig};
use si_data_pg::{InstrumentedClient, PgPool, PgPoolConfig};
use si_events::{ChangeSetId, WorkspacePk};
use si_layer_cache::activities::{Activity, ActivityPayload};
use telemetry::prelude::*;
use tokio::task::JoinHandle;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use veritech_client::Client as VeritechClient;

use crate::{
    change_set_requests::ChangeSetRequestsTask, Config, ServerError as Error, ServerResult,
};

const CONSUMER_NAME: &str = "rebaser-requests";

/// Server metadata, used with telemetry.
#[derive(Clone, Debug)]
pub struct ServerMetadata {
    #[allow(dead_code)] // TODO(fnichol): this will be used in telemetry, so drop the dead_code
    // exception then
    instance_id: String,
}

/// A service which concurrently processes rebaser requests across multiple change sets.
///
/// Each change set is processed with a dedicated micro service (a [`ChangeSetRequestsTask`]) which
/// consumes a work queue of rebaser requests. The work queue semantics allows multiple Rebaser
/// server instances to run and share work while collectively processing one message at a time,
/// thus preserving the total order of requests within the scope of a chanage set.
///
/// An activity stream of all rebaser requests is processed in the main loop to determine if new
/// change set tasks should be spawned.
#[derive(Debug)]
pub struct Server {
    metadata: Arc<ServerMetadata>,
    ctx_builder: DalContextBuilder,
    change_set_tasks: HashMap<ChangeSetId, RunningTask>,
    shutdown_token: CancellationToken,
}

impl Server {
    /// Creates a runnable [`Server`] from configuration.
    #[instrument(name = "rebaser.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        shutdown_token: CancellationToken,
        tracker: TaskTracker,
    ) -> ServerResult<Self> {
        dal::init()?;

        let encryption_key = Self::load_encryption_key(config.crypto().clone()).await?;
        let nats = Self::connect_to_nats(config.nats()).await?;
        let pg_pool = Self::create_pg_pool(config.pg_pool()).await?;
        let veritech = Self::create_veritech_client(nats.clone());
        let job_processor = Self::create_job_processor(nats.clone());
        let symmetric_crypto_service =
            Self::create_symmetric_crypto_service(config.symmetric_crypto_service()).await?;

        let (layer_db, layer_db_graceful_shutdown) =
            DalLayerDb::from_config(config.layer_db_config().clone(), shutdown_token.clone())
                .await?;
        tracker.spawn(layer_db_graceful_shutdown.into_future());

        let services_context = ServicesContext::new(
            pg_pool,
            nats.clone(),
            job_processor,
            veritech.clone(),
            encryption_key,
            None,
            None,
            symmetric_crypto_service,
            layer_db,
            FeatureFlagService::default(),
        );

        Self::from_services(
            config.instance_id().to_string(),
            services_context,
            shutdown_token,
        )
    }

    /// Creates a runnable [`Server`] from pre-configured and pre-created services.
    #[instrument(name = "rebaser.init.from_services", level = "info", skip_all)]
    pub fn from_services(
        instance_id: impl Into<String>,
        services_context: ServicesContext,
        shutdown_token: CancellationToken,
    ) -> ServerResult<Self> {
        dal::init()?;

        let metadata = ServerMetadata {
            instance_id: instance_id.into(),
        };

        let ctx_builder = DalContext::builder(services_context, false);

        Ok(Self {
            metadata: Arc::new(metadata),
            ctx_builder,
            change_set_tasks: HashMap::default(),
            shutdown_token,
        })
    }

    /// Runs the service to completion or until the first internal error is encountered.
    #[inline]
    pub async fn run(self) {
        if let Err(err) = self.try_run().await {
            error!(error = ?err, "error while running main loop");
        }
    }

    /// Runs the service to completion, returning its result (i.e. whether it successful or an
    /// internal error was encountered).
    pub async fn try_run(mut self) -> ServerResult<()> {
        // TODO(fnichol): while it would be great to query the database on launch to get the active
        // change sets, in an initial cluster deployment, the database may not yet be created or
        // migrated. This is an outstanding issue and a micro-service "smell" as the Rebaser is not
        // the logical owner of the database that it wants to query. As we are evolving the LayerDb
        // infrastructure, it's more likely that a LayerDb instance with active change sets will
        // be the path forward, but for now I'm leaving this skeleton implementation (which is also
        // commented out) as a marker of what we *should* be doing, once we've properly figured
        // that out ;)
        //
        // self.launch_initial_change_set_tasks().await?;

        // Set up an activity stream with change set-related messages
        let mut activities = self
            .ctx_builder
            .layer_db()
            .activity()
            .rebase()
            .rebaser_activity_stream()
            .await?;

        // Consume and process messages from the activity stream until a shutdown is signaled...
        loop {
            tokio::select! {
                // Graceful shutdown has been signaled, so cleanly end the processing loop
                _ = self.shutdown_token.cancelled() => {
                    debug!("main loop received cancellation");
                    break;
                }
                // New activity message from the stream
                maybe_result = activities.next() => {
                    match maybe_result {
                        // Successfully received a new activity message
                        Some(Ok(activity)) => {
                            if let Err(err) = self.process_activity(activity).await {
                                warn!(error = ?err, "failed to process an activity message");
                            }
                        }
                        // Error on next activity message. This is from a Broadcast channel, so an
                        // error indicates we are a slow consumer and are lagging behind the
                        // capacity of the channel. This also means that we will miss/skip
                        // messages.
                        Some(Err(lagged_err)) => {
                            warn!(
                                error = ?lagged_err,
                                "lagged error, messages have been skipped",
                            );
                        }
                        // Stream has closed, so cleanly end the processing loop
                        None => {
                            trace!("activities stream has closed");
                            break;
                        }
                    }
                }
            }
        }

        self.terminate_all_change_set_tasks().await?;

        info!("main loop shutdown complete");
        Ok(())
    }

    #[inline]
    async fn process_activity(&mut self, activity: Activity) -> ServerResult<()> {
        match activity.payload {
            // A rebase request implies a work queue should be set up for the associated change
            // set, so we'll launch a task to process from this queue.
            ActivityPayload::RebaseRequest(req) => {
                let workspace_id = activity.metadata.tenancy.workspace_pk;
                let change_set_id = req.to_rebase_change_set_id.into();
                trace!(%workspace_id, %change_set_id, "processing rebase request activity");

                if !self.running_change_set_task(change_set_id) {
                    self.launch_change_set_task(workspace_id, change_set_id)
                        .await?;
                }
            }
            // Exhaustively match variants so we catch future new variants
            ActivityPayload::RebaseFinished(_)
            | ActivityPayload::IntegrationTest(_)
            | ActivityPayload::IntegrationTestAlt(_) => {
                trace!(payload = ?activity.payload, "ignoring activity message type");
            }
        }

        Ok(())
    }

    /// Gets a [`ShutdownHandle`] that can externally or on demand trigger the server's shutdown
    /// process.
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            token: self.shutdown_token.clone(),
        }
    }

    fn running_change_set_task(&self, change_set_id: ChangeSetId) -> bool {
        self.change_set_tasks.contains_key(&change_set_id)
    }

    async fn launch_initial_change_set_tasks(&mut self) -> ServerResult<()> {
        let ctx = self.ctx_builder.build_default().await?;
        let pg = ctx.pg_pool().get().await.map_err(Error::dal_pg_pool)?;
        let ids = Self::all_open_change_sets(&pg).await?;

        for (workspace_id, change_set_id) in ids {
            self.launch_change_set_task(workspace_id, change_set_id)
                .await?;
        }

        Ok(())
    }

    #[instrument(
        name = "rebaser.launch_change_set_task",
        level = "debug",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.pk = %workspace_id,
        )
    )]
    async fn launch_change_set_task(
        &mut self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) -> ServerResult<()> {
        if self.running_change_set_task(change_set_id) {
            return Err(Error::ExistingChangeSetTask(change_set_id));
        }

        let incoming = self
            .ctx_builder
            .layer_db()
            .activity()
            .rebaser_change_set_requests_work_queue_stream(workspace_id, change_set_id)
            .await?
            .create_consumer(Self::consumer_config())
            .await?
            .messages()
            .await?;

        let token = CancellationToken::new();
        let task = ChangeSetRequestsTask::create(
            self.metadata.clone(),
            workspace_id,
            change_set_id,
            incoming,
            self.ctx_builder.clone(),
            token.clone(),
        );
        let handle = tokio::spawn(task.run());
        let running_task = RunningTask { handle, token };

        self.change_set_tasks.insert(change_set_id, running_task);

        Ok(())
    }

    async fn terminate_all_change_set_tasks(&mut self) -> ServerResult<()> {
        let change_set_ids: Vec<_> = self.change_set_tasks.keys().copied().collect();

        // Signal all running change set tasks to shut down and await their graceful shutdowns. As
        // we have the Tokio task's `Handle` which `impl Future` we'll use a `TaskTracker` to track
        // the resolving of all `Handle`s. Nice!
        let tracker = TaskTracker::new();
        for change_set_id in change_set_ids {
            tracker.spawn(self.terminate_change_set_task(change_set_id)?);
        }
        tracker.close();
        tracker.wait().await;

        Ok(())
    }

    #[instrument(
        name = "rebaser.terminate_change_set_task",
        level = "debug",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
        )
    )]
    fn terminate_change_set_task(
        &mut self,
        change_set_id: ChangeSetId,
    ) -> ServerResult<JoinHandle<()>> {
        let task = self
            .change_set_tasks
            .remove(&change_set_id)
            // Error if a task is not being tracked
            .ok_or(Error::MissingChangeSetTask(change_set_id))?;
        task.token.cancel();

        // Return an optional await-able future that corresponds that resolves when the task has
        // completely shutdown
        Ok(task.handle.into_future())
    }

    #[instrument(name = "rebaser.all_open_change_sets", level = "debug", skip_all)]
    async fn all_open_change_sets(
        pg: &InstrumentedClient,
    ) -> ServerResult<HashSet<(WorkspacePk, ChangeSetId)>> {
        const SQL_OPEN_CHANGE_SETS: &str =
            "SELECT id, workspace_id from change_set_pointers WHERE status IN ($1, $2, $3)";

        let rows = pg
            .query(
                SQL_OPEN_CHANGE_SETS,
                &[
                    &ChangeSetStatus::Open.as_ref(),
                    &ChangeSetStatus::NeedsApproval.as_ref(),
                    &ChangeSetStatus::NeedsAbandonApproval.as_ref(),
                ],
            )
            .await
            .map_err(Error::DalOpenChangeSets)?;

        let mut ids = HashSet::with_capacity(rows.len());
        for row in rows {
            let change_set_id: dal::ChangeSetId =
                row.try_get("id").map_err(Error::DalOpenChangeSets)?;
            let workspace_id: dal::WorkspacePk = row
                .try_get("workspace_id")
                .map_err(Error::DalOpenChangeSets)?;
            ids.insert((workspace_id.into(), change_set_id.into()));
        }

        Ok(ids)
    }

    #[inline]
    fn consumer_config() -> jetstream::consumer::pull::Config {
        jetstream::consumer::pull::Config {
            durable_name: Some(CONSUMER_NAME.to_owned()),
            // Ensure that only *one* message is processed before the next message is processed.
            // Note that the consumer is shared across potentially multiple connected clients,
            // meaning they all share this behavior (i.e. only one service processes one message
            // at a time, thus guarenteeing queue is processed serially, in order).
            max_ack_pending: 1,
            ..Default::default()
        }
    }

    #[instrument(name = "rebaser.init.load_encryption_key", level = "info", skip_all)]
    async fn load_encryption_key(
        crypto_config: CryptoConfig,
    ) -> ServerResult<Arc<CycloneEncryptionKey>> {
        Ok(Arc::new(
            CycloneEncryptionKey::from_config(crypto_config)
                .await
                .map_err(Error::CycloneEncryptionKey)?,
        ))
    }

    #[instrument(name = "rebaser.init.connect_to_nats", level = "info", skip_all)]
    async fn connect_to_nats(nats_config: &NatsConfig) -> ServerResult<NatsClient> {
        let client = NatsClient::new(nats_config).await?;
        debug!("successfully connected nats client");
        Ok(client)
    }

    #[instrument(name = "rebaser.init.create_pg_pool", level = "info", skip_all)]
    async fn create_pg_pool(pg_pool_config: &PgPoolConfig) -> ServerResult<PgPool> {
        let pool = PgPool::new(pg_pool_config)
            .await
            .map_err(Error::dal_pg_pool)?;
        debug!("successfully started pg pool (note that not all connections may be healthy)");
        Ok(pool)
    }

    #[instrument(name = "rebaser.init.create_veritech_client", level = "info", skip_all)]
    fn create_veritech_client(nats: NatsClient) -> VeritechClient {
        VeritechClient::new(nats)
    }

    #[instrument(name = "rebaser.init.create_job_processor", level = "info", skip_all)]
    fn create_job_processor(nats: NatsClient) -> Box<dyn JobQueueProcessor + Send + Sync> {
        Box::new(NatsProcessor::new(nats)) as Box<dyn JobQueueProcessor + Send + Sync>
    }

    #[instrument(
        name = "rebaser.init.create_symmetric_crypto_service",
        level = "info",
        skip_all
    )]
    async fn create_symmetric_crypto_service(
        config: &SymmetricCryptoServiceConfig,
    ) -> ServerResult<SymmetricCryptoService> {
        SymmetricCryptoService::from_config(config)
            .await
            .map_err(Into::into)
    }
}

#[derive(Debug)]
struct RunningTask {
    handle: JoinHandle<()>,
    token: CancellationToken,
}

#[derive(Clone, Debug)]
pub struct ShutdownHandle {
    token: CancellationToken,
}

impl ShutdownHandle {
    pub fn shutdown(self) {
        self.token.cancel()
    }
}
