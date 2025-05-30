//! This module provides [`ChangeSetTestHelpers`].

use std::time::Duration;

use color_eyre::{
    Result,
    eyre::eyre,
};
use dal::{
    ChangeSet,
    DalContext,
    Func,
    Schema,
    SchemaVariant,
    action::{
        Action,
        ActionState,
        dependency_graph::ActionDependencyGraph,
    },
    workspace_snapshot::{
        DependentValueRoot,
        selector::WorkspaceSnapshotSelectorDiscriminants,
    },
};

use crate::helpers::generate_fake_name;

/// First, this function performs a blocking commit which will return an error if
/// there are conflicts.  Then, it updates the snapshot to the current visibility.
pub async fn commit(ctx: &mut DalContext) -> Result<()> {
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await
}

/// Creates a new fork from head and returns the new `DalContext`. The original context
pub async fn fork(ctx: &DalContext) -> Result<DalContext> {
    let mut ctx = ctx.clone();
    ChangeSetTestHelpers::fork_from_head_change_set(&mut ctx).await?;
    Ok(ctx)
}

/// Applies a forked context to head and re-forks so it can be worked on again.
pub async fn apply_and_refork(ctx: &mut DalContext) -> Result<()> {
    ChangeSetTestHelpers::apply_change_set_to_base(ctx).await?;
    ChangeSetTestHelpers::fork_from_head_change_set(ctx).await?;
    Ok(())
}

/// Applies a forked context to head, consuming it.
pub async fn apply(mut ctx: DalContext) -> Result<()> {
    ChangeSetTestHelpers::apply_change_set_to_base(&mut ctx).await?;
    Ok(())
}

/// Waits for DVU to finish and updates the snapshot
pub async fn wait_for_dvu(ctx: &mut DalContext) -> Result<()> {
    ChangeSetTestHelpers::wait_for_dvu(ctx).await?;
    ctx.update_snapshot_to_visibility().await?;
    Ok(())
}

/// This unit struct provides helper functions for working with [`ChangeSets`](ChangeSet). It is
/// designed to centralize logic for test authors wishing to commit changes, fork, apply, abandon,
/// etc.
#[derive(Debug)]
pub struct ChangeSetTestHelpers;

impl ChangeSetTestHelpers {
    /// First, this function performs a blocking commit which will return an error if
    /// there are conflicts.  Then, it updates the snapshot to the current visibility.
    pub async fn commit_and_update_snapshot_to_visibility(ctx: &mut DalContext) -> Result<()> {
        Self::blocking_commit(ctx).await?;
        ctx.update_snapshot_to_visibility().await?;
        Ok(())
    }

    /// Wait for all actions queued on the workspace snapshot to either succeed (and therefore not
    /// be on the graph), fail, or be put on hold. Will wait for at least 10 seconds, checking every
    /// 100ms.
    pub async fn wait_for_actions_to_run(ctx: &mut DalContext) -> Result<()> {
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
        Err(eyre!(
            "timeout waiting for actions to clear from test workspace"
        ))
    }

    /// Apply Changeset To base Approvals
    pub async fn apply_change_set_to_base_approvals(ctx: &mut DalContext) -> Result<()> {
        ChangeSet::prepare_for_apply(ctx).await?;
        Self::apply_change_set_to_base_approvals_without_prepare_step(ctx).await?;
        Ok(())
    }

    /// Force Apply Changeset To base Approvals
    pub async fn force_apply_change_set_to_base_approvals(ctx: &mut DalContext) -> Result<()> {
        ChangeSet::prepare_for_force_apply(ctx).await?;
        Self::apply_change_set_to_base_approvals_without_prepare_step(ctx).await?;
        Ok(())
    }

    /// Apply the change set to base for the approvals flow, but without performing the "prepare"
    /// step.
    ///
    /// _Warning:_ if you do not know what to choose, use [Self::force_apply_change_set_to_base_approvals]
    /// or [Self::apply_change_set_to_base_approvals] instead of this function. This function
    /// should only be used if you have an alternative "prepare" workflow (e.g. testing fine grained
    /// access control in sdf tests).
    pub async fn apply_change_set_to_base_approvals_without_prepare_step(
        ctx: &mut DalContext,
    ) -> Result<()> {
        Self::commit_and_update_snapshot_to_visibility(ctx).await?;
        Self::apply_change_set_to_base_inner(ctx).await?;
        Ok(())
    }

    async fn has_updates(ctx: &mut DalContext) -> Result<bool> {
        Ok(
            match ctx.get_workspace_or_builtin().await?.snapshot_kind() {
                WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => ctx
                    .change_set()?
                    .detect_updates_that_will_be_applied_legacy(ctx)
                    .await?
                    .is_some_and(|batch| !batch.updates().is_empty()),
                WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => ctx
                    .change_set()?
                    .detect_updates_that_will_be_applied_split(ctx)
                    .await?
                    .is_some_and(|batch| !batch.is_empty()),
            },
        )
    }

    /// Applies the current change set to the base change set, waiting for replays to land on any open change sets.
    pub async fn apply_change_set_to_base_inner(ctx: &mut DalContext) -> Result<bool> {
        let mut open_change_sets = ChangeSet::list_active(ctx)
            .await?
            .iter()
            .map(|change_set| (change_set.id, change_set.updated_at))
            .collect::<Vec<(_, _)>>();

        let had_updates = Self::has_updates(ctx).await?;
        let applied_change_set = ChangeSet::apply_to_base_change_set(ctx).await?;
        ctx.update_visibility_and_snapshot_to_visibility(
            applied_change_set.base_change_set_id.ok_or(eyre!(
                "base change set not found for change set: {}",
                applied_change_set.id
            ))?,
        )
        .await?;

        // Applying to head will replay the changes against any open change
        // sets. We want to be sure that we've waited until those changes are
        // replayed, so we loop here for a little while (up to 10 seconds),
        // waiting for the changes to reach the open change sets.
        if had_updates {
            let mut iters = 0;
            // only do this for 10 seconds
            while !open_change_sets.is_empty() && iters < 1000 {
                let mut updated_sets = vec![];
                for (change_set_id, original_updated_at) in &open_change_sets {
                    if let Some(change_set) = ChangeSet::find(ctx, *change_set_id).await? {
                        if &change_set.updated_at > original_updated_at {
                            updated_sets.push(change_set.id);
                        }
                    } else {
                        // if we couldn't get it remove it so we don't loop forever
                        updated_sets.push(*change_set_id);
                    }
                }
                open_change_sets.retain(|(change_set_id, _)| !updated_sets.contains(change_set_id));
                if open_change_sets.is_empty() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
                iters += 1
            }
        }
        Ok(had_updates)
    }

    /// Applies the current [`ChangeSet`] to its base [`ChangeSet`], leaving ctx pointed at HEAD.
    /// 1. Prepares the apply by locking unlocked variants.
    /// 2. Prepares the apply by Then, it updates the snapshot
    ///    to the visibility without using an editing [`ChangeSet`]. In other words, the resulting,
    ///    snapshot is "HEAD" without an editing [`ChangeSet`].
    ///
    /// Also locks existing editing funcs and schema variants to mimic SDF
    pub async fn apply_change_set_to_base(ctx: &mut DalContext) -> Result<bool> {
        // Lock all unlocked variants
        for schema_id in Schema::list_ids(ctx).await? {
            let schema = Schema::get_by_id(ctx, schema_id).await?;
            let Some(variant) = SchemaVariant::get_unlocked_for_schema(ctx, schema_id).await?
            else {
                continue;
            };

            let variant_id = variant.id();

            variant.lock(ctx).await?;
            schema.set_default_variant_id(ctx, variant_id).await?;
        }
        // Lock all unlocked functions too
        for func in Func::list_for_default_and_editing(ctx).await? {
            if !func.is_locked {
                func.lock(ctx).await?;
            }
        }

        ctx.commit().await?;
        Self::apply_change_set_to_base_inner(ctx).await
    }

    /// Abandons the current [`ChangeSet`].
    pub async fn abandon_change_set(ctx: &mut DalContext) -> Result<()> {
        let mut abandonment_change_set = ChangeSet::get_by_id(ctx, ctx.change_set_id()).await?;
        abandonment_change_set.abandon(ctx).await?;
        Ok(())
    }

    /// "Forks" from the "HEAD" [`ChangeSet`], which is the default [`ChangeSet`] of the workspace.
    /// The name of the forked [`ChangeSet`] will be random.
    ///
    /// If you'd like to provide a name, use [`Self::fork_from_head_change_set_with_name`].
    pub async fn fork_from_head_change_set(ctx: &mut DalContext) -> Result<ChangeSet> {
        Self::fork_from_head_change_set_inner(ctx, generate_fake_name()?).await
    }

    /// "Forks" from the "HEAD" [`ChangeSet`], which is the default [`ChangeSet`] of the workspace.
    /// The name of the forked [`ChangeSet`] comes from the corresponding function parameter.
    ///
    /// If you'd like a randomly generated name, use [`Self::fork_from_head_change_set`].
    pub async fn fork_from_head_change_set_with_name(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
    ) -> Result<ChangeSet> {
        Self::fork_from_head_change_set_inner(ctx, name).await
    }

    async fn fork_from_head_change_set_inner(
        ctx: &mut DalContext,
        name: impl AsRef<str>,
    ) -> Result<ChangeSet> {
        let new_change_set = ChangeSet::fork_head(ctx, name).await?;

        ctx.update_visibility_and_snapshot_to_visibility(new_change_set.id)
            .await?;

        Ok(new_change_set)
    }

    /// Wait for the changeset's DVUs to be completely processed before continuing
    pub async fn wait_for_dvu(ctx: &DalContext) -> Result<()> {
        loop {
            let mut ctx_clone = ctx.clone();
            ctx_clone.update_snapshot_to_visibility().await?;
            if !DependentValueRoot::roots_exist(&ctx_clone).await? {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }

        Ok(())
    }

    async fn blocking_commit(ctx: &DalContext) -> Result<()> {
        // The rebaser has responsibility for executing dvu jobs, so we should
        // prevent them from running in tests
        let has_roots = ctx
            .txns()
            .await?
            .job_queue()
            .clear_dependent_values_jobs()
            .await;
        ctx.blocking_commit().await?;

        // But we have to wait until the dvu jobs complete
        if has_roots {
            Self::wait_for_dvu(ctx).await?;
        }

        Ok(())
    }
}
