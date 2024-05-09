//! A per-changeset task to debounce dependent values updates

use std::{collections::HashSet, sync::Arc};

use dal::{DalContextBuilder, Tenancy, TransactionsError, Visibility};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    select,
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    time::{interval, Duration},
};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

const DVU_INTERVAL: Duration = Duration::from_secs(5);

/// DvuDebouncer error type
#[derive(Error, Debug)]
pub enum DvuDebouncerError {
    /// A transactions error
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

/// DvuDebouncer result type
type DvuDebouncerResult<T> = Result<T, DvuDebouncerError>;

/// Messages to the value management task
#[derive(Debug)]
enum ValueTaskMessage {
    /// Enqueue values into the debouncer
    EnqueueValues(Vec<Ulid>),
}

/// Messages to DVU spawner task
#[derive(Debug)]
enum DvuTaskMessage {
    Dvu(Vec<Ulid>),
}

/// The DVU debouncer
#[derive(Clone, Debug)]
pub struct DvuDebouncer {
    values: Arc<Mutex<HashSet<Ulid>>>,
    value_tx: UnboundedSender<ValueTaskMessage>,
    dvu_tx: UnboundedSender<DvuTaskMessage>,
    cancellation_token: CancellationToken,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    ctx_builder: DalContextBuilder,
}

impl DvuDebouncer {
    /// Create a new DvuDebouncer. Spawns two long running tasks. One receives
    /// and dedupes values, and enqueues jobs using the values on an interval.
    /// The other processes the DVU job queue serially, as it receives messages.
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        cancellation_token: CancellationToken,
        ctx_builder: DalContextBuilder,
    ) -> Self {
        let values = Arc::new(Mutex::new(Default::default()));

        let (value_tx, value_rx) = unbounded_channel();
        let (dvu_tx, dvu_rx) = unbounded_channel();

        let debouncer = Self {
            values,
            value_tx,
            dvu_tx,
            cancellation_token,
            workspace_id,
            change_set_id,
            ctx_builder,
        };

        let debouncer_clone = debouncer.clone();
        let debouncer_clone_2 = debouncer.clone();

        tokio::task::spawn(async { dvu_task(debouncer_clone, dvu_rx).await });
        tokio::task::spawn(async { debouncer_task(debouncer_clone_2, value_rx).await });

        debouncer
    }

    /// Send a message to this debouncer's task
    pub fn enqueue_values(&self, values: Vec<Ulid>) {
        // Send only fails if the receiver end has gone away
        if let Err(err) = self.value_tx.send(ValueTaskMessage::EnqueueValues(values)) {
            error!(error = ?err, "Failed to enqueue values for dependent values update, receiver is closed");
        }
    }

    async fn handle_enqueue_values(&self, new_values: &[Ulid]) {
        let mut values = self.values.lock().await;
        for value in new_values {
            values.insert(*value);
        }
    }

    async fn drain_values(&self) -> Option<Vec<Ulid>> {
        let mut values = self.values.lock().await;
        if values.is_empty() {
            None
        } else {
            let current_values = values.clone();
            *values = HashSet::new();
            Some(current_values.into_iter().collect())
        }
    }

    async fn enqueue_dependent_values_update(&self) {
        if let Some(values) = self.drain_values().await {
            if let Err(err) = self.dvu_tx.send(DvuTaskMessage::Dvu(values)) {
                error!(error = ?err, "Failed to enqueue dependent values update, receiver is closed");
            }
        }
    }

    async fn handle_dependent_values_update(&self, values: Vec<Ulid>) -> DvuDebouncerResult<()> {
        let mut ctx = self.ctx_builder.build_default().await?;

        ctx.update_visibility_deprecated(Visibility::new(self.change_set_id.into_inner().into()));
        ctx.update_tenancy(Tenancy::new(self.workspace_id.into_inner().into()));

        ctx.enqueue_dependent_values_update(values.into_iter().collect())
            .await?;

        // the pinga job will do the rebase
        ctx.blocking_commit_no_rebase().await?;

        Ok(())
    }
}

/// This task executes dependent values update jobs in serial
async fn dvu_task(debouncer: DvuDebouncer, mut rx: UnboundedReceiver<DvuTaskMessage>) {
    info!("booting dvu handler task for {}", &debouncer.change_set_id,);
    loop {
        select! {
            _ = debouncer.cancellation_token.cancelled() => {
                info!("DVU debouncer: DVU task received cancellation message");
                return;
            }
            Some(message) = rx.recv() => {
                match message {
                    DvuTaskMessage::Dvu(values) => {
                        info!("Enqueueing dependent values update job for {} values on change set {}", values.len(), debouncer.change_set_id);
                        if let Err(err) = debouncer.handle_dependent_values_update(values).await {
                            error!(error = ?err, "Attempt to enqueue dependent values update job failed");
                        }
                    }
                }
            }
        }
    }
}

async fn debouncer_task(debouncer: DvuDebouncer, mut rx: UnboundedReceiver<ValueTaskMessage>) {
    info!("booting dvu values task for {}", &debouncer.change_set_id,);

    let mut ticker = interval(DVU_INTERVAL);

    loop {
        select! {
            _ = debouncer.cancellation_token.cancelled() => {
                info!("DVU debouncer: value task received cancellation message");

                return;
            }
            _ = ticker.tick() => {
                debouncer.enqueue_dependent_values_update().await;
            }
            Some(message) = rx.recv() => {
                match message {
                    ValueTaskMessage::EnqueueValues(values) => {
                        debouncer.handle_enqueue_values(&values).await;
                    }
                }
            }
        }
    }
}
