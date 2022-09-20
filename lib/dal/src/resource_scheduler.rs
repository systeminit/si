use std::time::Duration;

use si_data::{NatsError, PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{sync::broadcast, time};

use crate::{Resource, ServicesContext, StandardModel, StandardModelError, TransactionsError};

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

/// The resource scheduler handles looking up all the resources, and scheduling
/// them to refresh. Eventually, it should become smart enough to parallelize,
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
    /// seconds, it will iterate over all the resources in the database and
    /// schedule them to refresh.
    #[instrument(name = "resource_scheduler.start_task", skip_all, level = "debug")]
    async fn start_task(&self) {
        let mut interval = time::interval(Duration::from_secs(30));
        'schedule: loop {
            interval.tick().await;
            let resources = match self.resources().await {
                Ok(r) => r,
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to fetch resources; aborting scheduled interval check"
                    );
                    continue 'schedule;
                }
            };

            'refresh: for mut resource in resources {
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

                let read_tenancy = match resource.tenancy().clone_into_read_tenancy(&ctx).await {
                    Ok(tenancy) => tenancy,
                    Err(err) => {
                        error!(
                            "Unable to update dal context for workspace tenanices: {:?}",
                            err
                        );
                        continue 'refresh;
                    }
                };
                ctx.update_tenancies(read_tenancy, resource.tenancy().clone());

                if let Err(error) = resource.refresh(&ctx).await {
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
    async fn resources(&self) -> ResourceSchedulerResult<Vec<Resource>> {
        let builder = self.services_context.clone().into_builder();
        let ctx = builder
            .build_default()
            .await
            .expect("cannot start transactions");
        let results = Resource::list(&ctx).await?;
        ctx.commit().await?;
        Ok(results)
    }
}
