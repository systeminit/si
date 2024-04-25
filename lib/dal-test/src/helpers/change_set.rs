//! This module provides [`ChangeSetTestHelpers`].

#![warn(
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used
)]

use dal::context::Conflicts;
use dal::{
    ChangeSet, ChangeSetApplyError, ChangeSetError, ChangeSetId, DalContext, TransactionsError,
};
use thiserror::Error;

use crate::helpers::generate_fake_name;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ChangeSetTestHelpersError {
    #[error("base change set not found for change set: {0}")]
    BaseChangeSetNotFound(ChangeSetId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set apply error: {0}")]
    ChangeSetApply(#[from] ChangeSetApplyError),
    #[error("change set not found by id: {0}")]
    ChangeSetNotFound(ChangeSetId),
    #[error("found conflicts after commit: {0:?}")]
    ConflictsFoundAfterCommit(Conflicts),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
}

type ChangeSetTestHelpersResult<T> = Result<T, ChangeSetTestHelpersError>;

/// This unit struct providers helper functions for working with [`ChangeSets`](ChangeSet). It is
/// designed to centralize logic for test authors wishing to commit changes, fork, apply, abandon,
/// etc.
#[derive(Debug)]
pub struct ChangeSetTestHelpers;

impl ChangeSetTestHelpers {
    /// First, this function performs a blocking commit and checks for conflicts (via
    /// [`Self::commit_and_error_on_conflicts`]). Then, it updates the snapshot to the current
    /// visibility.
    pub async fn commit_and_update_snapshot_to_visibility(
        ctx: &mut DalContext,
    ) -> ChangeSetTestHelpersResult<()> {
        Self::commit_and_error_on_conflicts(ctx).await?;
        ctx.update_snapshot_to_visibility().await?;
        Ok(())
    }

    /// Applies the current [`ChangeSet`] to its base [`ChangeSet`]. Then, it updates the snapshot
    /// to the visibility without using an editing [`ChangeSet`]. In other words, the resulting,
    /// snapshot is "HEAD" without an editing [`ChangeSet`].
    pub async fn apply_change_set_to_base(ctx: &mut DalContext) -> ChangeSetTestHelpersResult<()> {
        let applied_change_set = ChangeSet::apply_to_base_change_set(ctx, true).await?;

        Self::commit_and_error_on_conflicts(ctx).await?;

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
    pub async fn fork_from_head_change_set(ctx: &mut DalContext) -> ChangeSetTestHelpersResult<()> {
        Self::fork_from_head_change_set_inner(ctx, generate_fake_name()).await
    }

    /// "Forks" from the "HEAD" [`ChangeSet`], which is the default [`ChangeSet`] of the workspace.
    /// The name of the forked [`ChangeSet`] comes from the corresponding function parameter.
    ///
    /// If you'd like a randomly generated name, use [`Self::fork_from_head_change_set`].
    pub async fn fork_from_head_change_set_with_name(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
    ) -> ChangeSetTestHelpersResult<()> {
        Self::fork_from_head_change_set_inner(ctx, name).await
    }

    async fn fork_from_head_change_set_inner(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
    ) -> ChangeSetTestHelpersResult<()> {
        let new_change_set = ChangeSet::fork_head(ctx, name).await?;

        ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
            .await?;

        Ok(())
    }

    async fn commit_and_error_on_conflicts(ctx: &DalContext) -> ChangeSetTestHelpersResult<()> {
        match ctx.blocking_commit().await? {
            Some(conflicts) => Err(ChangeSetTestHelpersError::ConflictsFoundAfterCommit(
                conflicts,
            )),
            None => Ok(()),
        }
    }
}
