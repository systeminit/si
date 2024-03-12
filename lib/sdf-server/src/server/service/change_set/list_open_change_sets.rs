use axum::Json;
use dal::change_set_pointer::view::OpenChangeSetsView;

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext};

pub type ListOpenChangeSetsResponse = OpenChangeSetsView;

pub async fn list_open_change_sets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
) -> ChangeSetResult<Json<ListOpenChangeSetsResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let view = OpenChangeSetsView::assemble(&ctx).await?;

    Ok(Json(view))
}
