use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{ActionPrototypeId, ChangeSet, ChangeSetPk, ChangeSetStatus, ComponentId};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    // FIXME(nick,zack,jacob): drop ActionId since it does not exist yet for the graph switchover.
    pub id: Ulid,
    pub action_prototype_id: ActionPrototypeId,
    pub name: String,
    pub component_id: ComponentId,
    pub actor: Option<String>,
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
        // let ctx =
        //     ctx.clone_with_new_visibility(Visibility::new(cs.pk, ctx.visibility().deleted_at));

        let mut actions = Vec::new();

        view.push(ChangeSetView {
            pk: cs.pk,
            name: cs.name,
            status: cs.status,
            actions,
        });
    }

    Ok(Json(view))
}
