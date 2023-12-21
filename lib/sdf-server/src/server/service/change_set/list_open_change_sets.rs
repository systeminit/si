use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{ChangeSet, ChangeSetPk, ChangeSetStatus};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub struct ChangeSetView {
    pub pk: ChangeSetPk,
    pub name: String,
    pub status: ChangeSetStatus,
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
        view.push(ChangeSetView {
            pk: cs.pk,
            name: cs.name,
            status: cs.status,
        });
    }

    Ok(Json(view))
}
