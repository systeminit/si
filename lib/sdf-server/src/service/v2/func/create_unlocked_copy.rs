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
    SchemaVariant,
    SchemaVariantId,
    WorkspacePk,
    func::authoring::FuncAuthoringClient,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_types::{
    FuncCode,
    FuncSummary,
};

use super::{
    FuncAPIError,
    FuncAPIResult,
    get_code_response,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnlockFuncRequest {
    pub schema_variant_id: Option<SchemaVariantId>,
}
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    summary: FuncSummary,
    code: FuncCode,
}

pub async fn create_unlocked_copy(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, func_id)): Path<(WorkspacePk, ChangeSetId, FuncId)>,
    Json(request): Json<UnlockFuncRequest>,
) -> FuncAPIResult<ForceChangeSetResponse<CreateFuncResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let existing_func = Func::get_by_id(&ctx, func_id).await?;
    if !existing_func.is_locked {
        return Err(FuncAPIError::FuncAlreadyUnlocked(func_id));
    }

    let new_func =
        FuncAuthoringClient::create_unlocked_func_copy(&ctx, func_id, request.schema_variant_id)
            .await?;
    let code = get_code_response(&ctx, new_func.id).await?;
    let summary = FuncAuthoringClient::publish_func_create_event(&ctx, &new_func).await?;

    let variant = if let Some(schema_variant_id) = request.schema_variant_id {
        SchemaVariant::get_by_id_opt(&ctx, schema_variant_id).await?
    } else {
        None
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "unlocked_func",
        serde_json::json!({
            "how": "/func/unlocked_func",
            "func_id": summary.func_id,
            "func_name": summary.name.to_owned(),
            "func_kind": summary.kind,
        }),
    );
    ctx.write_audit_log(
        AuditLogKind::UnlockFunc {
            func_id,
            func_display_name: summary.display_name.clone(),
            schema_variant_id: request.schema_variant_id,
            component_id: None,
            subject_name: variant.map(|v| v.display_name().to_owned()),
        },
        summary.name.clone(),
    )
    .await?;
    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        CreateFuncResponse { summary, code },
    ))
}
