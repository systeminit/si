use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    ActionId, ActionKind, ActionPrototypeId, ChangeSet, ChangeSetPk, ChangeSetStatus, ComponentId,
    Func, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub id: ActionId,
    pub action_prototype_id: ActionPrototypeId,
    pub name: String,
    pub component_id: ComponentId,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct ChangeSetView {
    pub pk: ChangeSetPk,
    pub name: String,
    pub status: ChangeSetStatus,
    pub actions: Vec<ActionView>,
}

pub type ListOpenChangeSetsResponse = Vec<ChangeSetView>;

pub async fn list_open_change_sets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> ChangeSetResult<Json<ListOpenChangeSetsResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let list = ChangeSet::list_open(&ctx).await?;
    let mut view = Vec::with_capacity(list.len());
    for cs in list {
        let ctx =
            ctx.clone_with_new_visibility(Visibility::new(cs.pk, ctx.visibility().deleted_at));
        let a = cs.actions(&ctx).await?;
        let mut actions = Vec::with_capacity(a.len());
        for action in a {
            let mut display_name = None;
            let prototype = action.prototype(&ctx).await?;
            let func_details = Func::get_by_id(&ctx, &prototype.func_id()).await?;
            if let Some(func) = func_details {
                if func.display_name().is_some() {
                    display_name = func.display_name().map(|dname| dname.to_string());
                }
            }
            actions.push(ActionView {
                id: *action.id(),
                action_prototype_id: *prototype.id(),
                name: display_name.unwrap_or_else(|| match prototype.kind() {
                    ActionKind::Create => "create".to_owned(),
                    ActionKind::Delete => "delete".to_owned(),
                    ActionKind::Other => "other".to_owned(),
                    ActionKind::Refresh => "refresh".to_owned(),
                }),
                component_id: *action.component_id(),
            });
        }

        view.push(ChangeSetView {
            pk: cs.pk,
            name: cs.name,
            status: cs.status,
            actions,
        });
    }

    Ok(Json(view))
}
