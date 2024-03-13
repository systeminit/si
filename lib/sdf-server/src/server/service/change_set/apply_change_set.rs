use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set_pointer::ChangeSetPointer;
use dal::{
    action::ActionBag, Action, ActionBatch, ActionError, ActionId, ActionPrototypeId, ActionRunner,
    ActionRunnerId, Component, ComponentId, HistoryActor, User, Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSetPointer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRunnerItem {
    pub id: ActionRunnerId,
    pub action_prototype_id: ActionPrototypeId,
    pub component_id: ComponentId,
    pub parents: Vec<ActionRunnerId>,
}

pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let actions = Action::build_graph(&ctx).await?;
    let mut prototype_by_action_id = HashMap::new();

    let applying_to_head = ctx.parent_is_head().await;
    if applying_to_head {
        for bag in actions.values() {
            prototype_by_action_id.insert(bag.action.id, bag.action.prototype(&ctx).await?.id);
            bag.action.clone().delete(&ctx).await?;
        }
    }

    ctx.blocking_commit().await?;

    let mut change_set = ChangeSetPointer::find(&ctx, request.visibility.change_set_pk.into())
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(&change_set)
        .await?;
    change_set.apply_to_base_change_set(&ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": request.visibility.change_set_pk,
        }),
    );

    let base_change_set_id = change_set
        .base_change_set_id
        .ok_or(ChangeSetError::BaseChangeSetNotFound(change_set.id))?;
    let head = ChangeSetPointer::find(&ctx, base_change_set_id)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    ctx.update_visibility_and_snapshot_to_visibility_no_editing_change_set(&head)
        .await?;
    ctx.update_visibility_and_snapshot_to_visibility(head.id)
        .await?;

    // If head and there are actions to apply
    if applying_to_head && !actions.is_empty() {
        let user = match ctx.history_actor() {
            HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
                .await?
                .ok_or(ChangeSetError::InvalidUser(*user_pk))?,

            HistoryActor::SystemInit => return Err(ChangeSetError::InvalidUserSystemInit),
        };

        // TODO: restore actors of change-set concept
        let actors_delimited_string = String::new();
        let batch = ActionBatch::new(&ctx, user.email(), &actors_delimited_string).await?;
        let mut runners: HashMap<ActionRunnerId, ActionRunnerItem> = HashMap::new();
        let mut runners_by_action: HashMap<ActionId, ActionRunnerId> = HashMap::new();

        let mut values: Vec<ActionBag> = actions.values().cloned().collect();
        values.sort_by_key(|a| a.action.id);

        let mut values: VecDeque<ActionBag> = values.into_iter().collect();

        // Runners have to be created in the order we want to display them in the actions history panel
        // So we do extra work here to ensure the order is the execution order
        'outer: while let Some(bag) = values.pop_front() {
            let prototype_id = *prototype_by_action_id
                .get(&bag.action.id)
                .ok_or(ActionError::PrototypeNotFoundFor(bag.action.id))?;

            let mut parents = Vec::new();
            for parent_id in bag.parents.clone() {
                if let Some(parent_id) = runners_by_action.get(&parent_id) {
                    parents.push(*parent_id);
                } else {
                    values.push_back(bag);
                    continue 'outer;
                }
            }

            let component = Component::get_by_id(&ctx, bag.component_id).await?;
            let runner = ActionRunner::new(
                &ctx,
                batch.id,
                bag.component_id,
                component.name(&ctx).await?,
                prototype_id,
            )
            .await?;
            runners_by_action.insert(bag.action.id, runner.id);

            runners.insert(
                runner.id,
                ActionRunnerItem {
                    id: runner.id,
                    component_id: bag.component_id,
                    action_prototype_id: prototype_id,
                    parents,
                },
            );
        }

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "apply_fix",
            serde_json::json!({
                "fix_batch_id": batch.id,
                "number_of_action_runners_in_batch": runners.len(),
                "action_runners_applied": runners,
            }),
        );

        // TODO: uncomment this
        // ctx.enqueue_job(ActionJob::new(&ctx, runners, batch.id))
        //     .await?;

        ctx.commit().await?;
    }

    // If anything fails with uploading the workspace backup module, just log it. We shouldn't
    // have the change set apply itself fail because of this.
    /*
    tokio::task::spawn(
        super::upload_workspace_backup_module(ctx, raw_access_token)
            .instrument(info_span!("Workspace backup module upload")),
    );
    */
    Ok(Json(ApplyChangeSetResponse {
        change_set: change_set.to_owned(),
    }))
}
