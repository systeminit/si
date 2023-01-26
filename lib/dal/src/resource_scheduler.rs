use std::{panic::AssertUnwindSafe, time::Duration};

use futures::future::FutureExt;
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{sync::broadcast, time};

use crate::{standard_model, Component, ServicesContext, StandardModelError, TransactionsError};

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

    #[instrument(name = "resource_scheduler.run", skip_all, level = "debug")]
    async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Refresh resources");
        let components = self.components().await?;

        for component in components {
            // First we're building a ctx with no tenancy at head, then updating it with a
            // workspace head request context

            let builder = self.services_context.clone().into_builder();
            let mut ctx = builder.build_default().await?;

            let read_tenancy = component.tenancy().clone_into_read_tenancy().await?;
            ctx.update_tenancies(read_tenancy, component.tenancy().clone());

            component.act(&ctx, "refresh").await?;
            ctx.commit().await?;
        }
        Ok(())
    }

    /// The internal task spawned by `start`. No more frequently than every 30
    /// seconds, it will iterate over all the components on head in the database and
    /// schedule them to refresh.
    #[instrument(name = "resource_scheduler.start_task", skip_all, level = "debug")]
    async fn start_task(&self) {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            match AssertUnwindSafe(self.run()).catch_unwind().await {
                Ok(Ok(())) => {}
                Ok(Err(err)) => error!("{err}"),
                Err(any) => {
                    // Note: Technically panics can be of any form, but most should be &str or String
                    match any.downcast::<String>() {
                        Ok(msg) => error!("panic: {msg}"),
                        Err(any) => match any.downcast::<&str>() {
                            Ok(msg) => error!("panic: {msg}"),
                            Err(any) => {
                                let id = any.type_id();
                                error!("panic message downcast failed of {id:?}",);
                            }
                        },
                    }
                }
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
