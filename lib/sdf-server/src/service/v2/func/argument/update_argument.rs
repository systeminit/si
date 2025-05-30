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
    func::{
        argument::{
            FuncArgument,
            FuncArgumentId,
        },
        authoring::FuncAuthoringError,
    },
};
use frontend_types::FuncSummary;
use sdf_extract::change_set::ChangeSetDalContext;
use si_frontend_types as frontend_types;

use crate::{
    extract::PosthogClient,
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::func::{
            FuncAPIError,
            FuncAPIResult,
        },
    },
    track,
};

pub async fn update_func_argument(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, _change_set_id, func_id, func_argument_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        FuncId,
        FuncArgumentId,
    )>,
    Json(request): Json<frontend_types::FuncArgument>,
) -> FuncAPIResult<ForceChangeSetResponse<FuncSummary>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let func = Func::get_by_id(ctx, func_id).await?;

    if func.is_transformation {
        return Err(FuncAPIError::FuncAuthoring(
            FuncAuthoringError::ModifyingTransformationArguments(func_id),
        ));
    }

    FuncArgument::modify_by_id(ctx, func_argument_id, |existing_arg| {
        existing_arg.name = request.name;
        existing_arg.kind = request.kind.into();
        existing_arg.element_kind = request.element_kind.map(Into::into);
        Ok(())
    })
    .await?;

    let func_summary = func.into_frontend_type(ctx).await?;
    WsEvent::func_updated(ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(ctx)
        .await?;

    track(
        &posthog_client,
        ctx,
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
