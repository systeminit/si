use crate::service::force_change_set_response::ForceChangeSetResponse;
use axum::{
    extract::{Host, OriginalUri, Path},
    Json,
};
use dal::{
    diagram::view::ViewId, func::authoring::FuncAuthoringClient,
    management::prototype::ManagementPrototype, schema::variant::authoring::VariantAuthoringClient,
    ChangeSet, ChangeSetId, ComponentId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};

use crate::extract::{AccessBuilder, HandlerContext, PosthogClient};

use super::{ManagementApiError, ManagementApiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateTemplateRequest {
    component_ids: Vec<ComponentId>,
    asset_name: String,
    category: String,
    color: String,
}

pub async fn generate_template(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<GenerateTemplateRequest>,
) -> ManagementApiResult<ForceChangeSetResponse<()>> {
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
        Some(request.asset_name.to_owned()),
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
        "message": format!("created {}", request.asset_name),
        "ops": {
            "create": create_operations,
        }
    });

    let return_value_string = serde_json::to_string_pretty(&return_value)?;

    let code = format!(
        r#"async function main({{ thisComponent, components }}: Input): Promise<Output> {{
  return {};
}}
"#,
        return_value_string
    );

    FuncAuthoringClient::save_code(&ctx, func.id, code).await?;

    let prototype = ManagementPrototype::get_by_id(&ctx, prototype_id)
        .await?
        .ok_or(ManagementApiError::FuncMissingPrototype(func.id))?;

    prototype
        .set_managed_schemas(&ctx, Some(managed_schemas))
        .await?;

    WsEvent::schema_variant_created(&ctx, schema_id, new_variant.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
