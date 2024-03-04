use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::service::change_set::ChangeSetError;
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerId};
use dal::ChangeSetStatus;
use serde::{Deserialize, Serialize};
//use telemetry::tracing::{info_span, Instrument, log::warn};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    pub change_set_pk: ChangeSetPointerId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSetPointer,
}

// TODO: This does not handle anything related to actions yet, after the switchover to workspace
//       snapshot graphs.
pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let mut change_set = ChangeSetPointer::find(&ctx, request.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;
    change_set.apply_to_base_change_set(&ctx).await?;
    change_set
        .update_status(&ctx, ChangeSetStatus::Applied)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": request.change_set_pk,
        }),
    );

    ctx.blocking_commit().await?;

    // TODO(nick): restore in new engine.
    // let user = match ctx.history_actor() {
    //     HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk)
    //         .await?
    //         .ok_or(ChangeSetError::InvalidUser(*user_pk))?,

    //     HistoryActor::SystemInit => return Err(ChangeSetError::InvalidUserSystemInit),
    // };

    // if !actions.is_empty() {
    //     let actors_delimited_string = actors.join(",");
    //     let batch = FixBatch::new(&ctx, user.email(), &actors_delimited_string).await?;
    //     let mut fixes: HashMap<FixId, FixItem> = HashMap::new();
    //     let mut fixes_by_action: HashMap<ActionId, FixId> = HashMap::new();

    //     let mut values: Vec<ActionBag> = actions.values().cloned().collect();
    //     values.sort_by_key(|a| *a.action.id());

    //     let mut values: VecDeque<ActionBag> = values.into_iter().collect();

    //     // Fixes have to be created in the order we want to display them in the fix history panel
    //     // So we do extra work here to ensure the order is the execution order
    //     'outer: while let Some(bag) = values.pop_front() {
    //         let mut parents = Vec::new();
    //         for parent_id in bag.parents.clone() {
    //             if let Some(parent_id) = fixes_by_action.get(&parent_id) {
    //                 parents.push(*parent_id);
    //             } else {
    //                 values.push_back(bag);
    //                 continue 'outer;
    //             }
    //         }

    //         let component = Component::get_by_id(
    //             &ctx.clone_with_delete_visibility(),
    //             bag.action.component_id(),
    //         )
    //         .await?
    //         .ok_or_else(|| ComponentError::NotFound(*bag.action.component_id()))?;
    //         let fix = Fix::new(
    //             &ctx,
    //             *batch.id(),
    //             *bag.action.component_id(),
    //             component.name(&ctx).await?,
    //             *bag.action.action_prototype_id(),
    //         )
    //         .await?;
    //         fixes_by_action.insert(*bag.action.id(), *fix.id());

    //         fixes.insert(
    //             *fix.id(),
    //             FixItem {
    //                 id: *fix.id(),
    //                 component_id: *bag.action.component_id(),
    //                 action_prototype_id: *bag.action.action_prototype_id(),
    //                 parents,
    //             },
    //         );
    //     }

    //     track(
    //         &posthog_client,
    //         &ctx,
    //         &original_uri,
    //         "apply_fix",
    //         serde_json::json!({
    //             "fix_batch_id": batch.id(),
    //             "number_of_fixes_in_batch": fixes.len(),
    //             "fixes_applied": fixes,
    //         }),
    //     );

    //     ctx.enqueue_job(FixesJob::new(&ctx, fixes, *batch.id()))
    //         .await?;
    // }

    // ctx.commit().await?;

    // // If anything fails with uploading the workspace backup module, just log it. We shouldn't
    // // have the change set apply itself fail because of this.
    // /*
    // tokio::task::spawn(
    //     super::upload_workspace_backup_module(ctx, raw_access_token)
    //         .instrument(info_span!("Workspace backup module upload")),
    // );
    // */
    Ok(Json(ApplyChangeSetResponse { change_set }))
}
