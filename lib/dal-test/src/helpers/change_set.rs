//! This module provides [`ChangeSetTestHelpers`].

#![warn(
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use std::time::Duration;

use dal::action::dependency_graph::ActionDependencyGraph;
use dal::action::{Action, ActionState};
use dal::context::Conflicts;
use dal::WorkspaceSnapshotError;
use dal::{
    action::ActionError, ChangeSet, ChangeSetApplyError, ChangeSetError, ChangeSetId, DalContext,
    TransactionsError,
};
use thiserror::Error;

use crate::helpers::generate_fake_name;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ChangeSetTestHelpersError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("timeout waiting for actions to clear from test workspace")]
    ActionTimeout,
    #[error("base change set not found for change set: {0}")]
    BaseChangeSetNotFound(ChangeSetId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] ChangeSetApplyError),
    #[error("change set not found by id: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("found conflicts after apply: {0:?}")]
    ConflictsFoundAfterApply(Conflicts),
    #[error("found conflicts after commit: {0:?}")]
    ConflictsFoundAfterCommit(Conflicts),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

type ChangeSetTestHelpersResult<T> = Result<T, ChangeSetTestHelpersError>;

/// This unit struct providers helper functions for working with [`ChangeSets`](ChangeSet). It is
/// designed to centralize logic for test authors wishing to commit changes, fork, apply, abandon,
/// etc.
#[derive(Debug)]
pub struct ChangeSetTestHelpers;

impl ChangeSetTestHelpers {
    /// First, this function performs a blocking commit which will return an error if
    /// there are conflicts.  Then, it updates the snapshot to the current visibility.
    pub async fn commit_and_update_snapshot_to_visibility(
        ctx: &mut DalContext,
    ) -> ChangeSetTestHelpersResult<()> {
        ctx.commit().await?;
        let total_count = 200;
        let mut count = 0;
        while count < total_count {
            ctx.update_snapshot_to_visibility().await?;
            let mut still_active = ctx
                .workspace_snapshot()?
                .has_dependent_value_roots()
                .await?;
            if !still_active {
                return Ok(());
            }

            count += 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(ChangeSetTestHelpersError::ActionTimeout)
    }

    /// Wait for all actions queued on the workspace snapshot to either succeed (and therefore not
    /// be on the graph), fail, or be put on hold. Will wait for at least 10 seconds, checking every
    /// 100ms.
    pub async fn wait_for_actions_to_run(ctx: &mut DalContext) -> ChangeSetTestHelpersResult<()> {
        let total_count = 100;
        let mut count = 0;

        while count < total_count {
            ctx.update_snapshot_to_visibility().await?;
            let action_graph = ActionDependencyGraph::for_workspace(ctx).await?;
            let mut still_active = false;
            for action_id in action_graph.independent_actions() {
                let a = Action::get_by_id(ctx, action_id).await?;
                match a.state() {
                    ActionState::Dispatched | ActionState::Queued | ActionState::Running => {
                        still_active = true;
                    }
                    ActionState::Failed | ActionState::OnHold => {}
                }
            }
            if !still_active {
                return Ok(());
            }
            count += 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(ChangeSetTestHelpersError::ActionTimeout)
    }

    /// Applies the current [`ChangeSet`] to its base [`ChangeSet`]. Then, it updates the snapshot
    /// to the visibility without using an editing [`ChangeSet`]. In other words, the resulting,
    /// snapshot is "HEAD" without an editing [`ChangeSet`].
    pub async fn apply_change_set_to_base(ctx: &mut DalContext) -> ChangeSetTestHelpersResult<()> {
        let applied_change_set = match ChangeSet::apply_to_base_change_set(ctx).await {
            Err(ChangeSetApplyError::ConflictsOnApply(conflicts)) => Err(
                ChangeSetTestHelpersError::ConflictsFoundAfterApply(conflicts),
            )?,
            err @ Err(_) => err?,
            Ok(change_set) => change_set,
        };

        ctx.commit().await?;

        ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(
            applied_change_set.base_change_set_id.ok_or(
                ChangeSetTestHelpersError::BaseChangeSetNotFound(applied_change_set.id),
            )?,
        )
        .await?;
        Ok(())
    }

    /// Abandons the current [`ChangeSet`].
    pub async fn abandon_change_set(ctx: &mut DalContext) -> ChangeSetTestHelpersResult<()> {
        let mut abandonment_change_set = ChangeSet::find(ctx, ctx.change_set_id()).await?.ok_or(
            ChangeSetTestHelpersError::ChangeSetNotFound(ctx.change_set_id()),
        )?;
        abandonment_change_set.abandon(ctx).await?;
        Ok(())
    }

    /// "Forks" from the "HEAD" [`ChangeSet`], which is the default [`ChangeSet`] of the workspace.
    /// The name of the forked [`ChangeSet`] will be random.
    ///
    /// If you'd like to provide a name, use [`Self::fork_from_head_change_set_with_name`].
    pub async fn fork_from_head_change_set(
        ctx: &mut DalContext,
    ) -> ChangeSetTestHelpersResult<ChangeSet> {
        Self::fork_from_head_change_set_inner(ctx, generate_fake_name()).await
    }

    /// "Forks" from the "HEAD" [`ChangeSet`], which is the default [`ChangeSet`] of the workspace.
    /// The name of the forked [`ChangeSet`] comes from the corresponding function parameter.
    ///
    /// If you'd like a randomly generated name, use [`Self::fork_from_head_change_set`].
    pub async fn fork_from_head_change_set_with_name(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
    ) -> ChangeSetTestHelpersResult<ChangeSet> {
        Self::fork_from_head_change_set_inner(ctx, name).await
    }

    async fn fork_from_head_change_set_inner(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
    ) -> ChangeSetTestHelpersResult<ChangeSet> {
        let new_change_set = ChangeSet::fork_head(ctx, name).await?;

        ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
            .await?;

        Ok(new_change_set)
    }
}
