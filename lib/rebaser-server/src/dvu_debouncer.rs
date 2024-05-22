//! A per-changeset task to debounce dependent values updates

use std::error::Error;

use chrono::{DateTime, Utc};
use dal::{
    ChangeSet, ChangeSetError, ChangeSetStatus, DalContextBuilder, Tenancy, TransactionsError,
    Visibility, WorkspaceSnapshotError,
};
use serde::{Deserialize, Serialize};
use si_data_nats::{
    async_nats::jetstream::{
        context::{CreateKeyValueError, PublishError, PublishErrorKind},
        kv::{Config as KvConfig, EntryError, PutError, Store as KvStore, UpdateError},
    },
    jetstream::{self, Context as JetstreamContext},
};
use si_events::{ChangeSetId, WorkspacePk};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{
    select,
    sync::mpsc::{error::SendError, unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{interval, Duration},
};
use tokio_util::{bytes::Bytes, sync::CancellationToken};
use ulid::Ulid;

const DVU_STATE_KEY: &str = "dvu_state";
const IDLE_TIME_KEY: &str = "idle_time";

const DVU_TIMEOUT: Duration = Duration::from_secs(45);
const DVU_QUIET_PERIOD: Duration = Duration::from_millis(2000);

/// DvuDebouncer error type
#[remain::sorted]
#[derive(Error, Debug)]
pub enum DvuDebouncerError {
    /// A Change set error
    #[error("change set: {0}")]
    ChangeSet(#[from] ChangeSetError),
    /// An error sending to an unbounded channel
    #[error("unable to send a message to the dvu debouncer task: {0}")]
    DvuMessageSend(#[from] SendError<DvuDebouncerMessage>),
    /// An error creating a key value bucket in the JetStream KV store
    #[error("jetstream kv bucket creation error: {0}")]
    JetStreamKvCreate(#[from] CreateKeyValueError),
    /// An error fetching a key value entry in the JetStream KV store
    #[error("jetstream kv entry error: {0}")]
    JetStreamKvEntry(#[from] EntryError),
    /// An error fetching putting a new key value entry into the JetStream KV store
    #[error("jetstream kv put error: {0}")]
    JetStreamKvPut(#[from] PutError),
    /// An error updating an existing key value entry into the JetStream KV store
    #[error("jetstream kv update error: {0}")]
    JetStreamKvUpdate(#[from] UpdateError),
    /// An error from the layer db
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    /// An error from serde json
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// A transactions error
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    /// Workspace Snapshot Error
    #[error("workspace snapshot: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

/// DvuDebouncer result type
type DvuDebouncerResult<T> = Result<T, DvuDebouncerError>;

/// Messages for the DVU debouncer task
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[remain::sorted]
pub enum DvuDebouncerMessage {
    /// Sent to the debouncer via the quiet time check
    QuietTimeCheck,
    /// Sent to the debouncer when a rebase suceeds
    RebaseSucceeded,
}

/// The DVU debouncer
#[derive(Clone, Debug)]
pub struct DvuDebouncer {
    id: Ulid,
    cancellation_token: CancellationToken,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    ctx_builder: DalContextBuilder,
    dvu_interval: Duration,
    dvu_idle_time: Duration,
    jetstream_ctx: JetstreamContext,
    bucket: String,
    dvu_message_tx: UnboundedSender<DvuDebouncerMessage>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum DvuState {
    Idle,
    Running,
}

#[derive(Serialize, Deserialize, Copy, Debug, Eq, PartialEq, Clone)]
#[remain::sorted]
enum DvuStatePayload {
    Idle {
        since: DateTime<Utc>,
    },
    Running {
        debouncer_id: Ulid,
        since: DateTime<Utc>,
    },
}

impl DvuDebouncer {
    /// Create a new dvu debouncer task
    pub fn new(
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        cancellation_token: CancellationToken,
        ctx_builder: DalContextBuilder,
        dvu_interval: Duration,
    ) -> Self {
        let jetstream_ctx = jetstream::new(ctx_builder.nats_conn().clone());
        let bucket = format!("{}-{}-dvu-test", workspace_id, change_set_id);

        let (dvu_message_tx, dvu_message_rx) = unbounded_channel();

        let debouncer = Self {
            id: Ulid::new(),
            cancellation_token,
            workspace_id,
            change_set_id,
            ctx_builder,
            dvu_interval,
            dvu_idle_time: DVU_QUIET_PERIOD,
            jetstream_ctx,
            bucket,
            dvu_message_tx,
        };

        let debouncer_clone = debouncer.clone();
        tokio::task::spawn(async { dvu_task(debouncer_clone, dvu_message_rx).await });

        let debouncer_clone = debouncer.clone();
        tokio::task::spawn(async { quiet_time_checker(debouncer_clone).await });

        debouncer
    }

    /// Send a message to this debouncer's message queue
    pub fn send_to_debouncer(&self, message: DvuDebouncerMessage) -> DvuDebouncerResult<()> {
        Ok(self.dvu_message_tx.send(message)?)
    }

    async fn kv_update(
        kv_store: KvStore,
        key: impl AsRef<str>,
        value: Bytes,
        revision: u64,
    ) -> DvuDebouncerResult<bool> {
        match kv_store.update(key, value, revision).await {
            Ok(_) => Ok(true),
            Err(err) => match err.source() {
                Some(source) => match source.downcast_ref::<PublishError>() {
                    Some(publish_err)
                        if PublishErrorKind::WrongLastSequence == publish_err.kind() =>
                    {
                        info!("raced while trying to update kv store");
                        Ok(false)
                    }
                    _ => Err(err)?,
                },
                None => Err(err)?,
            },
        }
    }

    async fn get_kv_store(&self) -> DvuDebouncerResult<KvStore> {
        Ok(self
            .jetstream_ctx
            .create_key_value(KvConfig {
                bucket: self.bucket.to_owned(),
                ..Default::default()
            })
            .await?)
    }

    async fn get_last_rebase_time(&self) -> DvuDebouncerResult<Option<DateTime<Utc>>> {
        Ok(
            if let Some(last_rebase_time_bytes) =
                self.get_kv_store().await?.get(IDLE_TIME_KEY).await?
            {
                Some(serde_json::from_slice(&last_rebase_time_bytes)?)
            } else {
                None
            },
        )
    }

    async fn set_last_rebase_time(&self) -> DvuDebouncerResult<()> {
        let now_string = serde_json::to_string(&Utc::now())?;
        let now_bytes = Bytes::from(now_string);
        self.get_kv_store()
            .await?
            .put(IDLE_TIME_KEY, now_bytes)
            .await?;
        Ok(())
    }

    // True means we changed state, false means we didn't, either because it
    // didn't make sense to (idle to idle, or running to running), or because we
    // couldn't (state was changed by another rebaser before we could)
    async fn maybe_update_dvu_state(&self, new_state: DvuState) -> DvuDebouncerResult<bool> {
        let now = Utc::now();

        let kv = self.get_kv_store().await?;

        let (current_state, revision) = match kv.entry(DVU_STATE_KEY).await? {
            Some(entry) => {
                let current_state: DvuStatePayload = serde_json::from_slice(&entry.value)?;
                (Some(current_state), entry.revision)
            }
            None => (None, 0),
        };

        let should_set = match (current_state, new_state) {
            (None, _) => true,
            (Some(DvuStatePayload::Idle { .. }), DvuState::Idle) => false,
            (Some(DvuStatePayload::Running { debouncer_id, .. }), DvuState::Idle) => {
                debouncer_id == self.id
            }
            (Some(DvuStatePayload::Idle { since, .. }), DvuState::Running) => {
                is_time_diff_gt(&now, &since, self.dvu_interval)
            }
            (Some(DvuStatePayload::Running { since, .. }), DvuState::Running) => {
                is_time_diff_gt(&now, &since, DVU_TIMEOUT)
            }
        };

        Ok(if should_set {
            let new_state_payload = match new_state {
                DvuState::Idle => DvuStatePayload::Idle { since: now },
                DvuState::Running => DvuStatePayload::Running {
                    debouncer_id: self.id,
                    since: now,
                },
            };

            let new_state_string = serde_json::to_string(&new_state_payload)?;
            let new_state_bytes = Bytes::from(new_state_string);
            if Self::kv_update(kv, DVU_STATE_KEY, new_state_bytes, revision).await? {
                info!("{}: set state: {:?}", self.id, new_state_payload);
                true
            } else {
                info!(
                    "{}: race on set, current_state: {:?}",
                    self.id, current_state
                );
                false
            }
        } else {
            false
        })
    }

    async fn run_dvu_if_values_pending(&self) -> DvuDebouncerResult<()> {
        let mut builder = self.ctx_builder.clone();
        // We shouldn't need to migrate any snapshots in the dvu debouncer. SDF
        // should do that when it reads the snapshot. Once rewritten, the
        // snapshot will already be migrated when we get here.
        builder.set_no_auto_migrate_snapshots();
        let mut ctx = builder.build_default().await?;

        ctx.update_visibility_deprecated(Visibility::new(self.change_set_id.into_inner().into()));
        ctx.update_tenancy(Tenancy::new(self.workspace_id.into_inner().into()));

        if let Some(change_set) =
            ChangeSet::find(&ctx, self.change_set_id.into_inner().into()).await?
        {
            if !matches!(
                change_set.status,
                ChangeSetStatus::Open
                    | ChangeSetStatus::NeedsApproval
                    | ChangeSetStatus::NeedsAbandonApproval
            ) {
                debug!("change set no longer open, not enqueuing dependent values updates");
                return Ok(());
            }
        }

        if let Err(err) = ctx.update_snapshot_to_visibility().await {
            if err.is_unmigrated_snapshot_error() {
                warn!("Snapshot not yet migrated. Not attempting dvu");
            } else {
                Err(err)?
            }
        }

        if ctx
            .workspace_snapshot()?
            .has_dependent_value_roots()
            .await?
        {
            let run_id = Ulid::new();
            if self.maybe_update_dvu_state(DvuState::Running).await? {
                info!("{}: enqueing dvu ({})", self.id, run_id);
                if let Err(err) = ctx.enqueue_dependent_values_update().await {
                    self.maybe_update_dvu_state(DvuState::Idle).await?;
                    return Err(err)?;
                }

                info!("{}: enqeued ({})", self.id, run_id);
                if let Err(err) = ctx.blocking_commit_no_rebase().await {
                    self.maybe_update_dvu_state(DvuState::Idle).await?;
                    return Err(err)?;
                }
                info!("{}: committed, no rebase ({})", self.id, run_id);

                if !self.maybe_update_dvu_state(DvuState::Idle).await? {
                    info!("{}: did not set to idle (race {})", self.id, run_id);
                }
            } else {
                info!("{}: another rebaser picked up the job {}", self.id, run_id);
            }
        } else {
            self.maybe_update_dvu_state(DvuState::Idle).await?;
        }

        Ok(())
    }
}

async fn quiet_time_checker(debouncer: DvuDebouncer) {
    // TODO: jitter the interval here so that each rebaser is checking at a slightly different interval
    let mut ticker = interval(Duration::from_millis(2000));

    loop {
        select! {
            _ = debouncer.cancellation_token.cancelled() => {
                info!("DVU debouncer: value task received cancellation message");

                return;
            }
            _ = ticker.tick() => {
                match debouncer.get_last_rebase_time().await {
                    Ok(Some(last_rebase_time)) if is_time_diff_gt(&Utc::now(), &last_rebase_time, debouncer.dvu_idle_time) => {
                        info!("idle time check succeeded");
                    }
                    Err(err) => {
                        error!(error=?err, "Failed to get last rebase time");
                    }
                    _ => {}
                }
            }
        }
    }
}

async fn dvu_task(debouncer: DvuDebouncer, mut rx: UnboundedReceiver<DvuDebouncerMessage>) {
    info!(
        "booting dvu task {} for {} with a {:?} interval",
        debouncer.id, &debouncer.change_set_id, debouncer.dvu_interval
    );

    if let Err(err) = debouncer.maybe_update_dvu_state(DvuState::Idle).await {
        error!(error=?err, "Unable to set initial idle state for dvu_task on change set {}", debouncer.change_set_id);
        return;
    }

    loop {
        select! {
            _ = debouncer.cancellation_token.cancelled() => {
                info!("DVU debouncer: value task received cancellation message");

                return;
            }
            Some(msg) = rx.recv() => {
                if msg == DvuDebouncerMessage::RebaseSucceeded {
                    if let Err(err) = debouncer.set_last_rebase_time().await {
                        error!(error=?err, "Failed to update last rebase time");
                    }
                }
                if let Err(err) = debouncer.run_dvu_if_values_pending().await {
                    error!(error = ?err, "Attempt to run dependent values update job failed for changeset {}", debouncer.change_set_id);
                }
            }
        }
    }
}

fn is_time_diff_gt(now: &DateTime<Utc>, then: &DateTime<Utc>, duration: Duration) -> bool {
    now.timestamp_micros()
        .checked_sub(then.timestamp_micros())
        .map(|diff| (diff > 0) && (Duration::from_micros(diff as u64) > duration))
        .unwrap_or(false)
}
