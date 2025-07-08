use std::{
    collections::{
        HashMap,
        VecDeque,
    },
    str::FromStr,
    sync::Arc,
};

use chrono::{
    DateTime,
    Utc,
};
use petgraph::Direction;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_nats::NatsError;
use si_data_pg::{
    PgError,
    PgRow,
};
use si_db::{
    HistoryActor,
    HistoryEvent,
    Tenancy,
    User,
    workspace::{
        SEARCH_WORKSPACES_BY_SNAPSHOT_ADDRESS,
        SEARCH_WORKSPACES_BY_ULID,
        SEARCH_WORKSPACES_USER_NAME_EMAIL,
        WORKSPACE_GET_BY_PK,
        WORKSPACE_LIST_ALL,
        WORKSPACE_LIST_FOR_USER,
    },
};
use si_events::{
    ContentHash,
    Timestamp,
    WorkspaceSnapshotAddress,
};
pub use si_id::{
    WorkspaceId,
    WorkspacePk,
};
use si_layer_cache::{
    LayerDbError,
    db::serialize,
};
use si_pkg::{
    WorkspaceExport,
    WorkspaceExportChangeSetV0,
    WorkspaceExportContentV0,
    WorkspaceExportMetadataV0,
};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    BuiltinsError,
    DalContext,
    KeyPairError,
    TransactionsError,
    WorkspaceSnapshot,
    WorkspaceSnapshotGraph,
    builtins::func::migrate_intrinsics_no_commit,
    change_set::{
        ChangeSet,
        ChangeSetError,
        ChangeSetId,
    },
    feature_flags::FeatureFlag,
    getter,
    layer_db_types::ContentTypes,
    workspace_integrations::{
        WorkspaceIntegration,
        WorkspaceIntegrationsError,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        WorkspaceSnapshotSelector,
        graph::WorkspaceSnapshotGraphDiscriminants,
        selector::WorkspaceSnapshotSelectorDiscriminants,
        split_snapshot::{
            SplitSnapshot,
            SplitSnapshotGraph,
            SubGraphVersionDiscriminants,
            SuperGraphVersionDiscriminants,
        },
    },
};

const DEFAULT_BUILTIN_WORKSPACE_NAME: &str = "builtin";
const DEFAULT_BUILTIN_WORKSPACE_TOKEN: &str = "builtin";
const DEFAULT_CHANGE_SET_NAME: &str = "HEAD";
const DEFAULT_COMPONENT_CONCURRENCY_LIMIT: i32 = 256;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WorkspaceError {
    #[error("builtins error: {0}")]
    Builtins(#[from] Box<BuiltinsError>),
    #[error("builtin workspace not found")]
    BuiltinWorkspaceNotFound,
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("could not find default change set {1} for workspace {0}")]
    DefaultChangeSetNotFound(WorkspacePk, ChangeSetId),
    #[error("Trying to export from system actor. This can only be done by a user actor")]
    ExportingFromSystemActor,
    #[error("Trying to import a changeset that does not have a valid base: {0}")]
    ImportingOrphanChangeset(ChangeSetId),
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
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("strum parse error: {0}")]
    StrumParse(#[from] strum::ParseError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("unknown snapshot kind {0} for workspace: {1}")]
    UnknownSnapshotKind(String, WorkspacePk),
    #[error("workspace integration error: {0}")]
    WorkspaceIntegration(#[from] WorkspaceIntegrationsError),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type WorkspaceResult<T> = Result<T, WorkspaceError>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotVersion {
    Legacy(WorkspaceSnapshotGraphDiscriminants),
    Split(SuperGraphVersionDiscriminants),
}

impl SnapshotVersion {
    pub fn db_string(&self) -> String {
        match self {
            SnapshotVersion::Legacy(legacy) => legacy.to_string(),
            SnapshotVersion::Split(split) => split.to_string(),
        }
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
    token: Option<String>,
    snapshot_version: SnapshotVersion,
    component_concurrency_limit: Option<i32>,
    snapshot_kind: WorkspaceSnapshotSelectorDiscriminants,
    subgraph_version: Option<SubGraphVersionDiscriminants>,
    approvals_enabled: bool,
}

impl TryFrom<PgRow> for Workspace {
    type Error = WorkspaceError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let updated_at: DateTime<Utc> = row.try_get("updated_at")?;
        let snapshot_version_string: String = row.try_get("snapshot_version")?;
        let snapshot_kind_string: String = row.try_get("snapshot_kind")?;
        let subgraph_version_string: Option<String> = row.try_get("subgraph_version")?;
        let snapshot_kind =
            WorkspaceSnapshotSelectorDiscriminants::from_str(&snapshot_kind_string)?;

        let snapshot_version = match snapshot_kind {
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => SnapshotVersion::Legacy(
                WorkspaceSnapshotGraphDiscriminants::from_str(&snapshot_version_string)?,
            ),
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => SnapshotVersion::Split(
                SuperGraphVersionDiscriminants::from_str(&snapshot_version_string)?,
            ),
        };

        let subgraph_version = match subgraph_version_string {
            Some(subgraph_version_string) => Some(SubGraphVersionDiscriminants::from_str(
                &subgraph_version_string,
            )?),
            None => None,
        };

        let pk = row.try_get("pk")?;
        Ok(Self {
            pk,
            name: row.try_get("name")?,
            default_change_set_id: row.try_get("default_change_set_id")?,
            uses_actions_v2: row.try_get("uses_actions_v2")?,
            timestamp: Timestamp {
                created_at,
                updated_at,
            },
            token: row.try_get("token")?,
            snapshot_version,
            component_concurrency_limit: row.try_get("component_concurrency_limit")?,
            snapshot_kind,
            subgraph_version,
            approvals_enabled: row.try_get("approvals_enabled")?,
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

    pub fn token(&self) -> Option<String> {
        self.token.clone()
    }

    pub fn snapshot_version(&self) -> SnapshotVersion {
        self.snapshot_version
    }

    pub fn snapshot_kind(&self) -> WorkspaceSnapshotSelectorDiscriminants {
        self.snapshot_kind
    }

    pub fn subgraph_version(&self) -> Option<SubGraphVersionDiscriminants> {
        self.subgraph_version
    }

    pub fn approvals_enabled(&self) -> bool {
        self.approvals_enabled
    }

    pub fn is_current_version_and_kind(&self) -> bool {
        match self.snapshot_kind() {
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => false,
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
                match self.snapshot_version() {
                    SnapshotVersion::Legacy(_) => false,
                    SnapshotVersion::Split(split_version) => {
                        split_version == SuperGraphVersionDiscriminants::current_discriminant()
                            && self.subgraph_version().is_some_and(|version| {
                                version == SubGraphVersionDiscriminants::current()
                            })
                    }
                }
            }
        }
    }

    pub async fn set_token(&mut self, ctx: &DalContext, token: String) -> WorkspaceResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE workspaces SET token = $2 WHERE pk = $1",
                &[&self.pk, &token],
            )
            .await?;
        self.token = Some(token);

        Ok(())
    }

    pub async fn update_default_change_set_id(
        &mut self,
        ctx: &mut DalContext,
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
        // Bust the default change set id in `DalContext`
        ctx.update_tenancy(*ctx.tenancy());

        Ok(())
    }

    pub async fn update_approvals_enabled(
        &mut self,
        ctx: &DalContext,
        approvals_enabled: bool,
    ) -> WorkspaceResult<()> {
        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE workspaces SET approvals_enabled = $2 WHERE pk = $1",
                &[&self.pk, &approvals_enabled],
            )
            .await?;

        self.approvals_enabled = approvals_enabled;

        Ok(())
    }

    /// Find or create the builtin [`Workspace`].
    #[instrument(skip_all)]
    pub async fn setup_builtin(ctx: &mut DalContext) -> WorkspaceResult<()> {
        // Check if the builtin already exists. If so, update our tenancy and visibility using it.
        if let Some(mut found_builtin) = Self::find_builtin(ctx).await? {
            // 'reset' the workspace snapshot so that we remigrate all builtins on each startup
            let new_snapshot = WorkspaceSnapshot::initial(ctx).await?;
            let new_snap_address = new_snapshot.write(ctx).await?;
            let new_change_set = ChangeSet::new(
                ctx,
                DEFAULT_CHANGE_SET_NAME,
                None,
                new_snap_address,
                WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
            )
            .await?;
            found_builtin
                .update_default_change_set_id(ctx, new_change_set.id)
                .await?;

            ctx.update_tenancy(Tenancy::new(*found_builtin.pk()));
            ctx.update_visibility_and_snapshot_to_visibility(found_builtin.default_change_set_id)
                .await?;

            return Ok(());
        }

        let workspace_snapshot = WorkspaceSnapshot::initial(ctx).await?;

        // If not, create the builtin workspace with a corresponding base change set and initial
        // workspace snapshot.
        let mut change_set = ChangeSet::new(
            ctx,
            DEFAULT_CHANGE_SET_NAME,
            None,
            workspace_snapshot.id().await,
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
        )
        .await?;
        let change_set_id = change_set.id;

        let head_pk = WorkspaceId::NONE;

        let uses_actions_v2 = ctx
            .services_context()
            .feature_flags_service()
            .feature_is_enabled(&FeatureFlag::ActionsV2);

        let version_string = WorkspaceSnapshotGraph::current_discriminant().to_string();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspaces (pk, name, default_change_set_id, uses_actions_v2, token, snapshot_version) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
                &[&head_pk, &DEFAULT_BUILTIN_WORKSPACE_NAME, &change_set_id, &uses_actions_v2, &DEFAULT_BUILTIN_WORKSPACE_TOKEN, &version_string],
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

    /// This method attempts to find the builtin [`Workspace`].
    #[instrument(skip_all)]
    pub async fn find_builtin(ctx: &DalContext) -> WorkspaceResult<Option<Self>> {
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

    pub async fn list_all(ctx: &DalContext) -> WorkspaceResult<Vec<Self>> {
        let rows = ctx
            .txns()
            .await?
            .pg()
            .query(WORKSPACE_LIST_ALL, &[])
            .await?;

        let mut result = vec![];

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
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

        let mut result = Vec::with_capacity(rows.len());

        for row in rows {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    pub async fn search(
        ctx: &DalContext,
        query: Option<&str>,
        limit: usize,
    ) -> WorkspaceResult<Vec<Self>> {
        let query = query.unwrap_or("").trim();

        let rows = if query.len() < 3 {
            let select_stmt =
                format!("SELECT * FROM workspaces AS w ORDER BY created_at DESC LIMIT {limit}");

            ctx.txns().await?.pg().query(&select_stmt, &[]).await?
        } else {
            let (select_stmt, query) = if ulid::Ulid::from_string(query).is_ok() {
                (
                    format!("{SEARCH_WORKSPACES_BY_ULID} LIMIT {limit}"),
                    query.to_string(),
                )
            } else if WorkspaceSnapshotAddress::from_str(query).is_ok() {
                (
                    format!("{SEARCH_WORKSPACES_BY_SNAPSHOT_ADDRESS} LIMIT {limit}"),
                    query.to_string(),
                )
            } else {
                (
                    format!("{SEARCH_WORKSPACES_USER_NAME_EMAIL} LIMIT {limit}"),
                    format!("%{query}%"),
                )
            };

            ctx.txns()
                .await?
                .pg()
                .query(&select_stmt, &[&query])
                .await?
        };

        let mut result = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            result.push(Self::try_from(row)?);
        }

        Ok(result)
    }

    pub async fn find_first_user_workspace(ctx: &DalContext) -> WorkspaceResult<Option<Self>> {
        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(
                "SELECT w.* FROM workspaces AS w WHERE pk != $1 ORDER BY created_at ASC LIMIT 1",
                &[&WorkspacePk::NONE],
            )
            .await?;

        let maybe_workspace = match maybe_row {
            Some(found) => Some(Self::try_from(found)?),
            None => None,
        };
        Ok(maybe_workspace)
    }

    /// Creates a new workspace with only the intrinsic functions installed,
    /// suitable for on demand asset installation
    pub async fn new_for_on_demand_assets(
        ctx: &mut DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
        token: impl AsRef<str>,
    ) -> WorkspaceResult<Self> {
        let workspace_snapshot = WorkspaceSnapshot::initial(ctx).await?;
        ctx.set_workspace_snapshot(workspace_snapshot);

        migrate_intrinsics_no_commit(ctx).await.map_err(Box::new)?;

        let workspace_snapshot_address = ctx
            .workspace_snapshot()?
            .as_legacy_snapshot()?
            .write(ctx)
            .await?;

        let mut head_change_set = ChangeSet::new(
            ctx,
            DEFAULT_CHANGE_SET_NAME,
            None,
            workspace_snapshot_address,
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
        )
        .await?;

        let workspace = Self::insert_workspace(
            ctx,
            pk,
            name,
            head_change_set.id,
            token,
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
        )
        .await?;

        head_change_set
            .update_workspace_id(ctx, workspace.pk)
            .await?;

        ctx.update_tenancy(Tenancy::new(pk));
        ctx.update_visibility_and_snapshot_to_visibility(head_change_set.id)
            .await?;

        let _history_event = HistoryEvent::new(
            ctx,
            "workspace.create".to_owned(),
            "Workspace created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;

        // Create an entry in the workspace integrations table by default
        WorkspaceIntegration::new(ctx, None).await?;

        Ok(workspace)
    }

    pub async fn new_split_graph_workspace(
        ctx: &mut DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
        token: impl AsRef<str>,
        split_max: usize,
    ) -> WorkspaceResult<Self> {
        let workspace_snapshot = SplitSnapshot::initial(ctx, split_max).await?;
        ctx.set_workspace_split_snapshot(workspace_snapshot);

        migrate_intrinsics_no_commit(ctx).await.map_err(Box::new)?;

        let workspace_snapshot_address = ctx
            .workspace_snapshot()?
            .as_split_snapshot()?
            .write(ctx)
            .await?;

        let mut head_change_set = ChangeSet::new(
            ctx,
            DEFAULT_CHANGE_SET_NAME,
            None,
            workspace_snapshot_address,
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot,
        )
        .await?;

        let workspace = Self::insert_workspace(
            ctx,
            pk,
            name,
            head_change_set.id,
            token,
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot,
        )
        .await?;
        head_change_set
            .update_workspace_id(ctx, workspace.pk)
            .await?;

        ctx.update_tenancy(Tenancy::new(pk));
        ctx.update_visibility_and_snapshot_to_visibility(head_change_set.id)
            .await?;

        let _history_event = HistoryEvent::new(
            ctx,
            "workspace.create".to_owned(),
            "Workspace created".to_owned(),
            &serde_json::json![{ "visibility": ctx.visibility() }],
        )
        .await?;

        // Create an entry in the workspace integrations table by default
        WorkspaceIntegration::new(ctx, None).await?;

        Ok(workspace)
    }

    pub async fn snapshot_for_change_set(
        &self,
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WorkspaceResult<WorkspaceSnapshotSelector> {
        Ok(match self.snapshot_kind() {
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
                WorkspaceSnapshotSelector::LegacySnapshot(Arc::new(
                    WorkspaceSnapshot::find_for_change_set(ctx, change_set_id).await?,
                ))
            }
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
                WorkspaceSnapshotSelector::SplitSnapshot(Arc::new(
                    SplitSnapshot::find_for_change_set(ctx, change_set_id).await?,
                ))
            }
        })
    }

    pub async fn insert_workspace(
        ctx: &DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
        change_set_id: ChangeSetId,
        token: impl AsRef<str>,
        kind: WorkspaceSnapshotSelectorDiscriminants,
    ) -> WorkspaceResult<Self> {
        let name = name.as_ref();
        let token = token.as_ref();
        let version_string = match kind {
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
                WorkspaceSnapshotGraph::current_discriminant().to_string()
            }
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
                SplitSnapshotGraph::current_discriminant().to_string()
            }
        };

        let uses_actions_v2 = ctx
            .services_context()
            .feature_flags_service()
            .feature_is_enabled(&FeatureFlag::ActionsV2);

        let kind = kind.to_string();

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "INSERT INTO workspaces (
                    pk,
                    name,
                    default_change_set_id,
                    uses_actions_v2,
                    snapshot_version,
                    token,
                    snapshot_kind
                ) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *",
                &[
                    &pk,
                    &name,
                    &change_set_id,
                    &uses_actions_v2,
                    &version_string,
                    &token,
                    &kind,
                ],
            )
            .await?;

        Self::try_from(row)
    }

    pub async fn new_from_builtin(
        ctx: &mut DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
        token: impl AsRef<str>,
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
            DEFAULT_CHANGE_SET_NAME,
            Some(builtin.default_change_set_id),
            workspace_snapshot.id().await,
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
        )
        .await?;
        let change_set_id = change_set.id;

        let new_workspace = Self::insert_workspace(
            ctx,
            pk,
            name,
            change_set_id,
            &token,
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
        )
        .await?;
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

    pub async fn get_by_pk_opt(
        ctx: &DalContext,
        pk: WorkspacePk,
    ) -> WorkspaceResult<Option<Workspace>> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(WORKSPACE_GET_BY_PK, &[&pk])
            .await?;

        if let Some(row) = row {
            Ok(Some(Self::try_from(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_by_pk(ctx: &DalContext, pk: WorkspacePk) -> WorkspaceResult<Workspace> {
        Self::get_by_pk_opt(ctx, pk)
            .await?
            .ok_or(WorkspaceError::WorkspaceNotFound(pk))
    }

    pub async fn generate_export_data(
        &self,
        ctx: &DalContext,
        workspace_version: &str,
    ) -> WorkspaceResult<WorkspaceExport> {
        let mut content_hashes = vec![];
        let mut change_sets: HashMap<Ulid, Vec<WorkspaceExportChangeSetV0>> = HashMap::new();
        let mut default_change_set_base = Ulid::nil();
        for change_set in ChangeSet::list_active(ctx).await? {
            let snap = WorkspaceSnapshot::find_for_change_set(ctx, change_set.id).await?;

            // From root, get every value from every node, store with hash
            let mut queue = VecDeque::from([snap.root().await?]);

            while let Some(this_node_id) = queue.pop_front() {
                // Queue contents
                content_hashes.extend(
                    snap.get_node_weight(this_node_id)
                        .await?
                        .content_store_hashes(),
                );

                let children = snap
                    .edges_directed(this_node_id, Direction::Outgoing)
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

        let (content_store_values, _) = serialize::to_vec(&store_values_map)?;

        let created_by = if let HistoryActor::User(user_pk) = ctx.history_actor() {
            let user = User::get_by_pk(ctx, *user_pk).await?;

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
        ctx: &mut DalContext,
        workspace_data: WorkspaceExport,
    ) -> WorkspaceResult<()> {
        let WorkspaceExportContentV0 {
            change_sets,
            content_store_values,
            metadata,
        } = workspace_data.into_latest();

        let workspace = ctx.get_workspace().await?;

        // ABANDON PREVIOUS CHANGESETS
        for mut change_set in ChangeSet::list_active(ctx).await? {
            change_set.abandon(ctx).await?;
        }

        let base_changeset_for_default = {
            let changeset_id = self.default_change_set_id();

            let changeset = ChangeSet::get_by_id(ctx, changeset_id).await?;

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
                )?;

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

                let new_snap_address = imported_snapshot.write(ctx).await?;

                let new_change_set = ChangeSet::new(
                    ctx,
                    change_set_data.name.clone(),
                    actual_base_changeset,
                    new_snap_address,
                    workspace.snapshot_kind(),
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
                .write(content, None, ctx.events_tenancy(), ctx.events_actor())?;
        }

        Ok(())
    }

    getter!(name, String);

    pub async fn has_change_set(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
    ) -> WorkspaceResult<bool> {
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT count(*) > 0 AS has_change_set FROM change_set_pointers WHERE workspace_id = $1 AND id = $2",
                &[&ctx.tenancy().workspace_pk_opt(), &change_set_id],
            )
            .await?;
        let has_change_set: bool = row.try_get("has_change_set")?;

        Ok(has_change_set)
    }

    pub async fn default_change_set(&self, ctx: &DalContext) -> WorkspaceResult<ChangeSet> {
        ChangeSet::find(ctx, self.default_change_set_id)
            .await?
            .ok_or_else(|| {
                WorkspaceError::DefaultChangeSetNotFound(self.pk, self.default_change_set_id)
            })
    }

    pub fn component_concurrency_limit(&self) -> i32 {
        self.component_concurrency_limit
            .unwrap_or(DEFAULT_COMPONENT_CONCURRENCY_LIMIT)
    }

    pub fn raw_component_concurrency_limit(&self) -> Option<i32> {
        self.component_concurrency_limit
    }

    pub async fn set_component_concurrency_limit(
        &mut self,
        ctx: &DalContext,
        limit: Option<i32>,
    ) -> WorkspaceResult<()> {
        let limit = match limit {
            Some(limit) if limit <= 0 => None,
            other => other,
        };

        ctx.txns()
            .await?
            .pg()
            .query_none(
                "UPDATE workspaces SET component_concurrency_limit = $2 WHERE pk = $1",
                &[&self.pk, &limit],
            )
            .await?;

        self.component_concurrency_limit = limit;

        Ok(())
    }

    pub async fn set_snapshot_versions(
        &mut self,
        ctx: &DalContext,
        snapshot_version: SnapshotVersion,
        subgraph_version: Option<SubGraphVersionDiscriminants>,
    ) -> WorkspaceResult<()> {
        let version_string = snapshot_version.db_string();
        let subgraph_version_string = subgraph_version.map(|v| v.to_string());
        let snapshot_kind = match snapshot_version {
            SnapshotVersion::Legacy(_) => WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot,
            SnapshotVersion::Split(_) => WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot,
        };
        ctx.txns()
            .await?
            .pg()
            .query(
                "UPDATE workspaces SET snapshot_kind=$2, snapshot_version=$3, subgraph_version=$4 WHERE pk = $1",
                &[
                    &self.pk,
                    &(snapshot_kind.to_string()),
                    &version_string,
                    &subgraph_version_string
                ],
            )
            .await?;

        self.snapshot_version = snapshot_version;
        self.subgraph_version = subgraph_version;
        self.snapshot_kind = snapshot_kind;

        Ok(())
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }
}
