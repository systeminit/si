use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::extract::Query;
use axum::Json;
use dal::{
    deprecated_action::{ActionView, DeprecatedActionBag},
    history_event, ActionId, ActorView, DeprecatedAction, DeprecatedActionKind, Func, Visibility,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListQueuedActionsRequest>,
) -> ChangeSetResult<Json<ListQueuedActionsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut actions = HashMap::new();
    for (
        _,
        DeprecatedActionBag {
            action,
            parents,
            kind,
            component_id,
        },
    ) in DeprecatedAction::build_graph(&ctx).await?
    {
        let mut display_name = None;
        let prototype = action.prototype(&ctx).await?;
        let func = Func::get_by_id_or_error(&ctx, prototype.func_id(&ctx).await?).await?;
        if func.display_name.is_some() {
            display_name = func.display_name.as_ref().map(|dname| dname.to_string());
        }

        let mut actor_email: Option<String> = None;
        {
            if let Some(created_at_user) = action.creation_user_pk {
                let history_actor = history_event::HistoryActor::User(created_at_user);
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
            action.id,
            ActionView {
                id: action.id,
                action_prototype_id: prototype.id,
                kind,
                name: display_name.unwrap_or_else(|| match kind {
                    DeprecatedActionKind::Create => "create".to_owned(),
                    DeprecatedActionKind::Delete => "delete".to_owned(),
                    DeprecatedActionKind::Other => "other".to_owned(),
                    DeprecatedActionKind::Refresh => "refresh".to_owned(),
                }),
                component_id,
                actor: actor_email,
                parents,
            },
        );
    }
    Ok(Json(ListQueuedActionsResponse { actions }))
}
