//! This module contains [`WorkspaceSignup`], which is created during
//! [`macro expansion`](crate::expand_helpers) and used for tests that need the [`Workspace`],
//! [`User`] and/or [`KeyPair`].

use dal::{
    ChangeSet,
    DalContext,
    KeyPair,
    UserPk,
    Workspace,
    WorkspacePk,
    builtins::func::migrate_intrinsics_no_commit,
    workspace_integrations::WorkspaceIntegration,
    workspace_snapshot::{
        selector::WorkspaceSnapshotSelectorDiscriminants,
        split_snapshot::SplitSnapshot,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    HistoryActor,
    HistoryEvent,
    Tenancy,
    User,
};

/// A wrapper for creating [`Workspaces`](Workspace) for integration tests.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSignup {
    /// The key pair created after creating the workspace.
    pub key_pair: KeyPair,
    /// The user created after creating the workspace.
    pub user: User,
    /// The created workspace.
    pub workspace: Workspace,
}

impl WorkspaceSignup {
    #[allow(unused)]
    async fn new_split_graph_workspace(
        ctx: &mut DalContext,
        pk: WorkspacePk,
        name: impl AsRef<str>,
        token: impl AsRef<str>,
        split_max: usize,
    ) -> color_eyre::Result<Workspace> {
        let workspace_snapshot = SplitSnapshot::initial(ctx, split_max).await?;
        ctx.set_workspace_split_snapshot(workspace_snapshot);

        migrate_intrinsics_no_commit(ctx).await.map_err(Box::new)?;
        crate::test_exclusive_schemas::migrate(ctx).await?;

        let workspace_snapshot_address = ctx
            .workspace_snapshot()?
            .as_split_snapshot()?
            .write(ctx)
            .await?;

        let mut head_change_set =
            ChangeSet::new(ctx, "HEAD", None, workspace_snapshot_address).await?;

        let workspace = Workspace::insert_workspace(
            ctx,
            pk,
            name,
            head_change_set.id,
            token,
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot,
        )
        .await?;
        head_change_set
            .update_workspace_id(ctx, *workspace.pk())
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

    /// Creates a [`Workspace`] with associated items used during
    /// [`macro expansion`](crate::expand_helpers).
    pub async fn new(
        ctx: &mut DalContext,
        workspace_name: impl AsRef<str>,
        user_name: impl AsRef<str>,
        user_email: impl AsRef<str>,
        token: impl AsRef<str>,
    ) -> color_eyre::Result<Self> {
        let use_split = false;

        let workspace = if use_split {
            Self::new_split_graph_workspace(
                ctx,
                WorkspacePk::generate(),
                workspace_name,
                token,
                500,
            )
            .await?
        } else {
            Workspace::new_from_builtin(ctx, WorkspacePk::generate(), workspace_name, token).await?
        };

        let key_pair = KeyPair::new(ctx, "default").await?;

        let user = User::new(
            ctx,
            UserPk::generate(),
            &user_name,
            &user_email,
            None::<&str>,
        )
        .await?;
        ctx.update_history_actor(HistoryActor::User(user.pk()));

        Ok(Self {
            key_pair,
            user,
            workspace,
        })
    }
}
