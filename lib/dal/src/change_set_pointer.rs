//! The sequel to [`ChangeSets`](crate::ChangeSet). Coming to an SI instance near you!

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_pg::{PgError, PgRow};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::{Generator, Ulid};

use crate::workspace_snapshot::vector_clock::VectorClockId;
use crate::workspace_snapshot::WorkspaceSnapshotId;
use crate::{pk, DalContext, TransactionsError};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetPointerError {
    #[error("ulid monotonic error: {0}")]
    Monotonic(#[from] ulid::MonotonicError),
    #[error("mutex error: {0}")]
    Mutex(String),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ChangeSetPointerResult<T> = Result<T, ChangeSetPointerError>;

pk!(ChangeSetPointerId);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeSetPointer {
    pub id: ChangeSetPointerId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub name: String,
    pub base_change_set_id: Option<ChangeSetPointerId>,
    pub workspace_snapshot_id: Option<WorkspaceSnapshotId>,

    #[serde(skip)]
    pub generator: Arc<Mutex<Generator>>,
}

impl TryFrom<PgRow> for ChangeSetPointer {
    type Error = ChangeSetPointerError;

    fn try_from(value: PgRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.try_get("id")?,
            created_at: value.try_get("created_at")?,
            updated_at: value.try_get("updated_at")?,
            name: value.try_get("name")?,
            base_change_set_id: value.try_get("base_change_set_id")?,
            workspace_snapshot_id: value.try_get("workspace_snapshot_id")?,
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
            generator: Arc::new(Mutex::new(generator)),
            base_change_set_id: None,
            workspace_snapshot_id: None,
            name: "".to_string(),
        })
    }

    pub fn editing_changeset(&self) -> ChangeSetPointerResult<Self> {
        let mut new_local = Self::new_local()?;
        new_local.base_change_set_id = self.base_change_set_id;
        new_local.workspace_snapshot_id = self.workspace_snapshot_id;
        new_local.name = self.name.to_owned();
        Ok(new_local)
    }

    pub async fn new(
        ctx: &DalContext,
        name: impl AsRef<str>,
        base_change_set_id: Option<ChangeSetPointerId>,
    ) -> ChangeSetPointerResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO change_set_pointers (name, base_change_set_id) VALUES ($1, $2) RETURNING *",
                &[&name, &base_change_set_id],
            )
            .await?;
        Self::try_from(row)
    }

    pub async fn new_head(ctx: &DalContext) -> ChangeSetPointerResult<Self> {
        let name = "HEAD";
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO change_set_pointers (id, name, base_change_set_id) VALUES ($1, $2, $3) RETURNING *",
                &[&ChangeSetPointerId::NONE, &name, &None::<ChangeSetPointerId>],
            )
            .await?;
        Self::try_from(row)
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
}

impl std::fmt::Debug for ChangeSetPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeSetPointer")
            .field("id", &self.id.to_string())
            .finish()
    }
}
