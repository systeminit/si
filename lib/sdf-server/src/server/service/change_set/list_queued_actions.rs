use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::change_set::ChangeSetError;
use axum::extract::Query;
use axum::Json;
use dal::{
    action::ActionBag, history_event, ActionId, ActionKind, ActionPrototypeId, ActorView,
    ChangeSet, ComponentId, Func, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub id: ActionId,
    pub action_prototype_id: ActionPrototypeId,
    pub kind: ActionKind,
    pub name: String,
    pub component_id: ComponentId,
    pub actor: Option<String>,
    pub parents: Vec<ActionId>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQueuedActionsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQueuedActionsResponse {
    pub actions: HashMap<ActionId, ActionView>,
}

pub async fn list_queued_actions(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Query(request): Query<ListQueuedActionsRequest>,
) -> ChangeSetResult<Json<ListQueuedActionsResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set = ChangeSet::get_by_pk(&ctx, &request.visibility.change_set_pk)
        .await?
        .ok_or(ChangeSetError::ChangeSetNotFound)?;

    let ctx =
        ctx.clone_with_new_visibility(Visibility::new(change_set.pk, ctx.visibility().deleted_at));
    let mut actions = HashMap::new();
    for (
        _,
        ActionBag {
            action,
            parents,
            kind,
        },
    ) in change_set.actions(&ctx).await?
    {
        let mut display_name = None;
        let prototype = action.prototype(&ctx).await?;
        let func_details = Func::get_by_id(&ctx, &prototype.func_id()).await?;
        if let Some(func) = func_details {
            if func.display_name().is_some() {
                display_name = func.display_name().map(|dname| dname.to_string());
            }
        }

        let mut actor_email: Option<String> = None;
        {
            if let Some(created_at_user) = action.creation_user_id() {
                let history_actor = history_event::HistoryActor::User(*created_at_user);
                let actor = ActorView::from_history_actor(&ctx, history_actor).await?;
                match actor {
                    ActorView::System { label } => actor_email = Some(label),
                    ActorView::User { label, email, .. } => {
                        if let Some(em) = email {
                            actor_email = Some(em)
                        } else {
                            actor_email = Some(label)
                        }
                    }
                };
            }
        }

        actions.insert(
            *action.id(),
            ActionView {
                id: *action.id(),
                action_prototype_id: *prototype.id(),
                kind,
                name: display_name.unwrap_or_else(|| match kind {
                    ActionKind::Create => "create".to_owned(),
                    ActionKind::Delete => "delete".to_owned(),
                    ActionKind::Other => "other".to_owned(),
                    ActionKind::Refresh => "refresh".to_owned(),
                }),
                component_id: *action.component_id(),
                actor: actor_email,
                parents,
            },
        );
    }
    Ok(Json(ListQueuedActionsResponse { actions }))
}
