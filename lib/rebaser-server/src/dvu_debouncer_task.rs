//! A per-changeset task to debounce dependent values updates

use std::{str::Utf8Error, time::Duration};

use dal::{
    workspace_snapshot::graph::WorkspaceSnapshotGraphDiscriminants, ChangeSet, ChangeSetStatus,
    DalContextBuilder, Tenancy, Visibility, Workspace,
};
use futures::StreamExt as _;
use serde::{Deserialize, Serialize};
use si_data_nats::{async_nats::jetstream::kv, Subject};
use si_events::{ChangeSetId, WorkspacePk};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::{sync::mpsc, time};
use tokio_util::sync::CancellationToken;

/// An error that can be returned when running a "depdendent values update" debouncer task.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DvuDebouncerTaskError {
    /// When working with a change set
    #[error("change set: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    /// When sending a control operation to an internal child task
    #[error("keepalive error when sending ctrl op")]
    KeepaliveCtrlSend,
    /// When an internal child task is shutting down
    #[error("keepalive task is shutting down; ctl op rejected")]
    KeepaliveCtrlShutdown,
    /// When an internal child task fails to shut down cleanly
    #[error("keepalive task join errored")]
    KeepaliveTaskJoin,
    /// When a KV key fails to be created
    #[error("kv create error")]
    KvCreate(#[source] kv::CreateError),
    /// When a KV key fails to be purged
    #[error("kv purge error; err={0:?}, revision={1}, key={2}")]
    KvPurge(#[source] kv::PurgeError, u64, String),
    /// When a KV key fails to be purged
    #[error("kv purge error; err={0:?}, key={1}")]
    KvPurgeNoRevision(#[source] kv::PurgeError, String),
    /// When failing to fetch a KV key status
    #[error("kv status error")]
    KvStatus(#[source] kv::StatusError),
    /// When a KV key fails to be updated
    #[error("kv update value error; err={0:?}, revision={1}, key={2}")]
    KvUpdate(#[source] kv::UpdateError, u64, String),
    /// When failing to construct a KV key watch subscription
    #[error("kv watch error: {0}")]
    KvWatch(#[source] kv::WatchError),
    /// When watch_with_history() stream unexpectedly ends
    #[error("kv watch with history unexpectedly ended")]
    KvWatchWithHistoryEnded,
    /// When failing to serialize a type to json
    #[error("serialize error: {0}")]
    Serialize(#[source] serde_json::Error),
    /// When failing to serialize kv state to json
    #[error("failed to serialize kv state")]
    SerializeState(#[source] serde_json::Error),
    /// When failing to create a DAL context
    #[error("Transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    /// When parsing a string from bytes
    #[error("error when parsing string from bytes: {0}")]
    Uft8(#[from] Utf8Error),
    /// Workspace error
    #[error("workspace error: {0}")]
    Workspace(#[from] dal::WorkspaceError),
    /// When working with a workspace snapshot
    #[error("workspace snapshot: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

type DvuDebouncerTaskResult<T> = Result<T, DvuDebouncerTaskError>;

#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum KvStatus {
    Running,
    Waiting,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct KvState {
    instance_id: String,
    status: KvStatus,
}

#[remain::sorted]
#[derive(Debug)]
#[allow(clippy::enum_variant_names)] // Variant names are more descriptive with the shared postfix
enum DebouncerState {
    Cancelled,
    RunningAsLeader((KvState, u64)),
    WaitingToBecomeLeader,
}

/// A per-change set task to debounce dependent values updates.
#[derive(Debug)]
pub struct DvuDebouncerTask {
    instance_id: String,
    kv: kv::Store,
    watch_subject: Subject,
    workspace_id: WorkspacePk,
    change_set_id: ChangeSetId,
    ctx_builder: DalContextBuilder,
    interval_duration: Duration,
    token: CancellationToken,
    restarted_count: usize,
}

impl DvuDebouncerTask {
    const NAME: &'static str = "rebaser_server::dvu_debouncer_task";

    /// Creates and returns a runnable [`DvuDebouncerTask`].
    pub fn create(
        instance_id: String,
        kv: kv::Store,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        ctx_builder: DalContextBuilder,
        interval_duration: Duration,
    ) -> DvuDebouncerTaskResult<Self> {
        let watch_subject = Subject::from_utf8(format!("{workspace_id}.{change_set_id}"))?;

        Ok(Self {
            instance_id,
            kv,
            watch_subject,
            workspace_id,
            change_set_id,
            ctx_builder,
            interval_duration,
            token: CancellationToken::new(),
            restarted_count: 0,
        })
    }

    /// Returns a [`CancellationToken`] which can be used to cancel this task.
    pub fn cancellation_token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Runs the service to completion and will restart when an internal error is encountered.
    #[inline]
    pub async fn run(mut self) {
        loop {
            match self.try_run().await {
                Ok(_) => break,
                Err(err) => {
                    warn!(
                        task = Self::NAME,
                        error = ?err,
                        key = self.watch_subject.to_string(),
                        restarted_count = self.restarted_count,
                        "error found while running task; restarting task",
                    );
                    self.restarted_count += 1;
                }
            }
        }
    }

    /// Runs the service to completion, returning its result (i.e. whether it successful or an
    /// internal error was encountered).
    async fn try_run(&mut self) -> DvuDebouncerTaskResult<()> {
        // Set initial state of waiting to become leader
        let mut state = DebouncerState::WaitingToBecomeLeader;

        loop {
            state = match state {
                DebouncerState::WaitingToBecomeLeader => self.waiting_to_become_leader().await?,
                DebouncerState::RunningAsLeader((kv_state, revision)) => {
                    self.running_as_leader(kv_state, revision).await?
                }
                DebouncerState::Cancelled => break,
            };
        }

        debug!(
            task = Self::NAME,
            key = self.watch_subject.to_string(),
            "shutdown complete",
        );
        Ok(())
    }

    async fn waiting_to_become_leader(&mut self) -> DvuDebouncerTaskResult<DebouncerState> {
        info!(
            task = Self::NAME,
            key = self.watch_subject.to_string(),
            "waiting to become leader",
        );

        let mut watch = self
            .kv
            .watch_with_history(self.watch_subject.as_str())
            .await
            .map_err(DvuDebouncerTaskError::KvWatch)?;

        let mut check_missing_key_interval = time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                biased;

                // Cancellation token has fired, time to shut down
                _ = self.token.cancelled() => {
                    debug!(
                        task = Self::NAME,
                        key = self.watch_subject.to_string(),
                        state = "waiting_to_become_leader",
                        "received cancellation",
                    );
                    // Beggining to shut dow, return to break out of try_run loop
                    return Ok(DebouncerState::Cancelled);
                }
                // Received next watch item
                maybe_entry_result = watch.next() => {
                    match maybe_entry_result {
                        // Next item is an watch entry
                        Some(Ok(entry)) => match self.process_entry_update(entry).await {
                            Ok(Some(new_state)) => return Ok(new_state),
                            Ok(None) => {},
                            Err(err) => {
                                warn!(
                                    task = Self::NAME,
                                    key = self.watch_subject.to_string(),
                                    error = ?err,
                                    "failed to process update message",
                                );
                            }
                        },
                        // Next item is an error
                        Some(Err(err)) => {
                            warn!(
                                task = Self::NAME,
                                key = self.watch_subject.to_string(),
                                error = ?err,
                                "failed to process message",
                            );
                        }
                        // Watch stream has ended
                        // End of watch stream, return to break out of try_run loop
                        None => return Err(DvuDebouncerTaskError::KvWatchWithHistoryEnded),
                    }
                }
                // Interval for checking for missing key has ticked
                _ = check_missing_key_interval.tick() => {
                    if let Some(new_state) = self.attempt_to_acquire_key().await? {
                        return Ok(new_state);
                    }
                }
            }
        }
    }

    async fn running_as_leader(
        &mut self,
        kv_state: KvState,
        revision: u64,
    ) -> DvuDebouncerTaskResult<DebouncerState> {
        info!(
            task = Self::NAME,
            key = self.watch_subject.to_string(),
            "running as leader",
        );

        let task_token = CancellationToken::new();
        let task = DvuDebouncerKeepaliveTask::new(
            self.kv.clone(),
            self.watch_subject.clone(),
            kv_state,
            revision,
            task_token.clone(),
        )
        .await?;
        let keepalive = task.ctl();
        // Convert the cancellation token into a drop guard to ensure task is cancelled no matter
        // what
        let task_drop_guard = task_token.drop_guard();
        let task_handle = tokio::spawn(task.try_run());

        // Don't early-return on errors as we want to clean up the keepalive task
        let inner_result = self.running_as_leader_inner(keepalive).await;

        // Cancel the keepalive task and await its completion. On success it returns the revision
        // of the key
        debug!(
            task = Self::NAME,
            key = self.watch_subject.to_string(),
            "cancelling keepalive"
        );
        task_drop_guard.disarm().cancel();
        match task_handle
            .await
            .map_err(|_err| DvuDebouncerTaskError::KeepaliveTaskJoin)?
        {
            Ok(revision) => {
                self.purge_key(Some(revision)).await?;
            }
            Err(task_err) => {
                warn!(
                    task = Self::NAME,
                    key = self.watch_subject.to_string(),
                    error = ?task_err,
                    "error found while awaiting keepalive task",
                );
                self.purge_key(None).await?;
            }
        };

        info!(
            task = Self::NAME,
            key = self.watch_subject.to_string(),
            "demoting self as leader",
        );
        inner_result
    }

    async fn running_as_leader_inner(
        &mut self,
        keepalive: DvuDebouncerKeepalive,
    ) -> DvuDebouncerTaskResult<DebouncerState> {
        let mut interval = time::interval_at(
            time::Instant::now() + self.interval_duration,
            self.interval_duration,
        );

        loop {
            tokio::select! {
                biased;

                // Cancellation token has fired, time to shut down
                _ = self.token.cancelled() => {
                    debug!(
                        task = Self::NAME,
                        key = self.watch_subject.to_string(),
                        state = "running_as_leader",
                        "received cancellation",
                    );
                    // Beggining to shut down, return to break out of try_run loop
                    return Ok(DebouncerState::Cancelled);
                }
                // Interval for running dependent values update if values are pending has ticked
                _ = interval.tick() => {
                    // This will block the next `select` which is intended as we want a depdendent
                    // values update to be allowed to run to completion before checking to see if
                    // the cancellation token has fired in the meantime.
                    if let Some(new_state) = self.run_dvu_if_values_pending(&keepalive).await? {
                        // Dependent values update has run, return to continue try_run loop
                        return Ok(new_state);
                    }
                }
            }
        }
    }

    #[inline]
    async fn process_entry_update(
        &mut self,
        entry: kv::Entry,
    ) -> DvuDebouncerTaskResult<Option<DebouncerState>> {
        match entry.operation {
            // The key has been deleted/purged so we should try to become leader
            kv::Operation::Delete | kv::Operation::Purge => self.attempt_to_acquire_key().await,
            // Ingore updates to key--an instance is currently leader and keeping the key alive
            kv::Operation::Put => {
                trace!(
                    task = Self::NAME,
                    key = entry.key.as_str(),
                    "received update message",
                );

                // No leader changes, return to continue waiting to become leader loop
                Ok(None)
            }
        }
    }

    async fn attempt_to_acquire_key(&mut self) -> DvuDebouncerTaskResult<Option<DebouncerState>> {
        let kv_state = KvState {
            instance_id: self.instance_id.clone(),
            status: KvStatus::Waiting,
        };

        let value = serde_json::to_vec(&kv_state).map_err(DvuDebouncerTaskError::Serialize)?;

        match self
            .kv
            .create(self.watch_subject.as_str(), value.into())
            .await
        {
            // Success: we should set up to be the leader
            // State change, return to break out of waiting to become leader loop
            Ok(revision) => Ok(Some(DebouncerState::RunningAsLeader((kv_state, revision)))),
            Err(err) => {
                if !matches!(err.kind(), kv::CreateErrorKind::AlreadyExists) {
                    warn!(
                        task = Self::NAME,
                        key = self.watch_subject.to_string(),
                        error = ?err,
                        "unexpected error while attempting to create key",
                    );
                }

                // Lost race to become leader, return to continue waiting to become
                // leader loop
                Ok(Some(DebouncerState::WaitingToBecomeLeader))
            }
        }
    }

    async fn purge_key(&self, maybe_revision: Option<u64>) -> DvuDebouncerTaskResult<()> {
        match maybe_revision {
            Some(revision) => {
                // Purge the key with the expected revision
                if let Err(err) = self
                    .kv
                    .purge_expect_revision(self.watch_subject.as_str(), Some(revision))
                    .await
                {
                    warn!(
                        task = Self::NAME,
                        key = self.watch_subject.to_string(),
                        expected_revision = revision,
                        error = ?err,
                        "failed to purge key with expected revision",
                    );
                    // TODO remove. If we failed to purge the key, it could be because someone
                    // else became leader. The key will age itself out.
                    // Attempt to purge the key without revision--as we are the leader this is our
                    // data
                    self.kv
                        .purge(self.watch_subject.as_str())
                        .await
                        .map_err(|err| {
                            DvuDebouncerTaskError::KvPurgeNoRevision(
                                err,
                                self.watch_subject.to_string(),
                            )
                        })?;
                }
            }
            None => {
                // TODO remove. If we failed to purge the key, it could be because someone
                // else became leader. The key will age itself out.
                // Attempt to purge the key--as we are the leader this is our data
                self.kv
                    .purge(self.watch_subject.as_str())
                    .await
                    .map_err(|err| {
                        DvuDebouncerTaskError::KvPurgeNoRevision(
                            err,
                            self.watch_subject.to_string(),
                        )
                    })?;
            }
        };

        Ok(())
    }

    async fn run_dvu_if_values_pending(
        &mut self,
        keepalive: &DvuDebouncerKeepalive,
    ) -> DvuDebouncerTaskResult<Option<DebouncerState>> {
        let builder = self.ctx_builder.clone();
        let mut ctx = builder.build_default().await?;

        ctx.update_visibility_deprecated(Visibility::new(self.change_set_id.into_raw_id().into()));
        ctx.update_tenancy(Tenancy::new(self.workspace_id.into_raw_id().into()));

        if let Some(workspace) =
            Workspace::get_by_pk(&ctx, &self.workspace_id.into_raw_id().into()).await?
        {
            if workspace.snapshot_version() != WorkspaceSnapshotGraphDiscriminants::V2 {
                debug!("snapshot not yet migrated; not attempting dependent values update");
                // No depdendent values update to perform, return to continue running as leader loop
                return Ok(None);
            }
        }

        if let Some(change_set) =
            ChangeSet::find(&ctx, self.change_set_id.into_raw_id().into()).await?
        {
            if !matches!(
                change_set.status,
                ChangeSetStatus::Open
                    | ChangeSetStatus::NeedsApproval
                    | ChangeSetStatus::NeedsAbandonApproval
            ) {
                debug!(
                    task = Self::NAME,
                    si.workspace.id = %self.workspace_id,
                    si.change_set.id = %self.change_set_id,
                    "change set no longer open, not enqueuing dependent values updates",
                );
                return Ok(None);
            }
        }

        ctx.update_snapshot_to_visibility().await?;

        if ctx
            .workspace_snapshot()?
            .has_dependent_value_roots()
            .await?
        {
            keepalive.update_status_to_running().await?;

            info!(
                task = Self::NAME,
                si.workspace.id = %self.workspace_id,
                si.change_set.id = %self.change_set_id,
                "enqueuing dependent_values_update",
            );
            ctx.enqueue_dependent_values_update().await?;
            ctx.blocking_commit_no_rebase().await?;

            // Finished as leader, return to break out of running as leader loop
            Ok(Some(DebouncerState::WaitingToBecomeLeader))
        } else {
            // No depdendent values update to perform, return to continue running as leader loop
            Ok(None)
        }
    }
}

#[remain::sorted]
#[derive(Debug)]
enum KeepaliveOp {
    UpdateStatusToRunning,
}

#[derive(Debug)]
struct DvuDebouncerKeepalive(mpsc::Sender<KeepaliveOp>);

impl DvuDebouncerKeepalive {
    async fn update_status_to_running(&self) -> DvuDebouncerTaskResult<()> {
        self.send_op(KeepaliveOp::UpdateStatusToRunning)
            .await
            .map_err(Into::into)
    }

    async fn send_op(&self, op: KeepaliveOp) -> DvuDebouncerTaskResult<()> {
        if self.0.is_closed() {
            return Err(DvuDebouncerTaskError::KeepaliveCtrlShutdown);
        }

        self.0
            .send(op)
            .await
            .map_err(|_err| DvuDebouncerTaskError::KeepaliveCtrlSend)
    }
}

#[derive(Debug)]
struct DvuDebouncerKeepaliveTask {
    kv: kv::Store,
    key: Subject,
    state: KvState,
    revision: u64,
    interval_duration: Duration,
    ops_rx: mpsc::Receiver<KeepaliveOp>,
    _ops_tx: mpsc::Sender<KeepaliveOp>,
    token: CancellationToken,
}

impl DvuDebouncerKeepaliveTask {
    const NAME: &'static str = "rebaser_server::dvu_debouncer_keepalive_task";

    async fn new(
        kv: kv::Store,
        key: Subject,
        kv_state: KvState,
        revision: u64,
        token: CancellationToken,
    ) -> DvuDebouncerTaskResult<Self> {
        // We want to keep the key from aging out and so want our interval to be *less* than the
        // key time-to-live
        let max_age = kv
            .status()
            .await
            .map_err(DvuDebouncerTaskError::KvStatus)?
            .max_age();
        let interval_duration = Duration::from_secs_f64(max_age.as_secs_f64() * 0.85);

        let (_ops_tx, ops_rx) = mpsc::channel(4);

        Ok(Self {
            kv,
            key,
            state: kv_state,
            revision,
            interval_duration,
            ops_rx,
            _ops_tx,
            token,
        })
    }

    fn ctl(&self) -> DvuDebouncerKeepalive {
        DvuDebouncerKeepalive(self._ops_tx.clone())
    }

    async fn try_run(mut self) -> DvuDebouncerTaskResult<u64> {
        let mut interval = time::interval(self.interval_duration);

        loop {
            tokio::select! {
                biased;

                // Cancellation token has fired, time to shut down
                _ = self.token.cancelled() => {
                    debug!(
                        task = Self::NAME,
                        key = self.key.to_string(),
                        "received cancellation",
                    );
                    break;
                }
                // Interval for updating state key has ticked
                _ = interval.tick() => self.update_entry().await?,
                // There is a next op value on the channel
                maybe_op = self.ops_rx.recv() => match maybe_op {
                    // We have an op value, process it
                    Some(op) => self.process_op(op).await?,
                    // No more op values, channel is drained, we can break to finish shutdown
                    None => break,
                }
            }
        }

        debug!(
            task = Self::NAME,
            key = self.key.to_string(),
            "shutdown complete",
        );
        Ok(self.revision)
    }

    #[inline]
    async fn process_op(&mut self, op: KeepaliveOp) -> DvuDebouncerTaskResult<()> {
        match op {
            KeepaliveOp::UpdateStatusToRunning => {
                self.state.status = KvStatus::Running;
                self.update_entry().await
            }
        }
    }

    async fn update_entry(&mut self) -> DvuDebouncerTaskResult<()> {
        let value =
            serde_json::to_vec(&self.state).map_err(DvuDebouncerTaskError::SerializeState)?;
        trace!(
            task = Self::NAME,
            key = self.key.as_str(),
            last_revision = self.revision,
            "updating entry"
        );
        let new_revision = self
            .kv
            .update(self.key.as_str(), value.into(), self.revision)
            .await
            .map_err(|err| {
                DvuDebouncerTaskError::KvUpdate(err, self.revision, self.key.to_string())
            })?;
        self.revision = new_revision;

        Ok(())
    }
}
