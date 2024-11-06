use crate::{ViewResult, ViewView};
use axum::extract::{Json, Path};
use axum_util::extract::{AccessBuilder, HandlerContext};
use dal::{diagram::view::View, ChangeSetId, Visibility, WorkspacePk};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type Response = Vec<ViewView>;

pub async fn list_views(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> ViewResult<Json<Response>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let mut views = vec![];
    for view in View::list(&ctx).await? {
        views.push(ViewView::from_view(&ctx, view).await?);
    }

    Ok(Json(views))
}
