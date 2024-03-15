//! The sequel to [`ChangeSets`](crate::ChangeSet). Coming to an SI instance near you!

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgRow};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::{Generator, Ulid};

use crate::context::RebaseRequest;
use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::workspace_snapshot::WorkspaceSnapshotId;
use crate::{
    id, ChangeSetStatus, DalContext, HistoryEvent, HistoryEventError, TransactionsError, UserPk,
    Workspace, WorkspacePk, WsEvent, WsEventError, WsEventResult, WsPayload,
};

pub mod status;
pub mod view;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("could not find default change set: {0}")]
    DefaultChangeSetNotFound(ChangeSetId),
    #[error("default change set {0} has no workspace snapshot pointer")]
    DefaultChangeSetNoWorkspaceSnapshotPointer(ChangeSetId),
    #[error("enum parse error: {0}")]
    EnumParse(#[from] strum::ParseError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("ulid monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("mutex error: {0}")]
    Mutex(String),
    #[error("Changeset {0} does not have a base change set")]
    NoBaseChangeSet(ChangeSetId),
    #[error("no tenancy set in context")]
    NoTenancySet,
    #[error("Changeset {0} does not have a workspace snapshot")]
    NoWorkspaceSnapshot(ChangeSetId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("found an unexpected number of open change sets matching default change set (should be one, found {0:?})")]
    UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(Vec<ChangeSetId>),
    #[error("workspace error: {0}")]
    Workspace(String),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

id!(ChangeSetId);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: ChangeSetId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name: String,
    pub status: ChangeSetStatus,
    pub base_change_set_id: Option<ChangeSetId>,
    pub workspace_snapshot_id: Option<WorkspaceSnapshotId>,
    pub workspace_id: Option<WorkspacePk>,

    #[serde(skip)]
    pub generator: Arc<Mutex<Generator>>,
}

impl TryFrom<PgRow> for ChangeSet {
    type Error = ChangeSetError;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let status_string: String = value.try_get("status")?;
        let status = ChangeSetStatus::try_from(status_string.as_str())?;
        Ok(Self {
            id: value.try_get("id")?,
            created_at: value.try_get("created_at")?,
            updated_at: value.try_get("updated_at")?,
            name: value.try_get("name")?,
            status,
            base_change_set_id: value.try_get("base_change_set_id")?,
            workspace_snapshot_id: value.try_get("workspace_snapshot_id")?,
            workspace_id: value.try_get("workspace_id")?,
            generator: Arc::new(Mutex::new(Default::default())),
        })
    }
}

impl ChangeSet {
    pub fn new_local() -> ChangeSetResult<Self> {
        let mut generator = Generator::new();
        let id = generator.generate()?;

        Ok(Self {
            id: id.into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            generator: Arc::new(Mutex::new(generator)),
            base_change_set_id: None,
            workspace_snapshot_id: None,
            workspace_id: None,
            name: "".to_string(),
            status: ChangeSetStatus::Open,
        })
    }

    pub fn editing_changeset(&self) -> ChangeSetResult<Self> {
        let mut new_local = Self::new_local()?;
        new_local.base_change_set_id = self.base_change_set_id;
        new_local.workspace_snapshot_id = self.workspace_snapshot_id;
        new_local.workspace_id = self.workspace_id;
        new_local.name = self.name.to_owned();
        new_local.status = self.status.to_owned();
        Ok(new_local)
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        base_change_set_id: Option<ChangeSetId>,
    ) -> ChangeSetResult<Self> {
        let workspace_id = ctx.tenancy().workspace_pk();
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO change_set_pointers (name, base_change_set_id, status, workspace_id) VALUES ($1, $2, $3, $4) RETURNING *",
                &[&name, &base_change_set_id, &ChangeSetStatus::Open.to_string(), &workspace_id],
            )
            .await?;
        let change_set = Self::try_from(row)?;
        let _history_event = HistoryEvent::new(
            ctx,
            "change_set.create",
            "Change Set created",
            &serde_json::to_value(&change_set)?,
        )
        .await?;
        Ok(change_set)
    }

    pub async fn fork_head(ctx: &DalContext, name: impl AsRef<str>) -> ChangeSetResult<Self> {
        let workspace_pk = ctx
            .tenancy()
            .workspace_pk()
            .ok_or(ChangeSetError::NoTenancySet)?;

        let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
            .await
            .map_err(|err| ChangeSetError::Workspace(err.to_string()))?
            .ok_or(ChangeSetError::WorkspaceNotFound(workspace_pk))?;

        let base_change_set = ChangeSet::find(ctx, workspace.default_change_set_id())
            .await?
            .ok_or(ChangeSetError::DefaultChangeSetNotFound(
                workspace.default_change_set_id(),
            ))?;

        let mut change_set =
            ChangeSet::new(ctx, name, Some(workspace.default_change_set_id())).await?;

        change_set
            .update_pointer(
                ctx,
                base_change_set.workspace_snapshot_id.ok_or(
                    ChangeSetError::DefaultChangeSetNoWorkspaceSnapshotPointer(
                        workspace.default_change_set_id(),
                    ),
                )?,
            )
            .await?;

        Ok(change_set)
    }

    /// Create a [`VectorClockId`] from the [`ChangeSet`].
    pub fn vector_clock_id(&self) -> VectorClockId {
        VectorClockId::from(Ulid::from(self.id))
    }

    pub fn generate_ulid(&self) -> ChangeSetResult<Ulid> {
        self.generator
            .lock()
            .map_err(|e| ChangeSetError::Mutex(e.to_string()))?
            .generate()
            .map_err(Into::into)
    }

    pub async fn update_workspace_id(
        &mut self,
        ctx: &DalContext,
        workspace_id: WorkspacePk,
    ) -> ChangeSetResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_sets SET workspace_id = $2 WHERE id = $1",
                &[&self.id, &workspace_id],
            )
            .await?;

        self.workspace_id = Some(workspace_id);

        Ok(())
    }

    pub async fn update_pointer(
        &mut self,
        ctx: &DalContext,
        workspace_snapshot_id: WorkspaceSnapshotId,
    ) -> ChangeSetResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET workspace_snapshot_id = $2 WHERE id = $1",
                &[&self.id, &workspace_snapshot_id],
            )
            .await?;

        self.workspace_snapshot_id = Some(workspace_snapshot_id);

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        ctx: &DalContext,
        status: ChangeSetStatus,
    ) -> ChangeSetResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET status = $2 WHERE id = $1",
                &[&self.id, &status.to_string()],
            )
            .await?;

        self.status = status;

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn find(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> ChangeSetResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT * FROM change_set_pointers WHERE id = $1",
                &[&change_set_id],
            )
            .await?;

        match row {
            Some(row) => Ok(Some(Self::try_from(row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_open(ctx: &DalContext) -> ChangeSetResult<Vec<Self>> {
        let mut result = vec![];
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE workspace_id = $1 AND status = $2",
                &[
                    &ctx.tenancy().workspace_pk(),
                    &ChangeSetStatus::Open.to_string(),
                ],
            )
            .await?;

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    pub async fn apply_to_base_change_set(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let to_rebase_change_set_id = self
            .base_change_set_id
            .ok_or(ChangeSetError::NoBaseChangeSet(self.id))?;
        let onto_workspace_snapshot_id = self
            .workspace_snapshot_id
            .ok_or(ChangeSetError::NoWorkspaceSnapshot(self.id))?;
        let rebase_request = RebaseRequest {
            onto_workspace_snapshot_id,
            onto_vector_clock_id: self.vector_clock_id(),
            to_rebase_change_set_id,
        };
        ctx.do_rebase_request(rebase_request).await?;

        self.update_status(ctx, ChangeSetStatus::Applied).await?;

        Ok(())
    }

    pub async fn force_new(ctx: &mut DalContext) -> ChangeSetResult<Option<ChangeSetId>> {
        let maybe_fake_pk =
            if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
                let change_set = Self::fork_head(ctx, Self::generate_name()).await?;
                ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
                    .await?;

                WsEvent::change_set_created(ctx, change_set.id)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;

                Some(change_set.id)
            } else {
                None
            };
        Ok(maybe_fake_pk)
    }

    fn generate_name() -> String {
        Utc::now().format("%Y-%m-%d-%H:%M").to_string()
    }
}

impl std::fmt::Debug for ChangeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeSet")
            .field("id", &self.id.to_string())
            .field(
                "base_change_set_id",
                &self.base_change_set_id.map(|bcsid| bcsid.to_string()),
            )
            .field(
                "workspace_snapshot_id",
                &self.workspace_snapshot_id.map(|wsid| wsid.to_string()),
            )
            .finish()
    }
}

impl WsEvent {
    pub async fn change_set_created(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCreated(change_set_id)).await
    }

    pub async fn change_set_written(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetWritten(ctx.change_set_id())).await
    }

    pub async fn change_set_abandoned(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetAbandoned(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_applied(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetApplied(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_canceled(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ChangeSetCanceled(change_set_id)).await
    }

    pub async fn change_set_merge_vote(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: UserPk,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetMergeVote(ChangeSetMergeVotePayload {
                change_set_id,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn change_set_begin_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetBeginApprovalProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_cancel_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCancelApprovalProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_abandon_vote(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: UserPk,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetAbandonVote(ChangeSetMergeVotePayload {
                change_set_id,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn change_set_begin_abandon_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetBeginAbandonProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }

    pub async fn change_set_cancel_abandon_approval_process(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ChangeSetCancelAbandonProcess(ChangeSetActorPayload {
                change_set_id,
                user_pk,
            }),
        )
        .await
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetActorPayload {
    change_set_id: ChangeSetId,
    user_pk: Option<UserPk>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetMergeVotePayload {
    change_set_id: ChangeSetId,
    user_pk: UserPk,
    vote: String,
}
