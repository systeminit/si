use chrono::{DateTime, Utc};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::{PgError, PgRow};
use si_events::ContentHash;
use si_layer_cache::db::serialize;
use si_layer_cache::LayerDbError;
use si_pkg::{
    WorkspaceExport, WorkspaceExportChangeSetV0, WorkspaceExportContentV0,
    WorkspaceExportMetadataV0,
};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set::{ChangeSet, ChangeSetError, ChangeSetId};
use crate::feature_flags::FeatureFlag;
use crate::layer_db_types::ContentTypes;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, standard_model, standard_model_accessor_ro, DalContext, HistoryActor, HistoryEvent,
    HistoryEventError, KeyPairError, StandardModelError, Tenancy, Timestamp, TransactionsError,
    User, UserError, UserPk, WorkspaceSnapshot,
};

const WORKSPACE_GET_BY_PK: &str = include_str!("queries/workspace/get_by_pk.sql");
const WORKSPACE_LIST_FOR_USER: &str = include_str!("queries/workspace/list_for_user.sql");

const DEFAULT_BUILTIN_WORKSPACE_NAME: &str = "builtin";
const DEFAULT_CHANGE_SET_NAME: &str = "HEAD";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("builtin workspace not found")]
    BuiltinWorkspaceNotFound,
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set not found by id: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("Trying to export from system actor. This can only be done by a user actor")]
    ExportingFromSystemActor,
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error("Trying to import a changeset that does not have a valid base: {0}")]
    ImportingOrphanChangeset(ChangeSetId),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error(transparent)]
    KeyPair(#[from] KeyPairError),
    #[error("LayerDb error: {0}")]
    LayerDb(#[from] LayerDbError),
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
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

pk!(WorkspacePk);
pk!(WorkspaceId);

impl From<WorkspacePk> for si_events::WorkspacePk {
    fn from(value: WorkspacePk) -> Self {
        let id: ulid::Ulid = value.into();
        id.into()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pk: WorkspacePk,
    name: String,
    default_change_set_id: ChangeSetId,
    uses_actions_v2: bool,
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
            uses_actions_v2: row.try_get("uses_actions_v2")?,
            timestamp: Timestamp::assemble(created_at, updated_at),
        })
    }
}

impl Workspace {
    pub fn pk(&self) -> &WorkspacePk {
        &self.pk
    }

    pub fn default_change_set_id(&self) -> ChangeSetId {
        self.default_change_set_id
    }

    pub fn uses_actions_v2(&self) -> bool {
        self.uses_actions_v2
    }

    pub async fn update_default_change_set_id(
        &mut self,
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WorkspaceResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE workspaces SET default_change_set_id = $2 WHERE pk = $1",
                &[&self.pk, &change_set_id],
            )
            .await?;

        self.default_change_set_id = change_set_id;

        Ok(())
    }

    /// Find or create the builtin [`Workspace`].
    #[instrument(skip_all)]
    pub async fn setup_builtin(ctx: &mut DalContext) -> WorkspaceResult<()> {
        // Check if the builtin already exists. If so, update our tenancy and visibility using it.
        if let Some(found_builtin) = Self::find_builtin(ctx).await? {
            ctx.update_tenancy(Tenancy::new(*found_builtin.pk()));
            if let Err(err) = ctx
                .update_visibility_and_snapshot_to_visibility(found_builtin.default_change_set_id)
                .await
            {
                if err.is_unmigrated_snapshot_error() {
                    ChangeSet::migrate_change_set_snapshot(
                        ctx,
                        found_builtin.default_change_set_id,
                    )
                    .await?;
                    ctx.commit_no_rebase().await?;
                    ctx.update_visibility_and_snapshot_to_visibility(
                        found_builtin.default_change_set_id,
                    )
                    .await?;
                } else {
                    Err(err)?;
                }
            }
            return Ok(());
        }

        let initial_change_set = ChangeSet::new_local()?;
        let workspace_snapshot = WorkspaceSnapshot::initial(ctx, &initial_change_set).await?;

        // If not, create the builtin workspace with a corresponding base change set and initial
        // workspace snapshot.
        let mut change_set = ChangeSet::new(
            ctx,
            DEFAULT_CHANGE_SET_NAME,
            None,
            workspace_snapshot.id().await,
        )
        .await?;
        let change_set_id = change_set.id;

        let head_pk = WorkspaceId::NONE;

        let uses_actions_v2 = ctx
            .services_context()
            .feature_flags_service()
            .feature_is_enabled(&FeatureFlag::ActionsV2);

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspaces (pk, name, default_change_set_id, uses_actions_v2) VALUES ($1, $2, $3, $4) RETURNING *",
                &[&head_pk, &DEFAULT_BUILTIN_WORKSPACE_NAME, &change_set_id, &uses_actions_v2],
            )
            .await?;

        let workspace = Self::try_from(row)?;
        let workspace_pk = *workspace.pk();

        change_set.update_workspace_id(ctx, workspace_pk).await?;

        // Update our tenancy and visibility once it has been created.
        ctx.update_tenancy(Tenancy::new(workspace_pk));
        ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
            .await?;

        Ok(())
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
            None => return Err(WorkspaceError::BuiltinWorkspaceNotFound),
        };

        // Create a new change set whose base is the default change set of the workspace.
        // Point to the snapshot that the builtin's default change set is pointing to.
        let workspace_snapshot =
            WorkspaceSnapshot::find_for_change_set(ctx, builtin.default_change_set_id).await?;
        let mut change_set = ChangeSet::new(
            ctx,
            "HEAD",
            Some(builtin.default_change_set_id),
            workspace_snapshot.id().await,
        )
        .await?;
        let change_set_id = change_set.id;

        let uses_actions_v2 = ctx
            .services_context()
            .feature_flags_service()
            .feature_is_enabled(&FeatureFlag::ActionsV2);

        let name = name.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspaces (pk, name, default_change_set_id, uses_actions_v2) VALUES ($1, $2, $3, $4) RETURNING *",
                &[&pk, &name, &change_set_id, &uses_actions_v2],
            )
            .await?;
        let new_workspace = Self::try_from(row)?;

        change_set
            .update_workspace_id(ctx, *new_workspace.pk())
            .await?;

        ctx.update_tenancy(Tenancy::new(new_workspace.pk));

        // TODO(nick,zack,jacob): convert visibility (or get rid of it?) to use our the new change set id.
        // should set_change_set and set_workspace_snapshot happen in update_visibility?
        ctx.update_visibility_and_snapshot_to_visibility(change_set.id)
            .await?;

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

        Ok(workspace)
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

    pub async fn get_by_pk_or_error(
        ctx: &DalContext,
        pk: &WorkspacePk,
    ) -> WorkspaceResult<Workspace> {
        Self::get_by_pk(ctx, pk)
            .await?
            .ok_or(WorkspaceError::WorkspaceNotFound(*pk))
    }

    pub async fn generate_export_data(
        &self,
        ctx: &DalContext,
        workspace_version: &str,
    ) -> WorkspaceResult<WorkspaceExport> {
        let mut content_hashes = vec![];
        let mut change_sets: HashMap<Ulid, Vec<WorkspaceExportChangeSetV0>> = HashMap::new();
        let mut default_change_set_base = Ulid::nil();
        for change_set in ChangeSet::list_open(ctx).await? {
            let snap = WorkspaceSnapshot::find_for_change_set(ctx, change_set.id).await?;

            // From root, get every value from every node, store with hash
            let mut queue = VecDeque::from([snap.root().await?]);

            while let Some(this_node_idx) = queue.pop_front() {
                // Queue contents
                content_hashes.extend(
                    snap.get_node_weight(this_node_idx)
                        .await?
                        .content_store_hashes(),
                );

                let children = snap
                    .edges_directed_by_index(this_node_idx, Direction::Outgoing)
                    .await?
                    .into_iter()
                    .map(|(_, _, target)| target)
                    .collect::<VecDeque<_>>();

                queue.extend(children)
            }

            let base_changeset = change_set
                .base_change_set_id
                .map(|id| id.into_inner())
                .unwrap_or(Ulid::nil());

            if change_set.id == self.default_change_set_id() {
                default_change_set_base = base_changeset
            }

            change_sets
                .entry(base_changeset)
                .or_default()
                .push(WorkspaceExportChangeSetV0 {
                    id: change_set.id.into_inner(),
                    name: change_set.name.clone(),
                    base_change_set_id: change_set.base_change_set_id.map(|id| id.into_inner()),
                    workspace_snapshot_serialized_data: snap.serialized().await?,
                })
        }

        let store_values_map = ctx
            .layer_db()
            .cas()
            .read_many(content_hashes.as_ref())
            .await?
            .into_iter()
            .map(|(hash, content)| (hash, (content, "postcard".to_string())))
            .collect::<HashMap<_, _>>();

        let content_store_values = serialize::to_vec(&store_values_map)?;

        let created_by = if let HistoryActor::User(user_pk) = ctx.history_actor() {
            let user = User::get_by_pk(ctx, *user_pk)
                .await?
                .ok_or(WorkspaceError::InvalidUser(*user_pk))?;

            user.email().clone()
        } else {
            "SystemInit".to_string()
        };

        let metadata = WorkspaceExportMetadataV0 {
            name: self.name().clone(),
            version: workspace_version.to_string(),
            description: "Workspace Backup".to_string(), // TODO Get this from the user
            created_at: Default::default(),
            created_by,
            default_change_set: self.default_change_set_id().into_inner(),
            default_change_set_base,
            workspace_pk: self.pk().into_inner(),
            workspace_name: self.name().clone(),
        };

        Ok(WorkspaceExport::new(WorkspaceExportContentV0 {
            change_sets,
            content_store_values,
            metadata,
        }))
    }

    pub async fn import(
        &mut self,
        ctx: &DalContext,
        workspace_data: WorkspaceExport,
    ) -> WorkspaceResult<()> {
        let WorkspaceExportContentV0 {
            change_sets,
            content_store_values,
            metadata,
        } = workspace_data.into_latest();

        // ABANDON PREVIOUS CHANGESETS
        for mut change_set in ChangeSet::list_open(ctx).await? {
            change_set.abandon(ctx).await?;
        }

        let base_changeset_for_default = {
            let changeset_id = self.default_change_set_id();

            let changeset = ChangeSet::find(ctx, changeset_id)
                .await?
                .ok_or(WorkspaceError::ChangeSetNotFound(changeset_id))?;

            changeset.base_change_set_id
        };

        // Go from head changeset to children, creating new changesets and updating base references
        let mut base_change_set_queue = VecDeque::from([metadata.default_change_set_base]);
        let mut change_set_id_map = HashMap::new();
        while let Some(base_change_set_ulid) = base_change_set_queue.pop_front() {
            let Some(change_sets) = change_sets.get(&base_change_set_ulid) else {
                continue;
            };

            for change_set_data in change_sets {
                let imported_snapshot = WorkspaceSnapshot::from_bytes(
                    &change_set_data.workspace_snapshot_serialized_data,
                )
                .await?;

                // If base_change_set is default_change_set_base, it pointed to the builtin workspace
                // originally, so this change set needs to be the new default for the workspace - HEAD
                let mut is_new_default = false;
                let actual_base_changeset: Option<ChangeSetId> =
                    if base_change_set_ulid == metadata.default_change_set_base {
                        is_new_default = true;
                        base_changeset_for_default
                    } else {
                        Some(*change_set_id_map.get(&base_change_set_ulid).ok_or(
                            WorkspaceError::ImportingOrphanChangeset(base_change_set_ulid.into()),
                        )?)
                    };

                let local_change_set = ChangeSet::new_local()?;
                let new_snap_address = imported_snapshot
                    .write(ctx, local_change_set.vector_clock_id())
                    .await?;

                let new_change_set = ChangeSet::new(
                    ctx,
                    change_set_data.name.clone(),
                    actual_base_changeset,
                    new_snap_address,
                )
                .await?;

                change_set_id_map.insert(change_set_data.id, new_change_set.id);

                // Set new default changeset for workspace
                if is_new_default {
                    self.update_default_change_set_id(ctx, new_change_set.id)
                        .await?;
                }

                base_change_set_queue.push_back(change_set_data.id)
            }
        }

        let cas_values: HashMap<ContentHash, (Arc<ContentTypes>, String)> =
            serialize::from_bytes(&content_store_values)?;

        let layer_db = ctx.layer_db();

        // TODO use the serialization format to ensure we're hashing the data correctly, if we change the format
        for (_, (content, _serialization_format)) in cas_values {
            layer_db
                .cas()
                .write(content, None, ctx.events_tenancy(), ctx.events_actor())
                .await?;
        }

        Ok(())
    }

    standard_model_accessor_ro!(name, String);
}
