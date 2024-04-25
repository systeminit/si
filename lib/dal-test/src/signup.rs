//! This module contains [`WorkspaceSignup`], which is created during
//! [`macro expansion`](crate::expand_helpers) and used for tests that need the [`Workspace`],
//! [`User`] and/or [`KeyPair`].

use dal::{DalContext, HistoryActor, KeyPair, User, UserPk, Workspace, WorkspacePk};
use serde::{Deserialize, Serialize};

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
    /// Creates a [`Workspace`] with associated items used during
    /// [`macro expansion`](crate::expand_helpers).
    pub async fn new(
        ctx: &mut DalContext,
        workspace_name: impl AsRef<str>,
        user_name: impl AsRef<str>,
        user_email: impl AsRef<str>,
    ) -> color_eyre::Result<Self> {
        let workspace = Workspace::new(ctx, WorkspacePk::generate(), workspace_name).await?;
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
