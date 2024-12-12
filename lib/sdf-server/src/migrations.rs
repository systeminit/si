use std::future::IntoFuture as _;

use anyhow::{Context, Result};
use audit_database::{AuditDatabaseContext, AuditDatabaseContextError};
use dal::{
    cached_module::CachedModule, slow_rt::SlowRuntimeError,
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
    #[error("error while migrating audit database")]
    MigrateAuditDatabase,
    #[error("error while migrating cached modules")]
    MigrateCachedModules,
    #[error("error while migrating dal database")]
    MigrateDalDatabase,
    #[error("error while migrating layer db database")]
    MigrateLayerDbDatabase,
    #[error("error while migrating snapshots")]
    MigrateSnapshots,
    #[error("module index url not set")]
    ModuleIndexNotSet,
    #[error("slow runtime: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
}

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
    ) -> Result<Self> {
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
    pub async fn run_migrations(self, update_module_cache: bool) -> Result<()> {
        let span = current_span_for_instrument_at!("info");

        self.migrate_audit_database()
            .await
            .map_err(|err| span.record_err(err))?;

        self.migrate_layer_db_database()
            .await
            .map_err(|err| span.record_err(err))?;

        self.migrate_dal_database()
            .await
            .map_err(|err| span.record_err(err))?;

        self.migrate_snapshots()
            .await
            .map_err(|err| span.record_err(err))?;

        if update_module_cache {
            self.migrate_module_cache()
                .await
                .map_err(|err| span.record_err(err))?;
        }

        span.record_ok();
        Ok(())
    }

    #[instrument(name = "sdf.migrator.migrate_audit_database", level = "info", skip_all)]
    async fn migrate_audit_database(&self) -> Result<()> {
        audit_database::migrate(&self.audit_database_context)
            .await
            .context(MigratorError::MigrateAuditDatabase)
    }

    #[instrument(
        name = "sdf.migrator.migrate_layer_db_database",
        level = "info",
        skip_all
    )]
    async fn migrate_layer_db_database(&self) -> Result<()> {
        self.services_context
            .layer_db()
            .pg_migrate()
            .await
            .context(MigratorError::MigrateLayerDbDatabase)
    }

    #[instrument(name = "sdf.migrator.migrate_dal_database", level = "info", skip_all)]
    async fn migrate_dal_database(&self) -> Result<()> {
        dal::migrate_all_with_progress(&self.services_context)
            .await
            .context(MigratorError::MigrateDalDatabase)
    }

    #[instrument(name = "sdf.migrator.migrate_snapshots", level = "info", skip_all)]
    async fn migrate_snapshots(&self) -> Result<()> {
        let dal_context = self.services_context.clone().into_builder(true);
        let ctx = dal_context
            .build_default(None)
            .await
            .context(MigratorError::MigrateSnapshots)?;

        let mut migrator = SnapshotGraphMigrator::new();
        migrator
            .migrate_all(&ctx)
            .await
            .context(MigratorError::MigrateSnapshots)?;
        ctx.commit_no_rebase()
            .await
            .context(MigratorError::MigrateSnapshots)?;
        Ok(())
    }

    #[instrument(name = "sdf.migrator.migrate_module_cache", level = "info", skip_all)]
    async fn migrate_module_cache(&self) -> Result<()> {
        let dal_context = self.services_context.clone().into_builder(true);
        let ctx = dal_context
            .build_default(None)
            .await
            .context(MigratorError::MigrateCachedModules)?;

        info!("Updating local module cache");

        let ctx = ctx.clone();
        tokio::spawn(async move {
            let new_modules = CachedModule::update_cached_modules(&ctx)
                .await
                .context(MigratorError::MigrateCachedModules)?;
            info!(
                "{} new builtin assets found in module index",
                new_modules.len()
            );
            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}
