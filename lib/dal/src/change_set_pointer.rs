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
use crate::{pk, ChangeSetStatus, DalContext, TransactionsError, Workspace, WorkspacePk};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetPointerError {
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("could not find default change set: {0}")]
    DefaultChangeSetNotFound(ChangeSetPointerId),
    #[error("default change set {0} has no workspace snapshot pointer")]
    DefaultChangeSetNoWorkspaceSnapshotPointer(ChangeSetPointerId),
    #[error("enum parse error: {0}")]
    EnumParse(#[from] strum::ParseError),
    #[error("ulid monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("mutex error: {0}")]
    Mutex(String),
    #[error("Changeset {0} does not have a base change set")]
    NoBaseChangeSet(ChangeSetPointerId),
    #[error("no tenancy set in context")]
    NoTenancySet,
    #[error("Changeset {0} does not have a workspace snapshot")]
    NoWorkspaceSnapshot(ChangeSetPointerId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace error: {0}")]
    Workspace(String),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
}

pub type ChangeSetPointerResult<T> = Result<T, ChangeSetPointerError>;

pk!(ChangeSetPointerId);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeSetPointer {
    pub id: ChangeSetPointerId,
    pub pk: ChangeSetPointerId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name: String,
    pub status: ChangeSetStatus,
    pub base_change_set_id: Option<ChangeSetPointerId>,
    pub workspace_snapshot_id: Option<WorkspaceSnapshotId>,
    pub workspace_id: Option<WorkspacePk>,

    #[serde(skip)]
    pub generator: Arc<Mutex<Generator>>,
}

impl TryFrom<PgRow> for ChangeSetPointer {
    type Error = ChangeSetPointerError;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        let status_string: String = value.try_get("status")?;
        let status = ChangeSetStatus::try_from(status_string.as_str())?;
        Ok(Self {
            id: value.try_get("id")?,
            pk: value.try_get("id")?,
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

impl ChangeSetPointer {
    pub fn new_local() -> ChangeSetPointerResult<Self> {
        let mut generator = Generator::new();
        let id = generator.generate()?;

        Ok(Self {
            id: id.into(),
            pk: id.into(),
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

    pub fn editing_changeset(&self) -> ChangeSetPointerResult<Self> {
        let mut new_local = Self::new_local()?;
        new_local.base_change_set_id = self.base_change_set_id;
        new_local.workspace_snapshot_id = self.workspace_snapshot_id;
        new_local.workspace_id = self.workspace_id;
        new_local.name = self.name.to_owned();
        new_local.status = self.status.to_owned();
        Ok(new_local)
    }

    pub async fn new_with_id(
        ctx: &DalContext,
        name: impl AsRef<str>,
        id: ChangeSetPointerId,
        base_change_set_id: Option<ChangeSetPointerId>,
    ) -> ChangeSetPointerResult<Self> {
        let workspace_id = ctx.tenancy().workspace_pk();
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO change_set_pointers (id, name, base_change_set_id, status, workspace_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
                &[&id, &name, &base_change_set_id, &ChangeSetStatus::Open.to_string(), &workspace_id],
            )
            .await?;
        Self::try_from(row)
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        base_change_set_id: Option<ChangeSetPointerId>,
    ) -> ChangeSetPointerResult<Self> {
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
        Self::try_from(row)
    }

    pub async fn fork_head(
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> ChangeSetPointerResult<Self> {
        let workspace_pk = ctx
            .tenancy()
            .workspace_pk()
            .ok_or(ChangeSetPointerError::NoTenancySet)?;

        let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
            .await
            .map_err(|err| ChangeSetPointerError::Workspace(err.to_string()))?
            .ok_or(ChangeSetPointerError::WorkspaceNotFound(workspace_pk))?;

        let base_change_set_pointer =
            ChangeSetPointer::find(ctx, workspace.default_change_set_id())
                .await?
                .ok_or(ChangeSetPointerError::DefaultChangeSetNotFound(
                    workspace.default_change_set_id(),
                ))?;

        let mut change_set_pointer =
            ChangeSetPointer::new(ctx, name, Some(workspace.default_change_set_id())).await?;

        change_set_pointer
            .update_pointer(
                ctx,
                base_change_set_pointer.workspace_snapshot_id.ok_or(
                    ChangeSetPointerError::DefaultChangeSetNoWorkspaceSnapshotPointer(
                        workspace.default_change_set_id(),
                    ),
                )?,
            )
            .await?;

        Ok(change_set_pointer)
    }

    pub async fn new_head(ctx: &DalContext) -> ChangeSetPointerResult<Self> {
        let name = "HEAD";

        Self::new_with_id(ctx, name, ChangeSetPointerId::NONE, None).await
    }

    /// Create a [`VectorClockId`] from the [`ChangeSetPointer`].
    pub fn vector_clock_id(&self) -> VectorClockId {
        VectorClockId::from(Ulid::from(self.id))
    }

    pub fn generate_ulid(&self) -> ChangeSetPointerResult<Ulid> {
        self.generator
            .lock()
            .map_err(|e| ChangeSetPointerError::Mutex(e.to_string()))?
            .generate()
            .map_err(Into::into)
    }

    pub async fn update_workspace_id(
        &mut self,
        ctx: &DalContext,
        workspace_id: WorkspacePk,
    ) -> ChangeSetPointerResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE change_set_pointers SET workspace_id = $2 WHERE id = $1",
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
    ) -> ChangeSetPointerResult<()> {
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
    ) -> ChangeSetPointerResult<()> {
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
        change_set_pointer_id: ChangeSetPointerId,
    ) -> ChangeSetPointerResult<Option<Self>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT * FROM change_set_pointers WHERE id = $1",
                &[&change_set_pointer_id],
            )
            .await?;

        match row {
            Some(row) => Ok(Some(Self::try_from(row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_open(ctx: &DalContext) -> ChangeSetPointerResult<Vec<Self>> {
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

    pub async fn apply_to_base_change_set(&self, ctx: &DalContext) -> ChangeSetPointerResult<()> {
        let to_rebase_change_set_id = self
            .base_change_set_id
            .ok_or(ChangeSetPointerError::NoBaseChangeSet(self.id))?;
        let onto_workspace_snapshot_id = self
            .workspace_snapshot_id
            .ok_or(ChangeSetPointerError::NoWorkspaceSnapshot(self.id))?;
        let rebase_request = RebaseRequest {
            onto_workspace_snapshot_id,
            onto_vector_clock_id: self.vector_clock_id(),
            to_rebase_change_set_id,
        };
        ctx.do_rebase_request(rebase_request).await?;

        Ok(())
    }
}

impl std::fmt::Debug for ChangeSetPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeSetPointer")
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
