use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionId, SchemaVariantDefinitionJson,
        SchemaVariantDefinitionMetadataJson,
    },
    schema::SchemaUiMenu,
    Schema, SchemaVariant, SchemaVariantId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantDefResponse {
    pub id: SchemaVariantId,
    pub success: bool,
}

pub async fn exec_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ExecVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<ExecVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let metadata: SchemaVariantDefinitionMetadataJson = variant_def.clone().into();
    let definition: SchemaVariantDefinitionJson = variant_def.try_into()?;

    let mut schema = Schema::new(&ctx, metadata.name.clone(), &metadata.component_kind)
        .await
        .map_err(|e| {
            SchemaVariantDefinitionError::CouldNotCreateSchemaVariantFromDefinition(e.to_string())
        })?;

    let ui_menu_name = match metadata.menu_name {
        Some(ref provided_override) => provided_override.to_owned(),
        None => metadata.name.clone(),
    };
    let ui_menu = SchemaUiMenu::new(&ctx, ui_menu_name, &metadata.category)
        .await
        .map_err(|e| {
            SchemaVariantDefinitionError::CouldNotCreateSchemaVariantFromDefinition(e.to_string())
        })?;
    ui_menu.set_schema(&ctx, schema.id()).await.map_err(|e| {
        SchemaVariantDefinitionError::CouldNotCreateSchemaVariantFromDefinition(e.to_string())
    })?;

    let (mut schema_variant, _, _, _, _) =
        SchemaVariant::new_with_definition(&ctx, metadata, definition, "v0")
            .await
            .map_err(|e| {
                SchemaVariantDefinitionError::CouldNotCreateSchemaVariantFromDefinition(
                    e.to_string(),
                )
            })?;

    schema
        .set_default_schema_variant_id(&ctx, Some(*schema_variant.id()))
        .await
        .map_err(|e| {
            SchemaVariantDefinitionError::CouldNotCreateSchemaVariantFromDefinition(e.to_string())
        })?;

    schema_variant.finalize(&ctx, None).await.map_err(|e| {
        SchemaVariantDefinitionError::CouldNotCreateSchemaVariantFromDefinition(e.to_string())
    })?;

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(ExecVariantDefResponse {
        id: *schema_variant.id(),
        success: true,
    }))
}
