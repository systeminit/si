use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    func::argument::{FuncArgument, FuncArgumentId},
    ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk, WsEvent,
};
use frontend_types::FuncSummary;
use si_frontend_types as frontend_types;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    service::{force_change_set_response::ForceChangeSetResponse, v2::func::FuncAPIResult},
    track,
};

pub async fn update_func_argument(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id, func_argument_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        FuncId,
        FuncArgumentId,
    )>,
    Json(request): Json<frontend_types::FuncArgument>,
) -> FuncAPIResult<ForceChangeSetResponse<FuncSummary>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    FuncArgument::modify_by_id(&ctx, func_argument_id, |existing_arg| {
        existing_arg.name = request.name;
        existing_arg.kind = request.kind.into();
        existing_arg.element_kind = request.element_kind.map(Into::into);
        Ok(())
    })
    .await?;

    let func_summary = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    WsEvent::func_updated(&ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "update_func_argument",
        serde_json::json!({
            "how": "/func/update_func_argument",
            "func_id": func_id,
            "func_name": func_summary.name.clone(),
            "func_kind": func_summary.kind.clone(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        func_summary,
    ))
}
