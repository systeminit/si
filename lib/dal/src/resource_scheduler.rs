use std::time::Duration;

use si_data::{NatsError, PgError, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time;

use crate::{
    Component, ComponentError, HistoryActor, HistoryEventError, RequestContext, ServicesContext,
    StandardModelError, SystemId,
};

#[derive(Error, Debug)]
pub enum ResourceSchedulerError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg pool error: {0}")]
    PgPool(#[from] PgPoolError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("component")]
    Component(#[from] ComponentError),
}

pub type ResourceSchedulerResult<T> = Result<T, ResourceSchedulerError>;

/// The resource scheduler handles looking up all the components, and scheduling
/// them to refresh their resources. Eventually, it should become smart enough to
/// parallelize, it might be extracted to a fully separate service, etc etc. For now,
/// it is the dumbest thing that could possibly work - no more often than every 30
/// seconds, it will ask a component to sync_resource.
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
    pub async fn start(self) {
        tokio::spawn(async move { self.start_task().await });
    }

    /// The internal task spawned by `start`. No more frequently than every 30
    /// seconds, it will iterate over all the components in the database and
    /// schedule them to sync their resources.
    #[instrument(name = "resource_scheduler.start_task", skip_all, level = "debug")]
    async fn start_task(&self) {
        let mut interval = time::interval(Duration::from_secs(30));
        'schedule: loop {
            interval.tick().await;
            let to_check = match self.components_to_check().await {
                Ok(r) => r,
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to fetch components; aborting scheduled interval check"
                    );
                    continue 'schedule;
                }
            };

            'check: for (component, system_id) in to_check.into_iter() {
                if component.tenancy().workspaces().is_empty() {
                    error!(
                        "component does not have any workspaces in tenancy; skipping it: {:?}",
                        component
                    );
                    continue 'check;
                }
                let builder = self.services_context.clone().into_builder();
                let mut starter = match builder.transactions_starter().await {
                    Ok(starter) => starter,
                    Err(err) => {
                        error!("Unable to generate transaction starter: {:?}", err);
                        continue 'check;
                    }
                };
                let txns = match starter.start().await {
                    Ok(txns) => txns,
                    Err(err) => {
                        error!("Unable to start transactions: {:?}", err);
                        continue 'check;
                    }
                };
                let request_context = match RequestContext::new_workspace_head(
                    txns.pg(),
                    HistoryActor::SystemInit,
                    *component
                        .tenancy()
                        .workspaces()
                        .first()
                        .expect("empty workspace array when we checked earlier; bug!"),
                    None,
                )
                .await
                {
                    Ok(request_context) => request_context,
                    Err(err) => {
                        error!("Unable to create request context: {:?}", err);
                        continue 'check;
                    }
                };
                let ctx = builder.build(request_context, &txns);

                if let Err(error) = component.sync_resource(&ctx, system_id).await {
                    error!(?error, "Failed to sync component, moving to the next.");
                    continue 'check;
                }
                if let Err(err) = txns.commit().await {
                    error!("Unable to commit transactions: {:?}", err);
                    continue 'check;
                };
            }
        }
    }

    /// Gets a list of all the components in the database.
    #[instrument(skip_all, level = "debug")]
    async fn components_to_check(&self) -> ResourceSchedulerResult<Vec<(Component, SystemId)>> {
        let mut conn = self.services_context.pg_pool().get().await?;
        let txn = conn.transaction().await?;
        let results = Component::list_for_resource_sync(&txn).await?;
        txn.commit().await?;
        Ok(results)
    }
}
