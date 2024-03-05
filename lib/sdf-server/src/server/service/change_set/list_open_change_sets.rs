use axum::Json;
use chrono::{DateTime, Utc};
//use dal::action::ActionId;
use dal::change_set_pointer::{ChangeSetPointer, ChangeSetPointerId};
use dal::ActionKind;
use dal::{ActionPrototypeId, ChangeSetStatus, ComponentId, UserPk};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionView {
    // FIXME(nick,zack,jacob): drop ActionId since it does not exist yet for the graph switchover.
    pub id: Ulid,
    pub action_prototype_id: ActionPrototypeId,
    pub kind: ActionKind,
    pub name: String,
    pub component_id: ComponentId,
    pub actor: Option<String>,
    pub parents: Vec<()>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetView {
    // TODO: pk and id are now identical and one of them should be removed
    pub id: ChangeSetPointerId,
    pub pk: ChangeSetPointerId,
    pub name: String,
    pub status: ChangeSetStatus,
    pub merge_requested_at: Option<DateTime<Utc>>,
    pub base_change_set_id: ChangeSetPointerId,
    pub merge_requested_by_user_id: Option<UserPk>,
    pub abandon_requested_at: Option<DateTime<Utc>>,
    pub abandon_requested_by_user_id: Option<UserPk>,
}

pub type ListOpenChangeSetsResponse = Vec<ChangeSetView>;

pub async fn list_open_change_sets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> ChangeSetResult<Json<ListOpenChangeSetsResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let list = ChangeSetPointer::list_open(&ctx).await?;
    let mut view = Vec::with_capacity(list.len());
    for cs in list {
        view.push(ChangeSetView {
            // TODO: remove change sets entirely!
            id: cs.id,
            pk: cs.id,
            name: cs.name,
            status: cs.status,
            base_change_set_id: cs.base_change_set_id.unwrap_or_default(),
            merge_requested_at: None,           // cs.merge_requested_at,
            merge_requested_by_user_id: None,   // cs.merge_requested_by_user_id,
            abandon_requested_at: None,         // cs.abandon_requested_at,
            abandon_requested_by_user_id: None, // cs.abandon_requested_by_user_id,
        });
    }

    Ok(Json(view))
}
