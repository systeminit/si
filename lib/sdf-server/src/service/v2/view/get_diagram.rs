use crate::extract::{AccessBuilder, HandlerContext};
use crate::service::v2::view::{ViewError, ViewResult};
use axum::extract::{Json, Path};
use dal::diagram::view::{View, ViewId, ViewView};
use dal::diagram::Diagram;
use dal::{slow_rt, ChangeSetId, WorkspacePk};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    view: ViewView,
    diagram: Diagram,
}

pub async fn get_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
) -> ViewResult<Json<Response>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let view = View::get_by_id(&ctx, view_id).await?;

    let ctx_clone = ctx.clone();
    let diagram = slow_rt::spawn(async move {
        let ctx = &ctx_clone;
        Ok::<Diagram, ViewError>(Diagram::assemble(ctx, Some(view_id)).await?)
    })?
    .await??;

    Ok(Json(Response {
        view: ViewView::from_view(&ctx, view).await?,
        diagram,
    }))
}

pub async fn get_default_diagram(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> ViewResult<Json<Response>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let view_id = View::get_id_for_default(&ctx).await?;
    let view = View::get_by_id(&ctx, view_id).await?;

    let ctx_clone = ctx.clone();
    let diagram = slow_rt::spawn(async move {
        let ctx = &ctx_clone;
        Ok::<Diagram, ViewError>(Diagram::assemble_for_default_view(ctx).await?)
    })?
    .await??;

    Ok(Json(Response {
        view: ViewView::from_view(&ctx, view).await?,
        diagram,
    }))
}
