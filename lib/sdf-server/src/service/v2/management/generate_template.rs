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
    ComponentId,
    FuncId,
    SchemaVariantId,
    WorkspacePk,
    WsEvent,
    diagram::view::ViewId,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;

use super::{
    ManagementApiResult,
    track,
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
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateRequest {
    component_ids: Vec<ComponentId>,
    asset_name: String,
    func_name: String,
    category: String,
    color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateResponse {
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
}

pub async fn generate_template(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, _view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<GenerateTemplateRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<GenerateTemplateResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let (new_variant, schema_id, func, prototype_id) =
        sdf_core::generate_template::prepare_and_generate(
            &ctx,
            request.component_ids,
            request.asset_name.clone(),
            request.func_name.clone(),
            request.category.clone(),
            request.color.clone(),
        )
        .await?;

    let schema_variant_id = new_variant.id();
    WsEvent::schema_variant_created(&ctx, schema_id, new_variant.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    WsEvent::template_generated(
        &ctx,
        schema_id,
        schema_variant_id,
        func.id,
        request.asset_name.clone(),
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "generate_template",
        serde_json::json!({
            "generated_schema_variant_id": new_variant.id,
            "generated_prototype_id": prototype_id,
            "generated_func_id": func.id,
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::GenerateTemplate {
            schema_variant_id: new_variant.id,
            management_prototype_id: prototype_id,
            func_id: func.id,
            func_name: request.func_name,
            asset_name: request.asset_name.to_owned(),
        },
        request.asset_name,
    )
    .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        GenerateTemplateResponse {
            schema_variant_id: new_variant.id,
            func_id: func.id,
        },
    ))
}
