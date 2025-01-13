use crate::service::force_change_set_response::ForceChangeSetResponse;
use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    diagram::view::ViewId, func::authoring::FuncAuthoringClient,
    management::prototype::ManagementPrototype, schema::variant::authoring::VariantAuthoringClient,
    ChangeSet, ChangeSetId, ComponentId, FuncId, SchemaVariantId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use crate::extract::{HandlerContext, PosthogClient};
use crate::service::v2::AccessBuilder;

use super::{track, ManagementApiError, ManagementApiResult};

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
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<GenerateTemplateRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<GenerateTemplateResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let new_variant = VariantAuthoringClient::create_schema_and_variant(
        &ctx,
        request.asset_name.to_owned(),
        None,
        None,
        request.category,
        request.color,
    )
    .await?;

    let schema_id = new_variant.schema_id(&ctx).await?;

    let func = FuncAuthoringClient::create_new_management_func(
        &ctx,
        Some(request.func_name.clone()),
        new_variant.id(),
    )
    .await?;

    let prototype_id = ManagementPrototype::list_ids_for_func_id(&ctx, func.id)
        .await?
        .pop()
        .ok_or(ManagementApiError::FuncMissingPrototype(func.id))?;

    let (create_operations, managed_schemas) =
        dal::management::generator::generate_template(&ctx, view_id, &request.component_ids)
            .await?;

    let return_value = serde_json::json!({
        "status": "ok",
        "message": format!("created {}", &request.asset_name),
        "ops": {
            "create": create_operations,
        }
    });

    let return_value_string = serde_json::to_string_pretty(&return_value)?;
    let formatted = format_code(&return_value_string, 4, 1);

    let code = format!(
        r#"async function main({{
    thisComponent,
    components
}}: Input): Promise < Output > {{
    return {};
}}
"#,
        formatted
    );

    FuncAuthoringClient::save_code(&ctx, func.id, code).await?;

    let prototype = ManagementPrototype::get_by_id(&ctx, prototype_id)
        .await?
        .ok_or(ManagementApiError::FuncMissingPrototype(func.id))?;

    prototype
        .set_managed_schemas(&ctx, Some(managed_schemas))
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

const MAX_DEPTH: usize = 2048;
fn format_code(input: &str, tab_size: usize, initial_depth: usize) -> String {
    let (formatted, _) = input.lines().fold(
        (String::new(), initial_depth),
        |(formatted, mut depth), next_line| {
            if formatted.is_empty() {
                (next_line.to_string(), depth)
            } else {
                if formatted.ends_with("{") {
                    depth = depth.saturating_add(1);
                } else if !formatted.ends_with(",") {
                    depth = depth.saturating_sub(1);
                }

                // prevent panics from massive repeat allocations
                if depth > MAX_DEPTH {
                    depth = MAX_DEPTH;
                }

                (
                    format!(
                        "{formatted}\n{}{}",
                        " ".repeat(depth.saturating_mul(tab_size)),
                        next_line.trim()
                    ),
                    depth,
                )
            }
        },
    );

    formatted
}
