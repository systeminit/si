use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;

use billing_events::{
    BillingEventsError, BillingWorkspaceChangeEvent, BillingWorkspaceChangeEventLocation,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_layer_cache::LayerDbError;
use thiserror::Error;

use si_data_pg::{PgError, PgRow};
use si_events::{ulid::Ulid, WorkspaceSnapshotAddress};
use telemetry::prelude::*;

use crate::context::RebaseRequest;
use crate::slow_rt::SlowRuntimeError;
use crate::workspace_snapshot::graph::RebaseBatch;
use crate::WorkspaceError;
use crate::{
    action::{ActionError, ActionId},
    id, ChangeSetStatus, ComponentError, DalContext, HistoryActor, HistoryEvent, HistoryEventError,
    TransactionsError, User, UserError, UserPk, Workspace, WorkspacePk, WorkspaceSnapshot,
    WorkspaceSnapshotError, WsEvent, WsEventError,
};

pub mod event;
pub mod status;
pub mod view;

const FIND_ANCESTORS_QUERY: &str = include_str!("queries/change_set/find_ancestors.sql");

/// The primary error type for this module.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("billing events error: {0}")]
    BillingEvents(#[from] BillingEventsError),
    #[error("change set with id {0} not found")]
    ChangeSetNotFound(ChangeSetId),
    #[error("could not find default change set: {0}")]
    DefaultChangeSetNotFound(ChangeSetId),
    #[error("default change set {0} has no workspace snapshot pointer")]
    DefaultChangeSetNoWorkspaceSnapshotPointer(ChangeSetId),
    #[error("enum parse error: {0}")]
    EnumParse(#[from] strum::ParseError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid user actor pk")]
    InvalidActor(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
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
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("found an unexpected number of open change sets matching default change set (should be one, found {0:?})")]
    UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(Vec<ChangeSetId>),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<WorkspaceError> for ChangeSetError {
    fn from(value: WorkspaceError) -> Self {
        Self::Workspace(Box::new(value))
    }
}

impl From<WsEventError> for ChangeSetError {
    fn from(value: WsEventError) -> Self {
        Self::WsEvent(Box::new(value))
    }
}

/// The primary result type for this module.
pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

/// A superset of [`ChangeSetError`] used when performing apply logic.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetApplyError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action prototype not found for id: {0}")]
    ActionPrototypeNotFound(ActionId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set not found by id: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("invalid user: {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("change set ({0}) does not have a base change set")]
    NoBaseChangeSet(ChangeSetId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("user error: {0}")]
    User(#[from] UserError),
}

/// A superset of [`ChangeSetResult`] used when performing apply logic.
pub type ChangeSetApplyResult<T> = Result<T, ChangeSetApplyError>;

id!(ChangeSetId);

impl From<ChangeSetId> for si_events::ChangeSetId {
    fn from(value: ChangeSetId) -> Self {
        let id: ulid::Ulid = value.into();
        id.into()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub id: ChangeSetId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name: String,
    pub status: ChangeSetStatus,
    pub base_change_set_id: Option<ChangeSetId>,
    pub workspace_snapshot_address: WorkspaceSnapshotAddress,
    pub workspace_id: Option<WorkspacePk>,
    pub merge_requested_by_user_id: Option<UserPk>,
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
            workspace_snapshot_address: value.try_get("workspace_snapshot_address")?,
            workspace_id: value.try_get("workspace_id")?,
            merge_requested_by_user_id: value.try_get("merge_requested_by_user_id")?,
        })
    }
}

impl ChangeSet {
    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        base_change_set_id: Option<ChangeSetId>,
        workspace_snapshot_address: WorkspaceSnapshotAddress,
    ) -> ChangeSetResult<Self> {
        let id: Ulid = Ulid::new();
        let change_set_id: ChangeSetId = id.into();

        let workspace_snapshot = WorkspaceSnapshot::find(ctx, workspace_snapshot_address)
            .await
            .map_err(Box::new)?;
        // The workspace snapshot needs to be marked as seen by this new
        // changeset, so that edit sessions are able to know what is net new in
        // the edit session vs what the changeset already contained. The "onto"
        // changeset needs to have seen the "to_rebase" or we will treat them as
        // completely disjoint changesets.
        let workspace_snapshot_address = workspace_snapshot.write(ctx).await.map_err(Box::new)?;

        let workspace_id = ctx.tenancy().workspace_pk_opt();
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO change_set_pointers (id, name, base_change_set_id, status, workspace_id, workspace_snapshot_address) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                &[&change_set_id, &name, &base_change_set_id, &ChangeSetStatus::Open.to_string(), &workspace_id, &workspace_snapshot_address],
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
            .workspace_pk_opt()
            .ok_or(ChangeSetError::NoTenancySet)?;

        let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
            .await?
            .ok_or(ChangeSetError::WorkspaceNotFound(workspace_pk))?;

        let base_change_set = ChangeSet::find(ctx, workspace.default_change_set_id())
            .await?
            .ok_or(ChangeSetError::DefaultChangeSetNotFound(
                workspace.default_change_set_id(),
            ))?;

        let change_set = ChangeSet::new(
            ctx,
            name,
            Some(workspace.default_change_set_id()),
            base_change_set.workspace_snapshot_address,
        )
        .await?;

        Ok(change_set)
    }

    // pub fn generate_ulid(&self) -> ChangeSetResult<Ulid> {
    //     self.generator
    //         .lock()
    //         .map_err(|e| ChangeSetError::Mutex(e.to_string()))?
    //         .generate()
    //         .map(Into::into)
    //         .map_err(Into::into)
    // }

    pub async fn update_workspace_id(
        &mut self,
        ctx: &DalContext,
        workspace_id: WorkspacePk,
    ) -> ChangeSetResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET workspace_id = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &workspace_id],
            )
            .await?;

        self.workspace_id = Some(workspace_id);

        Ok(())
    }

    fn workspace_id(&self) -> ChangeSetResult<WorkspacePk> {
        self.workspace_id.ok_or(ChangeSetError::NoTenancySet)
    }

    async fn workspace(&self, ctx: &DalContext) -> ChangeSetResult<Workspace> {
        Ok(Workspace::get_by_pk_or_error(ctx, self.workspace_id()?).await?)
    }

    async fn is_head(&self, ctx: &DalContext) -> ChangeSetResult<bool> {
        Ok(self.workspace(ctx).await?.default_change_set_id() == self.id)
    }

    async fn publish_billing_workspace_change_event(
        &self,
        ctx: &DalContext,
        change_description: impl Into<String>,
        modify_event: impl FnOnce(&mut BillingWorkspaceChangeEvent),
    ) -> ChangeSetResult<()> {
        if self.workspace_id.is_some() && self.is_head(ctx).await? {
            let workspace = self.workspace_id()?;
            let mut event = BillingWorkspaceChangeEvent {
                workspace: workspace.into(),
                workspace_snapshot_address: self.workspace_snapshot_address,
                // TODO(nick,jkeiser): see the "TODO" on the struct definition for why we string convert here.
                status: self.status.to_string(),
                resource_count: 0,
                change_set_id: self.id.into(),
                merge_requested_by_user_id: self.merge_requested_by_user_id.map(Into::into),
                change_description: change_description.into(),
                // TODO(nick): we need to ensure that this is correct. This will likely come from
                // sdf and/or the billing events server if it runs in a separate binary.
                location: BillingWorkspaceChangeEventLocation::Local,
            };
            modify_event(&mut event);
            // Ensure queue is created
            ctx.services_context()
                .nats_streams()
                .billing_events()
                .publish_workspace_update(&workspace.to_string(), &event)
                .await?;
        }
        Ok(())
    }

    pub async fn update_pointer(
        &mut self,
        ctx: &DalContext,
        workspace_snapshot_address: WorkspaceSnapshotAddress,
    ) -> ChangeSetResult<()> {
        self.publish_billing_workspace_change_event(ctx, "update_pointer", |event| {
            event.workspace_snapshot_address = workspace_snapshot_address;
        })
        .await?;

        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET workspace_snapshot_address = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &workspace_snapshot_address],
            )
            .await?;

        self.workspace_snapshot_address = workspace_snapshot_address;

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        ctx: &DalContext,
        status: ChangeSetStatus,
    ) -> ChangeSetResult<()> {
        self.publish_billing_workspace_change_event(ctx, "update_pointer", |event| {
            // TODO(nick,jkeiser): see the "TODO" on the struct definition for why we string convert here.
            event.status = status.to_string();
        })
        .await?;

        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET status = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &status.to_string()],
            )
            .await?;

        self.status = status;

        Ok(())
    }

    pub async fn update_merge_requested_by_user_id(
        &mut self,
        ctx: &DalContext,
        user_pk: UserPk,
    ) -> ChangeSetResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET merge_requested_by_user_id = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &user_pk],
            )
            .await?;

        self.merge_requested_by_user_id = Some(user_pk);

        Ok(())
    }

    #[instrument(
        name = "change_set.find",
        level = "debug",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = Empty,
        ),
    )]
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
            Some(row) => {
                let span = Span::current();

                let change_set = Self::try_from(row)?;

                if let Some(workspace_id) = change_set.workspace_id {
                    span.record("si.workspace.id", workspace_id.to_string());
                }
                Ok(Some(change_set))
            }
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
                "SELECT * from change_set_pointers WHERE workspace_id = $1 AND status IN ($2, $3, $4)",
                &[
                    &ctx.tenancy().workspace_pk_opt(),
                    &ChangeSetStatus::Open.to_string(),
                    &ChangeSetStatus::NeedsApproval.to_string(),
                    &ChangeSetStatus::NeedsAbandonApproval.to_string(),
                ],
            )
            .await?;

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    /// List all change sets for a given workspace
    pub async fn list_all_for_workspace(
        ctx: &DalContext,
        workspace_pk: WorkspacePk,
    ) -> ChangeSetResult<Vec<Self>> {
        let mut result = vec![];

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE workspace_id = $1",
                &[&workspace_pk],
            )
            .await?;

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    /// Take care when working on these change sets to set the workspace id on the dal context!!!
    pub async fn list_open_for_all_workspaces(ctx: &DalContext) -> ChangeSetResult<Vec<Self>> {
        let mut result = vec![];
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE status IN ($1, $2, $3)",
                &[
                    &ChangeSetStatus::Open.to_string(),
                    &ChangeSetStatus::NeedsApproval.to_string(),
                    &ChangeSetStatus::NeedsAbandonApproval.to_string(),
                ],
            )
            .await?;

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    /// Applies the current [`ChangeSet`] in the provided [`DalContext`]. [`Actions`](Action)
    /// are enqueued as needed and only done so if the base [`ChangeSet`] is "HEAD" (i.e.
    /// the default [`ChangeSet`] of the [`Workspace`]).
    #[instrument(level = "info", skip_all)]
    pub async fn apply_to_base_change_set(ctx: &mut DalContext) -> ChangeSetApplyResult<ChangeSet> {
        // Apply to the base change with the current change set (non-editing) and commit.
        let mut change_set_to_be_applied = Self::find(ctx, ctx.change_set_id())
            .await?
            .ok_or(ChangeSetApplyError::ChangeSetNotFound(ctx.change_set_id()))?;
        ctx.update_visibility_and_snapshot_to_visibility(ctx.change_set_id())
            .await?;
        change_set_to_be_applied
            .apply_to_base_change_set_inner(ctx)
            .await?;

        // This is just to send the ws events
        ctx.blocking_commit_no_rebase().await?;

        Ok(change_set_to_be_applied)
    }

    pub async fn detect_updates_that_will_be_applied(
        &self,
        ctx: &DalContext,
    ) -> ChangeSetResult<Option<RebaseBatch>> {
        let base_change_set_id = self
            .base_change_set_id
            .ok_or(ChangeSetError::NoBaseChangeSet(self.id))?;

        let base_snapshot = Arc::new(
            WorkspaceSnapshot::find_for_change_set(ctx, base_change_set_id)
                .await
                .map_err(Box::new)?,
        );

        Ok(WorkspaceSnapshot::calculate_rebase_batch(
            base_snapshot,
            ctx.workspace_snapshot().map_err(Box::new)?,
        )
        .await
        .map_err(Box::new)?)
    }

    /// Applies the current [`ChangeSet`] in the provided [`DalContext`] to its base
    /// [`ChangeSet`]. This involves performing a rebase request and updating the status
    /// of the [`ChangeSet`] accordingly.
    ///
    /// This function neither changes the visibility nor the snapshot after performing the
    /// aforementioned actions.
    async fn apply_to_base_change_set_inner(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let base_change_set_id = self
            .base_change_set_id
            .ok_or(ChangeSetError::NoBaseChangeSet(self.id))?;

        if let Some(rebase_batch) = self.detect_updates_that_will_be_applied(ctx).await? {
            let rebase_batch_address = ctx.write_rebase_batch(rebase_batch).await?;

            let rebase_request = RebaseRequest::new(base_change_set_id, rebase_batch_address);
            ctx.do_rebase_request(rebase_request).await?;
        }

        self.update_status(ctx, ChangeSetStatus::Applied).await?;
        let user = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_applied(ctx, self.id, base_change_set_id, user)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }

    /// Returns a new [`ChangeSetId`](ChangeSet) if a new [`ChangeSet`] was created.
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

    pub async fn merge_vote(&mut self, ctx: &DalContext, vote: String) -> ChangeSetResult<()> {
        let user_id = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_merge_vote(ctx, self.id, user_id, vote)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }
    pub async fn abandon_vote(&mut self, ctx: &DalContext, vote: String) -> ChangeSetResult<()> {
        let user_id = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_abandon_vote(ctx, self.id, user_id, vote)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }

    pub async fn cancel_abandon_approval_flow(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        self.update_status(ctx, ChangeSetStatus::Open).await?;
        let user_id = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_cancel_abandon_approval_process(ctx, self.id, user_id)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }
    pub async fn begin_abandon_approval_flow(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        self.update_status(ctx, ChangeSetStatus::NeedsAbandonApproval)
            .await?;
        let user_id = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_begin_abandon_approval_process(ctx, self.id, user_id)
            .await?
            .publish_on_commit(ctx)
            .await?;
        WsEvent::change_set_abandon_vote(
            ctx,
            ctx.visibility().change_set_id,
            user_id,
            "Approve".to_string(),
        )
        .await?
        .publish_on_commit(ctx)
        .await?;
        Ok(())
    }

    pub async fn begin_approval_flow(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        self.update_status(ctx, ChangeSetStatus::NeedsApproval)
            .await?;
        let user_id = Self::extract_userid_from_context(ctx).await;
        if let Some(user_pk) = user_id {
            self.update_merge_requested_by_user_id(ctx, user_pk).await?;
        }
        WsEvent::change_set_begin_approval_process(ctx, self.id, user_id)
            .await?
            .publish_on_commit(ctx)
            .await?;
        WsEvent::change_set_merge_vote(
            ctx,
            ctx.visibility().change_set_id,
            user_id,
            "Approve".to_string(),
        )
        .await?
        .publish_on_commit(ctx)
        .await?;
        Ok(())
    }

    pub async fn cancel_approval_flow(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        self.update_status(ctx, ChangeSetStatus::Open).await?;
        let user_id = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_cancel_approval_process(ctx, self.id, user_id)
            .await?
            .publish_on_commit(ctx)
            .await?;
        Ok(())
    }

    pub async fn abandon(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        self.update_status(ctx, ChangeSetStatus::Abandoned).await?;
        let user_id = Self::extract_userid_from_context(ctx).await;
        WsEvent::change_set_abandoned(ctx, self.id, user_id)
            .await?
            .publish_on_commit(ctx)
            .await?;
        Ok(())
    }

    pub async fn extract_userid_from_context(ctx: &DalContext) -> Option<UserPk> {
        let user_id = match ctx.history_actor() {
            HistoryActor::User(user_pk) => {
                let maybe_user = User::get_by_pk(ctx, *user_pk).await;
                match maybe_user {
                    Ok(user_option) => user_option.map(|user| user.pk()),
                    Err(_) => None,
                }
            }
            HistoryActor::SystemInit => None,
        };
        user_id
    }

    #[instrument(
        name = "change_set.workspace_snapshot_in_use",
        level = "debug",
        skip_all,
        fields(
            si.workspace_snapshot_address = %workspace_snapshot_address,
            si.workspace.id = Empty,
        ),
    )]
    pub async fn workspace_snapshot_address_in_use(
        ctx: &DalContext,
        workspace_snapshot_address: &WorkspaceSnapshotAddress,
    ) -> ChangeSetResult<bool> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT count(id) AS count FROM change_set_pointers WHERE workspace_snapshot_address = $1",
                &[&workspace_snapshot_address],
            )
            .await?;

        let count: i64 = row.get("count");
        if count > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Walk the graph of change sets up to the change set that has no "base
    /// change set id" and return the set.
    pub async fn ancestors(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> ChangeSetResult<HashSet<ChangeSetId>> {
        let mut result = HashSet::new();
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(FIND_ANCESTORS_QUERY, &[&change_set_id])
            .await?;

        for row in rows {
            let id: String = row.get("id");
            result.insert(ChangeSetId::from_str(&id)?);
        }

        Ok(result)
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
                "workspace_snapshot_address",
                &self.workspace_snapshot_address.to_string(),
            )
            .field(
                "merge_requested_by_user_id",
                &self
                    .merge_requested_by_user_id
                    .map(|user_pk| user_pk.to_string()),
            )
            .finish()
    }
}
