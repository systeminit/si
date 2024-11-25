use std::future::IntoFuture as _;

use audit_logs::database::{
    AuditDatabaseContext, AuditDatabaseContextError, AuditDatabaseMigrationError,
};
use dal::{
    cached_module::CachedModuleError, slow_rt::SlowRuntimeError,
    workspace_snapshot::migrator::SnapshotGraphMigrator, ServicesContext,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinError;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{init, Config};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MigratorError {
    #[error("audit database context error: {0}")]
    AuditDatabaseContext(#[from] AuditDatabaseContextError),
    #[error("error while initializing: {0}")]
    Init(#[from] init::InitError),
    #[error("tokio join error: {0}")]
    Join(#[from] JoinError),
    #[error("error while migrating audit database: {0}")]
    MigrateAuditDatabase(#[source] AuditDatabaseMigrationError),
    #[error("error while migrating dal database: {0}")]
    MigrateDalDatabase(#[source] dal::ModelError),
    #[error("error while migrating layer db database: {0}")]
    MigrateLayerDbDatabase(#[source] si_layer_cache::LayerDbError),
    #[error("error while migrating snapshots: {0}")]
    MigrateSnapshots(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
    #[error("module cache error: {0}")]
    ModuleCache(#[from] CachedModuleError),
    #[error("module index url not set")]
    ModuleIndexNotSet,
    #[error("slow runtime: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
}

impl MigratorError {
    fn migrate_snapshots<E>(err: E) -> Self
    where
        E: std::error::Error + 'static + Sync + Send,
    {
        Self::MigrateSnapshots(Box::new(err))
    }
}

type MigratorResult<T> = std::result::Result<T, MigratorError>;

#[derive(Clone)]
pub struct Migrator {
    services_context: ServicesContext,
    audit_database_context: AuditDatabaseContext,
}

impl Migrator {
    #[instrument(name = "sdf.migrator.init.from_config", level = "info", skip_all)]
    pub async fn from_config(
        config: Config,
        helping_tasks_tracker: &TaskTracker,
        helping_tasks_token: CancellationToken,
    ) -> MigratorResult<Self> {
        let (services_context, layer_db_graceful_shutdown) =
            init::services_context_from_config(&config, helping_tasks_token).await?;

        // Spawn helping tasks and track them for graceful shutdown
        helping_tasks_tracker.spawn(layer_db_graceful_shutdown.into_future());

        let audit_database_context = AuditDatabaseContext::from_config(config.audit()).await?;

        Ok(Self::from_services(
            services_context,
            audit_database_context,
        ))
    }

    #[instrument(name = "sdf.migrator.init.from_services", level = "info", skip_all)]
    pub fn from_services(
        services_context: ServicesContext,
        audit_database_context: AuditDatabaseContext,
    ) -> Self {
        Self {
            services_context,
            audit_database_context,
        }
    }

    #[instrument(
        name = "sdf.migrator.run_migrations",
        level = "info",
        skip_all,
        fields(
            otel.status_code = Empty,
            otel.status_message = Empty,
        )
    )]
    pub async fn run_migrations(self) -> MigratorResult<()> {
        let span = current_span_for_instrument_at!("info");

        // TODO(nick,john): once we have seeded the database successfully, we can replace this with
        // error propagation.
        if let Err(err) = self.migrate_audit_database().await {
            warn!(
                ?err,
                "skipping audit database migration due to error, which is currently expected"
            );
        }

        self.migrate_layer_db_database()
            .await
            .map_err(|err| span.record_err(err))?;

        self.migrate_dal_database()
            .await
            .map_err(|err| span.record_err(err))?;

        self.migrate_snapshots()
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(())
    }

    #[instrument(name = "sdf.migrator.migrate_audit_database", level = "info", skip_all)]
    async fn migrate_audit_database(&self) -> MigratorResult<()> {
        audit_logs::database::migrate(&self.audit_database_context)
            .await
            .map_err(MigratorError::MigrateAuditDatabase)
    }

    #[instrument(
        name = "sdf.migrator.migrate_layer_db_database",
        level = "info",
        skip_all
    )]
    async fn migrate_layer_db_database(&self) -> MigratorResult<()> {
        self.services_context
            .layer_db()
            .pg_migrate()
            .await
            .map_err(MigratorError::MigrateLayerDbDatabase)
    }

    #[instrument(name = "sdf.migrator.migrate_dal_database", level = "info", skip_all)]
    async fn migrate_dal_database(&self) -> MigratorResult<()> {
        dal::migrate_all_with_progress(&self.services_context)
            .await
            .map_err(MigratorError::MigrateDalDatabase)
    }

    #[instrument(name = "sdf.migrator.migrate_snapshots", level = "info", skip_all)]
    async fn migrate_snapshots(&self) -> MigratorResult<()> {
        let dal_context = self.services_context.clone().into_builder(true);
        let ctx = dal_context
            .build_default()
            .await
            .map_err(MigratorError::migrate_snapshots)?;

        let mut migrator = SnapshotGraphMigrator::new();
        migrator
            .migrate_all(&ctx)
            .await
            .map_err(MigratorError::migrate_snapshots)?;
        ctx.commit_no_rebase()
            .await
            .map_err(MigratorError::migrate_snapshots)?;
        Ok(())
    }
}
