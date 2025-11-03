use std::{
    collections::HashSet,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::{
    PgError,
    PgRow,
};
use si_db::{
    HistoryActor,
    HistoryEvent,
    User,
    change_set::FIND_ANCESTORS_QUERY,
};
use si_events::{
    RebaseBatchAddressKind,
    WorkspaceSnapshotAddress,
    audit_log::AuditLogKind,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
    workspace_snapshot::Checksum,
};
use si_id::{
    ActionId,
    EntityId,
    UserPk,
    WorkspacePk,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time;

use crate::{
    ChangeSetStatus,
    ComponentError,
    DalContext,
    Func,
    FuncError,
    Schema,
    SchemaError,
    SchemaVariant,
    SchemaVariantError,
    TransactionsError,
    Workspace,
    WorkspaceError,
    WorkspaceSnapshot,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    action::ActionError,
    billing_publish::{
        self,
        BillingPublishError,
    },
    slow_rt::SlowRuntimeError,
    workspace_snapshot::{
        DependentValueRoot,
        dependent_value_root::DependentValueRootError,
        graph::RebaseBatch,
        selector::WorkspaceSnapshotSelectorDiscriminants,
        split_snapshot::{
            SplitRebaseBatchVCurrent,
            SplitSnapshot,
        },
    },
};

pub mod approval;
pub mod event;
pub mod status;
pub mod view;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error("billing publish error: {0}")]
    BillingPublish(#[from] Box<BillingPublishError>),
    #[error("cannot rename HEAD change set")]
    CantRenameHeadChangeSet,
    #[error("change set not approved for apply. Current state: {0}")]
    ChangeSetNotApprovedForApply(ChangeSetStatus),
    #[error("change set with id {0} not found")]
    ChangeSetNotFound(ChangeSetId),
    #[error("default change set {0} has no workspace snapshot pointer")]
    DefaultChangeSetNoWorkspaceSnapshotPointer(ChangeSetId),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("dvu roots are not empty for change set: {0}")]
    DvuRootsNotEmpty(ChangeSetId),
    #[error("enum parse error: {0}")]
    EnumParse(#[from] strum::ParseError),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("invalid user actor pk")]
    InvalidActor(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] Box<LayerDbError>),
    #[error("ulid monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("mutex error: {0}")]
    Mutex(String),
    #[error("Changeset {0} does not have a base change set")]
    NoBaseChangeSet(ChangeSetId),
    #[error("no tenancy set in context")]
    NoTenancySet,
    #[error("no workspace_pk is set for change_set_id={0}")]
    NoWorkspacePkSet(ChangeSetId),
    #[error("Changeset {0} does not have a workspace snapshot")]
    NoWorkspaceSnapshot(ChangeSetId),
    #[error("pg error: {0}")]
    Pg(#[from] Box<PgError>),
    #[error("rebaser client error: {0}")]
    RebaserClient(#[from] Box<rebaser_client::ClientError>),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] Box<si_db::Error>),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("timeout out waiting for dvu after {0}ms")]
    TimedOutAwaitingDvu(u64),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error(
        "found an unexpected number of open change sets matching default change set (should be one, found {0:?})"
    )]
    UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(Vec<ChangeSetId>),
    #[error("workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<LayerDbError> for ChangeSetError {
    fn from(value: LayerDbError) -> Self {
        Box::new(value).into()
    }
}

impl From<si_db::Error> for ChangeSetError {
    fn from(value: si_db::Error) -> Self {
        Box::new(value).into()
    }
}

impl From<PgError> for ChangeSetError {
    fn from(value: PgError) -> Self {
        Box::new(value).into()
    }
}

impl From<rebaser_client::ClientError> for ChangeSetError {
    fn from(value: rebaser_client::ClientError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for ChangeSetError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceError> for ChangeSetError {
    fn from(value: WorkspaceError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for ChangeSetError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

impl From<ActionError> for ChangeSetApplyError {
    fn from(value: ActionError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for ChangeSetApplyError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for ChangeSetApplyError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<si_db::Error> for ChangeSetApplyError {
    fn from(value: si_db::Error) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for ChangeSetApplyError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

/// The primary result type for this module.
pub type ChangeSetResult<T> = Result<T, ChangeSetError>;

/// A superset of [`ChangeSetError`] used when performing apply logic.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetApplyError {
    #[error("action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("action prototype not found for id: {0}")]
    ActionPrototypeNotFound(ActionId),
    #[error("Cannot apply changeset {0} to itself")]
    CannotApplyToItself(ChangeSetId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("invalid user: {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("change set ({0}) does not have a base change set")]
    NoBaseChangeSet(ChangeSetId),
    #[error("si db error: {0}")]
    SiDb(#[from] Box<si_db::Error>),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
}

/// A superset of [`ChangeSetResult`] used when performing apply logic.
pub type ChangeSetApplyResult<T> = Result<T, ChangeSetApplyError>;

pub use si_id::ChangeSetId;

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
    pub merge_requested_at: Option<DateTime<Utc>>,
    pub reviewed_by_user_id: Option<UserPk>,
    pub reviewed_at: Option<DateTime<Utc>>,
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
            merge_requested_at: value.try_get("merge_requested_at")?,
            reviewed_by_user_id: value.try_get("reviewed_by_user_id")?,
            reviewed_at: value.try_get("reviewed_at")?,
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
        let workspace_pk = ctx.workspace_pk()?;

        let workspace = Workspace::get_by_pk(ctx, workspace_pk).await?;

        let head = workspace.default_change_set(ctx).await?;

        let change_set =
            ChangeSet::new(ctx, name, Some(head.id), head.workspace_snapshot_address).await?;

        Ok(change_set)
    }

    pub async fn into_frontend_type(
        &self,
        ctx: &DalContext,
    ) -> ChangeSetResult<si_frontend_types::ChangeSet> {
        let merge_requested_by_user =
            if let Some(merge_requested_by) = self.merge_requested_by_user_id {
                User::get_by_pk_opt(ctx, merge_requested_by)
                    .await?
                    .map(|user| {
                        if user.name().is_empty() {
                            user.email().clone()
                        } else {
                            user.name().clone()
                        }
                    })
            } else {
                None
            };

        let reviewed_by_user = if let Some(reviewed_by) = self.reviewed_by_user_id {
            User::get_by_pk_opt(ctx, reviewed_by).await?.map(|user| {
                if user.name().is_empty() {
                    user.email().clone()
                } else {
                    user.name().clone()
                }
            })
        } else {
            None
        };

        let change_set = si_frontend_types::ChangeSet {
            created_at: self.created_at,
            id: self.id,
            updated_at: self.updated_at,
            name: self.name.clone(),
            status: self.status.into(),
            base_change_set_id: self.base_change_set_id,
            workspace_id: self.workspace_id.map_or("".to_owned(), |id| id.to_string()),
            merge_requested_by_user_id: self.merge_requested_by_user_id.map(|s| s.to_string()),
            merge_requested_by_user,
            merge_requested_at: self.merge_requested_at,
            reviewed_by_user_id: self.reviewed_by_user_id.map(|id| id.into()),
            reviewed_by_user,
            reviewed_at: self.reviewed_at,
        };

        Ok(change_set)
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
                "UPDATE change_set_pointers SET workspace_id = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &workspace_id],
            )
            .await?;

        self.workspace_id = Some(workspace_id);

        Ok(())
    }

    pub fn workspace_id(&self) -> ChangeSetResult<WorkspacePk> {
        self.workspace_id.ok_or(ChangeSetError::NoTenancySet)
    }

    async fn workspace(&self, ctx: &DalContext) -> ChangeSetResult<Workspace> {
        Ok(Workspace::get_by_pk(ctx, self.workspace_id()?).await?)
    }

    pub async fn is_head(&self, ctx: &DalContext) -> ChangeSetResult<bool> {
        Ok(self.workspace(ctx).await?.default_change_set_id() == self.id)
    }

    #[instrument(name = "change_set.update_pointer", level = "debug", skip_all)]
    pub async fn update_pointer(
        &mut self,
        ctx: &DalContext,
        workspace_snapshot_address: WorkspaceSnapshotAddress,
    ) -> ChangeSetResult<()> {
        let old_snapshot_address = self.workspace_snapshot_address;

        // Update change set pointer
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET workspace_snapshot_address = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &workspace_snapshot_address],
            )
            .await?;

        // Record when old snapshot was last used
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "INSERT INTO snapshot_last_used (snapshot_id, last_used_at, created_at)
                 VALUES ($1, CLOCK_TIMESTAMP(), CLOCK_TIMESTAMP())
                 ON CONFLICT (snapshot_id)
                 DO UPDATE SET last_used_at = CLOCK_TIMESTAMP()",
                &[&old_snapshot_address.to_string()],
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
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET status = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &status.to_string()],
            )
            .await?;

        self.status = status;
        billing_publish::for_change_set_status_update(ctx, self)
            .await
            .map_err(Box::new)?;
        Ok(())
    }

    pub async fn request_change_set_approval(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let user_pk = Self::extract_userid_from_context_or_error(ctx).await?;
        let status = ChangeSetStatus::NeedsApproval;
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET merge_requested_by_user_id = $2, merge_requested_at = CLOCK_TIMESTAMP(), status = $3, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &user_pk, &status.to_string()],
            )
            .await?;

        self.status = status;

        Ok(())
    }

    /// Set the status to Open, and clear any reviewed/merge requested info
    pub async fn reopen_change_set(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let status = ChangeSetStatus::Open;
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers
                SET reviewed_by_user_id = NULL,
                reviewed_at = NULL,
                merge_requested_by_user_id = NULL,
                merge_requested_at = NULL,
                status = $2,
                updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &status.to_string()],
            )
            .await?;

        self.status = status;

        Ok(())
    }

    /// First, transitions the status of the [`ChangeSet`] to [`ChangeSetStatus::NeedsApproval`]
    /// then [`ChangeSetStatus::Approved`]. Next, checks if DVU Roots still exist. Finally,
    /// lock every [`SchemaVariant`] and [`Func`] that is currently unlocked
    pub async fn prepare_for_force_apply(ctx: &DalContext) -> ChangeSetResult<()> {
        // first change the status to approved and who did it
        let mut change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;

        change_set.request_change_set_approval(ctx).await?;
        // then approve it
        change_set.approve_change_set_for_apply(ctx).await?;
        // then do the rest
        Self::prepare_for_apply(ctx).await
    }

    /// First, checks if DVU Roots still exist. Next, ensures the [`ChangeSet`] has an
    /// [`ChangeSetStatus::Approved`]. Finally,
    /// lock every [`SchemaVariant`] and [`Func`] that is currently unlocked
    pub async fn prepare_for_apply(ctx: &DalContext) -> ChangeSetResult<()> {
        Self::prepare_for_apply_inner(ctx, false).await
    }

    /// This is a copy of [Self::prepare_for_apply], but skips the status check. This is because
    /// sdf now handles the approvals flow as part of the fine grained access control work (i.e.
    /// SpiceDB is intentionally not accessible in the DAL).
    pub async fn prepare_for_apply_without_status_check(ctx: &DalContext) -> ChangeSetResult<()> {
        Self::prepare_for_apply_inner(ctx, true).await
    }

    // TODO(nick): now that the fine grained access control flag is gone, we can collapse the two
    // outer methods and chase down tests continuing to use the old method.
    async fn prepare_for_apply_inner(
        ctx: &DalContext,
        dangerous_skip_status_check: bool,
    ) -> ChangeSetResult<()> {
        // Ensure that DVU roots are empty before continuing.
        if DependentValueRoot::roots_exist(ctx).await? {
            // TODO(nick): we should consider requiring this check in integration tests too. Why did I
            // not do this at the time of writing? Tests have multiple ways to call "apply", whether
            // its via helpers or through the change set methods directly. In addition, they test
            // for success and failure, not solely for success. We should still do this, but not within
            // the PR corresponding to when this message was written.
            return Err(ChangeSetError::DvuRootsNotEmpty(ctx.change_set_id()));
        }

        // WARNING(nick): we should only skip this status check if using sdf's protected apply logic.
        if !dangerous_skip_status_check {
            // if the change set status isn't approved, we shouldn't go
            // locking stuff
            let change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;
            if change_set.status != ChangeSetStatus::Approved {
                return Err(ChangeSetError::ChangeSetNotApprovedForApply(
                    change_set.status,
                ));
            }
        }

        // Lock all unlocked variants
        for schema_id in Schema::list_ids(ctx).await.map_err(Box::new)? {
            let schema = Schema::get_by_id(ctx, schema_id).await.map_err(Box::new)?;
            let Some(variant) = SchemaVariant::get_unlocked_for_schema(ctx, schema_id)
                .await
                .map_err(Box::new)?
            else {
                continue;
            };

            let variant_id = variant.id();

            variant.lock(ctx).await.map_err(Box::new)?;
            schema
                .set_default_variant_id(ctx, variant_id)
                .await
                .map_err(Box::new)?;
        }
        // Lock all unlocked functions too
        for func in Func::list_for_default_and_editing(ctx)
            .await
            .map_err(Box::new)?
        {
            if !func.is_locked {
                func.lock(ctx).await.map_err(Box::new)?;
            }
        }
        Ok(())
    }

    pub async fn approve_change_set_for_apply(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let user_pk = Self::extract_userid_from_context_or_error(ctx).await?;
        let status = ChangeSetStatus::Approved;
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET reviewed_by_user_id = $2, reviewed_at = CLOCK_TIMESTAMP(), status = $3, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &user_pk, &status.to_string()],
            )
            .await?;

        self.status = status;

        Ok(())
    }

    pub async fn reject_change_set_for_apply(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let user_pk = Self::extract_userid_from_context_or_error(ctx).await?;
        let status = ChangeSetStatus::Rejected;
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET reviewed_by_user_id = $2, reviewed_at = CLOCK_TIMESTAMP(), status = $3, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&self.id, &user_pk, &status.to_string()],
            )
            .await?;

        self.status = status;

        Ok(())
    }

    /// Finds a [`ChangeSet`] across all workspaces, ignoring the provided [`WorkspacePk`] on the
    /// current [`DalContext`]
    pub async fn get_by_id_across_workspaces(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> ChangeSetResult<Self> {
        Self::find_across_workspaces(ctx, change_set_id)
            .await?
            .ok_or_else(|| ChangeSetError::ChangeSetNotFound(change_set_id))
    }

    /// Finds a [`ChangeSet`] across all workspaces, ignoring the provided [`WorkspacePk`] on the
    /// current [`DalContext`]
    #[instrument(
        name = "change_set.find_across_workspaces",
        level = "debug",
        skip_all,
        fields(
            si.change_set.id = %change_set_id,
            si.workspace.id = Empty,
        ),
    )]
    pub async fn find_across_workspaces(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> ChangeSetResult<Option<Self>> {
        let span = current_span_for_instrument_at!("debug");

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
                let change_set = Self::try_from(row)?;

                if let Some(workspace_id) = change_set.workspace_id {
                    span.record("si.workspace.id", workspace_id.to_string());
                }
                Ok(Some(change_set))
            }
            None => Ok(None),
        }
    }

    /// Get a change set within the [`WorkspacePk`] set for the current [`DalContext`]
    pub async fn get_by_id(ctx: &DalContext, change_set_id: ChangeSetId) -> ChangeSetResult<Self> {
        Self::find(ctx, change_set_id)
            .await?
            .ok_or_else(|| ChangeSetError::ChangeSetNotFound(change_set_id))
    }

    /// Find a change set within the [`WorkspacePk`] set for the current [`DalContext`]
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
        let span = current_span_for_instrument_at!("debug");
        let workspace_id = ctx.workspace_pk()?;
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT * FROM change_set_pointers WHERE id = $1 AND workspace_id = $2",
                &[&change_set_id, &workspace_id],
            )
            .await?;

        match row {
            Some(row) => {
                let change_set = Self::try_from(row)?;

                if let Some(workspace_id) = change_set.workspace_id {
                    span.record("si.workspace.id", workspace_id.to_string());
                }
                Ok(Some(change_set))
            }
            None => {
                // warn here so we can see if something is requesting change sets cross workspace
                warn!(
                    si.workspace.id = %workspace_id,
                    "Change Set Id: {change_set_id} not found for Workspace: {workspace_id}",
                );
                Ok(None)
            }
        }
    }

    pub async fn list_active(ctx: &DalContext) -> ChangeSetResult<Vec<Self>> {
        let mut result = vec![];
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE workspace_id = $1 AND status IN ($2, $3, $4, $5, $6)",
                &[
                    &ctx.tenancy().workspace_pk_opt(),
                    &ChangeSetStatus::Open.to_string(),
                    &ChangeSetStatus::NeedsApproval.to_string(),
                    &ChangeSetStatus::NeedsAbandonApproval.to_string(),
                    &ChangeSetStatus::Approved.to_string(),
                    &ChangeSetStatus::Rejected.to_string(),
                ],
            )
            .await?;

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    /// List all change sets that are applied.
    pub async fn list_all_applied(
        ctx: &DalContext,
        workspace_pk: WorkspacePk,
    ) -> ChangeSetResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE workspace_id = $1 AND status = $2",
                &[&workspace_pk, &ChangeSetStatus::Applied.to_string()],
            )
            .await?;

        let mut result = Vec::with_capacity(rows.len());
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

    pub async fn list_active_for_workspace(
        ctx: &DalContext,
        workspace_pk: WorkspacePk,
    ) -> ChangeSetResult<Vec<Self>> {
        let mut result = vec![];

        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE workspace_id = $1 AND status IN ($2, $3, $4, $5, $6)",
                &[
                    &workspace_pk,
                    &ChangeSetStatus::Open.to_string(),
                    &ChangeSetStatus::NeedsApproval.to_string(),
                    &ChangeSetStatus::NeedsAbandonApproval.to_string(),
                    &ChangeSetStatus::Approved.to_string(),
                    &ChangeSetStatus::Rejected.to_string(),
                ],
            )
            .await?;

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    /// Take care when working on these change sets to set the workspace id on the dal context!!!
    pub async fn list_active_for_all_workspaces(ctx: &DalContext) -> ChangeSetResult<Vec<Self>> {
        let mut result = vec![];
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(
                "SELECT * from change_set_pointers WHERE status IN ($1, $2, $3, 4, 5)",
                &[
                    &ChangeSetStatus::Open.to_string(),
                    &ChangeSetStatus::NeedsApproval.to_string(),
                    &ChangeSetStatus::NeedsAbandonApproval.to_string(),
                    &ChangeSetStatus::Approved.to_string(),
                    &ChangeSetStatus::Rejected.to_string(),
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
    /// Also sends the relevant WSEvent
    #[instrument(level = "info", skip_all)]
    pub async fn apply_to_base_change_set(ctx: &mut DalContext) -> ChangeSetApplyResult<ChangeSet> {
        let base_change_set_id = ctx.get_workspace_default_change_set_id().await?;

        if ctx.change_set_id() == base_change_set_id {
            return Err(ChangeSetApplyError::CannotApplyToItself(base_change_set_id));
        }

        // Apply to the base change with the current change set (non-editing) and commit.
        let mut change_set_to_be_applied = Self::get_by_id(ctx, ctx.change_set_id()).await?;

        ctx.update_visibility_and_snapshot_to_visibility(ctx.change_set_id())
            .await?;
        change_set_to_be_applied
            .apply_to_base_change_set_inner(ctx)
            .await?;

        // This is just to send the ws events
        ctx.blocking_commit_no_rebase().await?;

        Ok(change_set_to_be_applied)
    }

    #[instrument(
        level = "info",
        name = "change_set.detect_updates_that_will_be_applied_split",
        skip_all
    )]
    pub async fn detect_updates_that_will_be_applied_split(
        &self,
        ctx: &DalContext,
    ) -> ChangeSetResult<Option<SplitRebaseBatchVCurrent>> {
        let base_change_set_id = self
            .base_change_set_id
            .ok_or(ChangeSetError::NoBaseChangeSet(self.id))?;

        let base_snapshot = Arc::new(
            SplitSnapshot::find_for_change_set(ctx, base_change_set_id)
                .await
                .map_err(Box::new)?,
        );

        Ok(SplitSnapshot::calculate_rebase_batch(
            base_snapshot,
            ctx.workspace_snapshot()
                .map_err(Box::new)?
                .as_split_snapshot()
                .map_err(Box::new)?,
        )
        .await
        .map_err(Box::new)?)
    }

    #[instrument(
        level = "info",
        name = "change_set.detect_updates_that_will_be_applied_legacy",
        skip_all
    )]
    pub async fn detect_updates_that_will_be_applied_legacy(
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
            ctx.workspace_snapshot()
                .map_err(Box::new)?
                .as_legacy_snapshot()
                .map_err(Box::new)?,
        )
        .await
        .map_err(Box::new)?)
    }

    /// Applies the current [`ChangeSet`] in the provided [`DalContext`] to its base
    /// [`ChangeSet`]. This involves performing a rebase request, updating the status
    /// of the [`ChangeSet`] accordingly, and publishing a WSEvent
    ///
    /// This function neither changes the visibility nor the snapshot after performing the
    /// aforementioned actions.
    async fn apply_to_base_change_set_inner(&mut self, ctx: &DalContext) -> ChangeSetResult<()> {
        let workspace_id = self
            .workspace_id
            .ok_or(ChangeSetError::NoWorkspacePkSet(self.id))?;
        let base_change_set_id = self
            .base_change_set_id
            .ok_or(ChangeSetError::NoBaseChangeSet(self.id))?;

        let snapshot_kind: WorkspaceSnapshotSelectorDiscriminants =
            ctx.workspace_snapshot().map_err(Box::new)?.into();

        let maybe_rebase_batch_address = match snapshot_kind {
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
                if let Some(rebase_batch) =
                    self.detect_updates_that_will_be_applied_legacy(ctx).await?
                {
                    Some(RebaseBatchAddressKind::Legacy(
                        ctx.write_legacy_rebase_batch(rebase_batch).await?,
                    ))
                } else {
                    None
                }
            }
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
                if let Some(rebase_batch) =
                    self.detect_updates_that_will_be_applied_split(ctx).await?
                {
                    Some(RebaseBatchAddressKind::Split(
                        ctx.write_split_snapshot_rebase_batch(rebase_batch).await?,
                    ))
                } else {
                    None
                }
            }
        };

        if let Some(rebase_batch_address) = maybe_rebase_batch_address {
            let (request_id, reply_fut) = ctx
                .run_rebase_from_change_set_with_reply(
                    workspace_id,
                    base_change_set_id,
                    rebase_batch_address,
                    self.id,
                )
                .await?;

            let reply_fut = reply_fut.instrument(info_span!(
                "rebaser_client.await_response",
                si.workspace.id = %workspace_id,
                si.change_set.id = %base_change_set_id,
            ));

            // Wait on response from Rebaser after request has processed
            let timeout = Duration::from_secs(60);
            let _reply = time::timeout(timeout, reply_fut)
                .await
                .map_err(|_elapsed| {
                    TransactionsError::RebaserReplyDeadlineElasped(timeout, request_id)
                })??;
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
    /// Also writes an audit log event to head (so we don't have to handle this in every route handler)
    pub async fn force_new(ctx: &mut DalContext) -> ChangeSetResult<Option<ChangeSetId>> {
        let maybe_fake_pk = if ctx.change_set_id()
            == ctx.get_workspace_default_change_set_id().await?
        {
            let change_set = Self::fork_head(ctx, Self::generate_name()).await?;
            ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
                .await?;
            ctx.write_audit_log_to_head(AuditLogKind::CreateChangeSet, change_set.name)
                .await?;
            WsEvent::change_set_created(ctx, change_set.id, change_set.workspace_snapshot_address)
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

    /// Updates the status for a ChangeSet to be [`ChangeSetStatus::Abandoned`] and fires necessary WSEvent
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
        match ctx.history_actor() {
            HistoryActor::User(user_pk) => {
                let maybe_user = User::get_by_pk_opt(ctx, *user_pk).await;
                match maybe_user {
                    Ok(user_option) => user_option.map(|user| user.pk()),
                    Err(_) => None,
                }
            }
            HistoryActor::SystemInit => None,
        }
    }

    pub async fn extract_userid_from_context_or_error(ctx: &DalContext) -> ChangeSetResult<UserPk> {
        let user_id = match ctx.history_actor() {
            HistoryActor::User(user_pk) => User::get_by_pk(ctx, *user_pk).await?.pk(),
            HistoryActor::SystemInit => return Err(ChangeSetError::InvalidUserSystemInit),
        };
        Ok(user_id)
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

    pub async fn rename_change_set(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        new_name: &String,
    ) -> ChangeSetResult<()> {
        let default_change_set_id = ctx.get_workspace_default_change_set_id().await?;
        if default_change_set_id == change_set_id {
            return Err(ChangeSetError::CantRenameHeadChangeSet);
        }

        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET name = $2, updated_at = CLOCK_TIMESTAMP() WHERE id = $1",
                &[&change_set_id, new_name],
            )
            .await?;
        WsEvent::rename_change_set(ctx, change_set_id, new_name.clone())
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }

    /// Wait for the changeset's DVUs to be completely processed before continuing
    /// This func also runs update_snapshot_to_visibility on the passed in ctx
    /// with_timeout makes the function fail after a minute, since that's also the rebaser timeout.
    /// On prod code, always pass `true`. `false` is for testing only.
    pub async fn wait_for_dvu(ctx: &mut DalContext, with_timeout: bool) -> ChangeSetResult<()> {
        let mut retry_count = 0;
        const AWAIT_MS: u64 = 25;
        const MAX_RETRIES: u64 = (60 * 1000) / AWAIT_MS;

        loop {
            ctx.update_snapshot_to_visibility().await?;
            if !DependentValueRoot::roots_exist(ctx).await? {
                break;
            }
            tokio::time::sleep(Duration::from_millis(AWAIT_MS)).await;

            if with_timeout {
                retry_count += 1;
                if retry_count > MAX_RETRIES {
                    return Err(ChangeSetError::TimedOutAwaitingDvu(AWAIT_MS * retry_count));
                }
            }
        }

        Ok(())
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

/// Calculates the checksum based on a list of IDs with hashes passed in.
#[instrument(name = "calculate_checksum", level = "debug", skip_all)]
pub async fn calculate_checksum(
    ctx: &DalContext,
    mut ids_with_hashes: Vec<(EntityId, MerkleTreeHash)>,
) -> ChangeSetResult<Checksum> {
    // If an empty list of IDs with hashes wass passed in, then we use the root node's ID and
    // merkle tree hash as our sole ID and hash so that algorithms using the checksum can
    // "invalidate" as needed.
    if ids_with_hashes.is_empty() {
        let root_node_id = ctx
            .workspace_snapshot()
            .map_err(Box::new)?
            .root()
            .await
            .map_err(Box::new)?;
        let root_node = ctx
            .workspace_snapshot()
            .map_err(Box::new)?
            .get_node_weight(root_node_id)
            .await
            .map_err(Box::new)?;

        ids_with_hashes.push((root_node_id.into(), root_node.merkle_tree_hash()));
    }

    // We MUST sort IDs (not hashes) before creating the checksum. This is so that we have
    // stable checksum calculation.
    ids_with_hashes.sort_by_key(|(id, _)| *id);

    // Now that we have strictly ordered IDs with hasesh and there's at least one group
    // present, we can create the checksum.
    let mut hasher = Checksum::hasher();
    for (_, hash) in ids_with_hashes {
        hasher.update(hash.as_bytes());
    }
    Ok(hasher.finalize())
}
