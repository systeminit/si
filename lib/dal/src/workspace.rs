use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgRow};
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set_pointer::{ChangeSetPointer, ChangeSetPointerError, ChangeSetPointerId};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, standard_model, standard_model_accessor_ro, DalContext, HistoryActor, HistoryEvent,
    HistoryEventError, KeyPairError, StandardModelError, Tenancy, Timestamp, TransactionsError,
    UserError, WorkspaceSnapshot,
};

const WORKSPACE_GET_BY_PK: &str = include_str!("queries/workspace/get_by_pk.sql");
const WORKSPACE_FIND_BY_NAME: &str = include_str!("queries/workspace/find_by_name.sql");
const WORKSPACE_LIST_FOR_USER: &str = include_str!("queries/workspace/list_for_user.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("change set pointer error: {0}")]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error(transparent)]
    KeyPair(#[from] KeyPairError),
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error("no user in context")]
    NoUserInContext,
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    User(#[from] UserError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

pk!(WorkspacePk);
pk!(WorkspaceId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pk: WorkspacePk,
    name: String,
    default_change_set_id: ChangeSetPointerId,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl TryFrom<PgRow> for Workspace {
    type Error = WorkspaceError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        Ok(Self {
            pk: row.try_get("pk")?,
            name: row.try_get("name")?,
            default_change_set_id: row.try_get("default_change_set_id")?,
            timestamp: Timestamp::assemble(created_at, updated_at),
        })
    }
}

impl Workspace {
    pub fn pk(&self) -> &WorkspacePk {
        &self.pk
    }

    pub fn default_change_set_id(&self) -> ChangeSetPointerId {
        self.default_change_set_id
    }

    /// Find or create the builtin [`Workspace`].
    #[instrument(skip_all)]
    pub async fn builtin(ctx: &mut DalContext) -> WorkspaceResult<Self> {
        // Check if the builtin already exists.
        if let Some(found_builtin) = Self::find_builtin(ctx).await? {
            return Ok(found_builtin);
        }

        // If not, create the builtin workspace with a corresponding base change set and initial
        // workspace snapshot.
        let name = "builtin";
        let mut change_set = ChangeSetPointer::new_head(ctx).await?;
        let workspace_snapshot = WorkspaceSnapshot::initial(ctx, &change_set).await?;
        change_set
            .update_pointer(ctx, workspace_snapshot.id())
            .await?;
        let change_set_id = change_set.id;

        let head_pk = WorkspaceId::NONE;

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspaces (pk, name, default_change_set_id) VALUES ($1, $2, $3) RETURNING *",
                &[&head_pk, &name, &change_set_id],
            )
            .await?;

        let workspace = Self::try_from(row)?;

        change_set.update_workspace_id(ctx, *workspace.pk()).await?;

        Ok(workspace)
    }

    /// This private method attempts to find the builtin [`Workspace`].
    #[instrument(skip_all)]
    async fn find_builtin(ctx: &DalContext) -> WorkspaceResult<Option<Self>> {
        let head_pk = WorkspaceId::NONE;
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt("SELECT * FROM workspaces WHERE pk = $1", &[&head_pk])
            .await?;
        let maybe_builtin = match maybe_row {
            Some(found) => Some(Self::try_from(found)?),
            None => None,
        };
        Ok(maybe_builtin)
    }

    pub async fn list_for_user(ctx: &DalContext) -> WorkspaceResult<Vec<Self>> {
        let user_pk = match ctx.history_actor() {
            HistoryActor::User(user_pk) => *user_pk,
            _ => return Err(WorkspaceError::NoUserInContext),
        };
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(WORKSPACE_LIST_FOR_USER, &[&user_pk])
            .await?;

        Ok(standard_model::objects_from_rows(rows)?)
    }

    pub async fn find_first_user_workspace(ctx: &DalContext) -> WorkspaceResult<Option<Self>> {
        let maybe_row = ctx.txns().await?.pg().query_opt(
            "SELECT row_to_json(w.*) AS object FROM workspaces AS w WHERE pk != $1 ORDER BY created_at ASC LIMIT 1", &[&WorkspacePk::NONE],
        ).await?;
        let maybe_workspace = match maybe_row {
            Some(found) => Some(Self::try_from(found)?),
            None => None,
        };
        Ok(maybe_workspace)
    }

    pub async fn new(
        ctx: &mut DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
    ) -> WorkspaceResult<Self> {
        // Get the default change set from the builtin workspace.
        let builtin = match Self::find_builtin(ctx).await? {
            Some(found_builtin) => found_builtin,
            None => {
                // TODO(nick,jacob): replace this with an error.
                todo!("this should not happen")
            }
        };

        // Create a new change set whose base is the default change set of the workspace.
        // Point to the snapshot that the builtin's default change set is pointing to.
        let mut change_set =
            ChangeSetPointer::new(ctx, "HEAD", Some(builtin.default_change_set_id)).await?;
        let workspace_snapshot =
            WorkspaceSnapshot::find_for_change_set(ctx, builtin.default_change_set_id).await?;
        change_set
            .update_pointer(ctx, workspace_snapshot.id())
            .await?;
        let change_set_id = change_set.id;

        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspaces (pk, name, default_change_set_id) VALUES ($1, $2, $3) RETURNING *",
                &[&pk, &name, &change_set_id],
            )
            .await?;
        let new_workspace = Self::try_from(row)?;

        change_set
            .update_workspace_id(ctx, *new_workspace.pk())
            .await?;

        ctx.update_tenancy(Tenancy::new(new_workspace.pk));

        // TODO(nick,zack,jacob): convert visibility (or get rid of it?) to use our the new change set id.
        // should set_change_set_pointer and set_workspace_snapshot happen in update_visibility?
        ctx.update_visibility_v2(&change_set);
        ctx.update_snapshot_to_visibility().await?;

        let _history_event = HistoryEvent::new(
            ctx,
            "workspace.create".to_owned(),
            "Workspace created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;
        Ok(new_workspace)
    }

    pub async fn clear(&self, ctx: &DalContext) -> WorkspaceResult<()> {
        let tenancy = Tenancy::new(self.pk);

        ctx.txns()
            .await?
            .pg()
            .execute("SELECT clear_workspace_v1($1)", &[&tenancy])
            .await?;

        Ok(())
    }

    pub async fn clear_or_create_workspace(
        ctx: &mut DalContext,
        workspace_pk: WorkspacePk,
        workspace_name: impl AsRef<str>,
    ) -> WorkspaceResult<Self> {
        let workspace = match Workspace::get_by_pk(ctx, &workspace_pk).await? {
            Some(existing_workspace) => {
                existing_workspace.clear(ctx).await?;
                existing_workspace
            }
            None => Workspace::new(ctx, workspace_pk, workspace_name).await?,
        };

        ctx.import_builtins().await?;

        Ok(workspace)
    }

    pub async fn find_by_name(ctx: &DalContext, name: &str) -> WorkspaceResult<Option<Workspace>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(WORKSPACE_FIND_BY_NAME, &[&name])
            .await?;
        let maybe_workspace = match maybe_row {
            Some(found) => Some(Self::try_from(found)?),
            None => None,
        };
        Ok(maybe_workspace)
    }

    pub async fn get_by_pk(
        ctx: &DalContext,
        pk: &WorkspacePk,
    ) -> WorkspaceResult<Option<Workspace>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(WORKSPACE_GET_BY_PK, &[&pk])
            .await?;
        if let Some(row) = row {
            let json: serde_json::Value = row.try_get("object")?;
            Ok(serde_json::from_value(json)?)
        } else {
            Ok(None)
        }
    }

    standard_model_accessor_ro!(name, String);
}
