use std::time::Duration;

use si_data::{NatsClient, NatsError, PgError, PgPool, PgPoolError};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time;
use veritech::EncryptionKey;

use crate::system::UNSET_ID_VALUE;
use crate::{Component, ComponentError, HistoryActor, HistoryEventError, StandardModelError};

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
    pg_pool: PgPool,
    nats: NatsClient,
    veritech: veritech::Client,
    encryption_key: EncryptionKey,
}

impl ResourceScheduler {
    pub fn new(
        pg_pool: PgPool,
        nats: NatsClient,
        veritech: veritech::Client,
        encryption_key: EncryptionKey,
    ) -> ResourceScheduler {
        ResourceScheduler {
            pg_pool,
            nats,
            veritech,
            encryption_key,
        }
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
            let components = match self.components_to_check().await {
                Ok(r) => r,
                Err(error) => {
                    error!(
                        ?error,
                        "Failed to fetch components; aborting scheduled interval check"
                    );
                    continue 'schedule;
                }
            };

            'check: for component in components.into_iter() {
                let mut conn = match self.pg_pool.get().await {
                    Ok(conn) => conn,
                    Err(err) => {
                        error!("Unable to get Pg Pool Connection: {:?}", err);
                        continue 'check;
                    }
                };
                let txn = match conn.transaction().await {
                    Ok(txn) => txn,
                    Err(err) => {
                        error!("Unable to start Pg Transaction: {:?}", err);
                        continue 'check;
                    }
                };
                let nats = self.nats.transaction();
                if let Err(error) = component
                    .sync_resource(
                        &txn,
                        &nats,
                        self.veritech.clone(),
                        &self.encryption_key,
                        &HistoryActor::SystemInit,
                        UNSET_ID_VALUE.into(),
                    )
                    .await
                {
                    error!(?error, "Failed to sync component, moving to the next.");
                    continue 'check;
                }
                if let Err(err) = txn.commit().await {
                    error!("Unable to commit Pg Transaction: {:?}", err);
                    continue 'check;
                };
                if let Err(err) = nats.commit().await {
                    error!("Unable to commit Nats Transaction: {:?}", err);
                    continue 'check;
                };
            }
        }
    }

    /// Gets a list of all the components in the database.
    #[instrument(skip_all, level = "debug")]
    async fn components_to_check(&self) -> ResourceSchedulerResult<Vec<Component>> {
        let mut conn = self.pg_pool.get().await?;
        let txn = conn.transaction().await?;
        let components = Component::list_for_resource_sync(&txn).await?;
        txn.commit().await?;
        Ok(components)
    }
}
