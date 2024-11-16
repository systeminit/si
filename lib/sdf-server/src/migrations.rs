use std::{future::IntoFuture as _, time::Duration};

use audit_logs::pg::{
    AuditDatabaseContext, AuditDatabaseContextError, AuditDatabaseMigrationError,
};
use dal::{
    builtins, cached_module::CachedModuleError, pkg::PkgError, slow_rt::SlowRuntimeError,
    workspace_snapshot::migrator::SnapshotGraphMigrator, DalContext, ServicesContext, Workspace,
};
use module_index_client::{BuiltinsDetailsResponse, ModuleDetailsResponse, ModuleIndexClient};
use si_pkg::SiPkg;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    task::{JoinError, JoinSet},
    time::{self, Instant},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use ulid::Ulid;

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
    #[error("error while migrating builtins from module index: {0}")]
    MigrateBuiltins(#[source] Box<dyn std::error::Error + 'static + Sync + Send>),
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

    fn migrate_builtins<E>(err: E) -> Self
    where
        E: std::error::Error + 'static + Sync + Send,
    {
        Self::MigrateBuiltins(Box::new(err))
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

        self.migrate_builtins_from_module_index()
            .await
            .map_err(|err| span.record_err(err))?;

        span.record_ok();
        Ok(())
    }

    #[instrument(name = "sdf.migrator.migrate_audit_database", level = "info", skip_all)]
    async fn migrate_audit_database(&self) -> MigratorResult<()> {
        audit_logs::pg::migrate(&self.audit_database_context)
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

    #[instrument(
        name = "sdf.migrator.migrate_builtins_from_module_index",
        level = "info",
        skip_all
    )]
    async fn migrate_builtins_from_module_index(&self) -> MigratorResult<()> {
        let mut interval = time::interval(Duration::from_secs(5));
        let instant = Instant::now();

        let mut dal_context = self.services_context.clone().into_builder(true);
        dal_context.set_no_dependent_values();
        let mut ctx = dal_context
            .build_default()
            .await
            .map_err(MigratorError::migrate_builtins)?;
        info!("setup builtin workspace");
        Workspace::setup_builtin(&mut ctx)
            .await
            .map_err(MigratorError::migrate_builtins)?;

        info!("migrating intrinsic functions");
        builtins::func::migrate_intrinsics(&ctx)
            .await
            .map_err(MigratorError::migrate_builtins)?;

        let module_index_url = self
            .services_context
            .module_index_url()
            .ok_or(MigratorError::ModuleIndexNotSet)?;

        let module_index_client = ModuleIndexClient::unauthenticated_client(
            module_index_url
                .try_into()
                .map_err(MigratorError::migrate_builtins)?,
        );
        let module_list = module_index_client
            .list_builtins()
            .await
            .map_err(MigratorError::migrate_builtins)?;
        info!("builtins install starting");
        let install_builtins = install_builtins(ctx, module_list, module_index_client);
        tokio::pin!(install_builtins);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    info!(elapsed = instant.elapsed().as_secs_f32(), "migrating in progress...");
                }
                result = &mut install_builtins  => match result {
                    Ok(_) => {
                        info!(elapsed = instant.elapsed().as_secs_f32(), "migrating completed");
                        break;
                    }
                    Err(err) => return Err(err),
                }
            }
        }

        Ok(())
    }
}

async fn install_builtins(
    ctx: DalContext,
    module_list: BuiltinsDetailsResponse,
    module_index_client: ModuleIndexClient,
) -> MigratorResult<()> {
    let dal = &ctx;
    let client = &module_index_client.clone();
    let modules: Vec<ModuleDetailsResponse> = module_list.modules;

    let total = modules.len();

    let mut join_set = JoinSet::new();
    for module in modules {
        let module = module.clone();
        let client = client.clone();
        join_set.spawn(async move {
            (
                module.name.to_owned(),
                (module.to_owned(), fetch_builtin(&module, &client).await),
            )
        });
    }

    let mut count: usize = 0;
    while let Some(res) = join_set.join_next().await {
        let (pkg_name, (module, res)) = res.map_err(MigratorError::migrate_builtins)?;
        match res {
            Ok(pkg) => {
                let instant = Instant::now();

                match dal::pkg::import_pkg_from_pkg(
                    &ctx,
                    &pkg,
                    Some(dal::pkg::ImportOptions {
                        is_builtin: true,
                        schema_id: module.schema_id().map(Into::into),
                        past_module_hashes: module.past_hashes,
                        ..Default::default()
                    }),
                )
                .await
                {
                    Ok(_) => {
                        count += 1;
                        let elapsed = instant.elapsed().as_secs_f32();
                        debug!(
                            "pkg {pkg_name} install finished successfully and took {elapsed:.2} seconds ({count} of {total} installed)",
                            );
                    }
                    Err(PkgError::PackageAlreadyInstalled(hash)) => {
                        count += 1;
                        debug!(%hash, "skipping pkg {pkg_name}: already installed ({count} of {total} installed)");
                    }
                    Err(err) => error!(?err, "pkg {pkg_name} install failed"),
                }
            }
            Err(err) => {
                error!(?err, "pkg {pkg_name} install failed with server error");
            }
        }
    }
    dal.commit()
        .await
        .map_err(MigratorError::migrate_builtins)?;

    let mut ctx = ctx.clone();
    ctx.update_snapshot_to_visibility()
        .await
        .map_err(MigratorError::migrate_builtins)?;

    Ok(())
}

async fn fetch_builtin(
    module: &ModuleDetailsResponse,
    module_index_client: &ModuleIndexClient,
) -> MigratorResult<SiPkg> {
    let module = module_index_client
        .get_builtin(Ulid::from_string(&module.id).unwrap_or_default())
        .await
        .map_err(MigratorError::migrate_builtins)?;

    SiPkg::load_from_bytes(&module).map_err(MigratorError::migrate_builtins)
}
