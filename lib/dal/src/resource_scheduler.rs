use std::time::Duration;

use si_data_nats::NatsError;
use si_data_pg::{PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{sync::broadcast, time};

use crate::{
    standard_model, Component, Resource, ServicesContext, StandardModelError, SystemId,
    TransactionsError,
};

#[derive(Error, Debug)]
pub enum ResourceSchedulerError {
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    PgPool(#[from] PgPoolError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    StandardModelError(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type ResourceSchedulerResult<T> = Result<T, ResourceSchedulerError>;

/// The resource scheduler handles looking up all the components, and scheduling
/// their resources to refresh. Eventually, it should become smart enough to parallelize,
/// it might be extracted to a fully separate service, etc etc. For now,
/// it is the dumbest thing that could possibly work - no more often than every 30
/// seconds, it will ask a resource to refresh
#[derive(Debug, Clone)]
pub struct ResourceScheduler {
    services_context: ServicesContext,
}

impl ResourceScheduler {
    pub fn new(services_context: ServicesContext) -> ResourceScheduler {
        ResourceScheduler { services_context }
    }

    /// Starts the scheduler. It returns the join handle to the spawned scheduler, and
    /// consumes itself. The caller should check for errors and restart the scheduler if
    /// it ever returns an error.
    pub fn start(self, mut shutdown_broadcast_rx: broadcast::Receiver<()>) {
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_broadcast_rx.recv() => {
                    info!("Resource Refreshing Scheduler received shutdown request, bailing out");
                },
                _ = self.start_task() => {}
            }
            info!("Resource Refreshing stopped");
        });
    }

    /// The internal task spawned by `start`. No more frequently than every 30
    /// seconds, it will iterate over all the components on head in the database and
    /// schedule them to refresh.
    #[instrument(name = "resource_scheduler.start_task", skip_all, level = "debug")]
    async fn start_task(&self) {
        let mut interval = time::interval(Duration::from_secs(30));
        'schedule: loop {
            interval.tick().await;
            let components = match self.components().await {
                Ok(r) => r,
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to fetch components; aborting scheduled interval check"
                    );
                    continue 'schedule;
                }
            };

            'refresh: for component in components {
                let builder = self.services_context.clone().into_builder();
                // First we're building a ctx with universal head, then updating it with a
                // workspace head request context

                let mut ctx = match builder.build_default().await {
                    Ok(ctx) => ctx,
                    Err(err) => {
                        error!("Unable to build dal context: {:?}", err);
                        continue 'refresh;
                    }
                };

                let read_tenancy = match component.tenancy().clone_into_read_tenancy(&ctx).await {
                    Ok(tenancy) => tenancy,
                    Err(err) => {
                        error!(
                            "Unable to update dal context for workspace tenanices: {:?}",
                            err
                        );
                        continue 'refresh;
                    }
                };
                ctx.update_tenancies(read_tenancy, component.tenancy().clone());

                if let Err(error) = Resource::refresh(&ctx, &component, SystemId::NONE).await {
                    error!(?error, "Failed to refresh resource, moving to the next.");
                    continue 'refresh;
                }
                if let Err(err) = ctx.commit().await {
                    error!("Unable to commit transactions: {:?}", err);
                    continue 'refresh;
                };
            }
        }
    }

    /// Gets a list of all the resources in the database.
    #[instrument(skip_all, level = "debug")]
    async fn components(&self) -> ResourceSchedulerResult<Vec<Component>> {
        let builder = self.services_context.clone().into_builder();
        let ctx = builder.build_default().await?;

        // We need to bypass tenancy checks, only lists components on head as they are the only ones refreshed
        let rows = ctx
            .txns()
            .pg()
            .query(
                "SELECT DISTINCT ON (id) id, row_to_json(components.*) as object
                 FROM components
                 WHERE is_visible_v1($1, components.visibility_change_set_pk, components.visibility_deleted_at)
                 ORDER BY id",
                &[ctx.visibility()],
            )
            .await?;
        let components: Vec<Component> = standard_model::objects_from_rows(rows)?;

        ctx.commit().await?;
        Ok(components)
    }
}
