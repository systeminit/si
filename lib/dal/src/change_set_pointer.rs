//! The sequel to [`ChangeSets`](crate::ChangeSet). Coming to an SI instance near you!

use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::{Generator, Ulid};

use crate::workspace_snapshot::WorkspaceSnapshotId;
use crate::{pk, standard_model, DalContext, StandardModelError, Timestamp, TransactionsError};

const FIND: &str = include_str!("queries/change_set_pointers/find.sql");

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
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

pub type ChangeSetPointerResult<T> = Result<T, ChangeSetPointerError>;

pk!(ChangeSetPointerId);

#[derive(Clone, Serialize, Deserialize)]
pub struct ChangeSetPointer {
    pub id: ChangeSetPointerId,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    #[serde(skip)]
    pub generator: Arc<Mutex<Generator>>,
    pub workspace_snapshot_id: Option<WorkspaceSnapshotId>,
    pub name: String,
}

impl ChangeSetPointer {
    pub fn new_local() -> ChangeSetPointerResult<Self> {
        let mut generator = Generator::new();
        let id = generator.generate()?;

        Ok(Self {
            id: id.into(),
            timestamp: Timestamp::now(),
            generator: Arc::new(Mutex::new(generator)),
            workspace_snapshot_id: None,
            name: "".to_string(),
        })
    }

    pub async fn new(ctx: &DalContext, name: impl AsRef<str>) -> ChangeSetPointerResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT change_set_pointer_create_v1($1) AS object",
                &[&name],
            )
            .await?;
        let json: Value = row.try_get("object")?;
        let object: Self = serde_json::from_value(json)?;
        Ok(object)
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
                "UPDATE change_set_pointers AS object SET workspace_snapshot_id = $2 WHERE id = $1",
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
    ) -> ChangeSetPointerResult<Self> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(FIND, &[&change_set_pointer_id])
            .await?;
        Ok(standard_model::object_from_row(row)?)
    }
}

impl std::fmt::Debug for ChangeSetPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeSetPointer")
            .field("id", &self.id.to_string())
            .finish()
    }
}
