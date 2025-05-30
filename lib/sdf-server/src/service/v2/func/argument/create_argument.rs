use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    Func,
    FuncId,
    WorkspacePk,
    WsEvent,
    func::authoring::FuncAuthoringClient,
};
use frontend_types::FuncSummary;
use sdf_extract::change_set::ChangeSetDalContext;
use si_events::audit_log::AuditLogKind;
use si_frontend_types as frontend_types;

use crate::{
    extract::PosthogClient,
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::func::FuncAPIResult,
    },
    track,
};

pub async fn create_func_argument(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, _change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<frontend_types::FuncArgument>,
) -> FuncAPIResult<ForceChangeSetResponse<FuncSummary>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let new_arg = FuncAuthoringClient::create_func_argument(
        ctx,
        func_id,
        request.name.clone(),
        request.kind.into(),
        request.element_kind.map(Into::into),
    )
    .await?;

    let func_summary = Func::get_by_id(ctx, func_id)
        .await?
        .into_frontend_type(ctx)
        .await?;
    ctx.write_audit_log(
        AuditLogKind::CreateFuncArgument {
            func_id: func_summary.func_id,
            func_display_name: func_summary.display_name.clone(),
            func_name: func_summary.name.clone(),
            kind: new_arg.kind.into(),
            element_kind: new_arg.element_kind.map(|a| a.into()),
        },
        request.name,
    )
    .await?;
    WsEvent::func_updated(ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(ctx)
        .await?;

    track(
        &posthog_client,
        ctx,
        &original_uri,
        &host_name,
        "create_func_argument",
        serde_json::json!({
            "how": "/func/create_func_argument",
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
