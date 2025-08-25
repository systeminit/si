use std::collections::{
    HashMap,
    HashSet,
};

use dal::{
    DalContext,
    action::Action,
};
use si_frontend_mv_types::action::action_diff_list::{
    ActionDiffList as ActionDiffListMv,
    ActionDiffStatus,
    ActionDiffView,
};
use si_id::ActionId;
use telemetry::prelude::*;

#[instrument(
    name = "dal_materialized_views.action_diff_list",
    level = "debug",
    skip_all
)]
pub async fn assemble(new_ctx: DalContext) -> crate::Result<ActionDiffListMv> {
    let new_ctx = &new_ctx;
    let old_ctx = new_ctx.clone_with_head().await?;
    let old_ctx = &old_ctx;

    let new_action_ids: HashSet<ActionId> =
        HashSet::from_iter(Action::list_topologically(new_ctx).await?);
    let old_action_ids: HashSet<ActionId> =
        HashSet::from_iter(Action::list_topologically(old_ctx).await?);

    let mut action_diffs: HashMap<ActionId, ActionDiffView> = HashMap::new();
    let only_old_actions: Vec<&ActionId> = old_action_ids.difference(&new_action_ids).collect();
    for old_action in only_old_actions {
        let Some(component_id) = Action::component_id(old_ctx, *old_action).await? else {
            debug!(si.error.message="Found orphaned action while building Diff MV", si.action.id=%old_action.to_string());
            continue;
        };

        let action_diff = ActionDiffView {
            id: *old_action,
            diff_status: ActionDiffStatus::Removed,
            component_id,
        };
        action_diffs.insert(*old_action, action_diff);
    }
    let only_new_actions: Vec<&ActionId> = new_action_ids.difference(&old_action_ids).collect();
    for new_action in only_new_actions {
        let Some(component_id) = Action::component_id(new_ctx, *new_action).await? else {
            debug!(si.error.message="Found orphaned action while building Diff MV", si.action.id=%new_action.to_string());
            continue;
        };
        let state = Action::get_by_id(new_ctx, *new_action).await?.state();
        let action_diff = ActionDiffView {
            id: *new_action,
            diff_status: ActionDiffStatus::Added { new_state: state },
            component_id,
        };
        action_diffs.insert(*new_action, action_diff);
    }

    let actions_in_both: Vec<&ActionId> = old_action_ids.intersection(&new_action_ids).collect();
    for action in actions_in_both {
        let old_state = Action::get_by_id(old_ctx, *action).await?.state();
        let new_state = Action::get_by_id(new_ctx, *action).await?.state();
        let Some(component_id) = Action::component_id(new_ctx, *action).await? else {
            debug!(si.error.message="Found orphaned action while building Diff MV", si.action.id=%action.to_string());
            continue;
        };
        let has_different_state = old_state != new_state;
        let diff_status = match has_different_state {
            true => ActionDiffStatus::Modified {
                old_state,
                new_state,
            },
            false => ActionDiffStatus::None,
        };
        let action_diff = ActionDiffView {
            id: *action,
            diff_status,
            component_id,
        };
        action_diffs.insert(*action, action_diff);
    }

    let id = new_ctx.workspace_pk()?;

    Ok(ActionDiffListMv { id, action_diffs })
}
