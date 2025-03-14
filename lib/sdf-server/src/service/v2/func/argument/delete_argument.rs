use axum::extract::{Host, OriginalUri, Path};
use dal::{
    func::{
        argument::{FuncArgument, FuncArgumentId},
        authoring::FuncAuthoringClient,
    },
    ChangeSet, ChangeSetId, Func, FuncId, WorkspacePk, WsEvent,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types::FuncSummary;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
    service::{force_change_set_response::ForceChangeSetResponse, v2::func::FuncAPIResult},
    track,
};

pub async fn delete_func_argument(
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
) -> FuncAPIResult<ForceChangeSetResponse<FuncSummary>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let func_arg = FuncArgument::get_by_id(&ctx, func_argument_id).await?;
    FuncAuthoringClient::delete_func_argument(&ctx, func_argument_id).await?;

    let func_summary = Func::get_by_id(&ctx, func_id)
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
        "delete_func_argument",
        serde_json::json!({
            "how": "/func/delete_func_argument",
            "func_id": func_id,
            "func_name": func_summary.name.clone(),
            "func_kind": func_summary.kind.clone(),
        }),
    );
    ctx.write_audit_log(
        AuditLogKind::DeleteFuncArgument {
            func_id,
            func_display_name: func_summary.display_name.clone(),
            func_name: func_summary.name.clone(),
            func_argument_id,
        },
        func_arg.name,
    )
    .await?;
    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        func_summary,
    ))
}
