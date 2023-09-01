use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    ActionId, ActionKind, ChangeSet, ChangeSetPk, ChangeSetStatus, ComponentId, StandardModel,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    pub id: ActionId,
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
        let a = cs.actions(&ctx).await?;
        let mut actions = Vec::with_capacity(a.len());
        for action in a {
            let prototype = action.prototype(&ctx).await?;
            actions.push(ActionView {
                id: *action.id(),
                name: prototype.name().map_or_else(
                    || match prototype.kind() {
                        ActionKind::Create => "create".to_owned(),
                        ActionKind::Delete => "delete".to_owned(),
                        ActionKind::Other => "other".to_owned(),
                        ActionKind::Refresh => " refresh".to_owned(),
                    },
                    ToOwned::to_owned,
                ),
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
