use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{
    generate_unique_id,
    schema::variant::definition::{
        SchemaVariantDefinition, SchemaVariantDefinitionError as DalSchemaVariantDefinitionError,
        SchemaVariantDefinitionId,
    },
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncError, FuncId, Schema,
    SchemaError, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CloneVariantDefResponse {
    pub id: SchemaVariantDefinitionId,
    pub success: bool,
}

pub async fn clone_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CloneVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<CloneVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    // Generate a unique name and make sure it's not in use
    let mut name;
    loop {
        name = format!("{} Clone {}", variant_def.name(), generate_unique_id(4));
        match Schema::find_by_name(&ctx, &name).await {
            Ok(_) => continue,
            Err(SchemaError::NotFoundByName(_)) | Err(SchemaError::NoDefaultVariant(_)) => break,
            Err(e) => {
                return Err(
                    DalSchemaVariantDefinitionError::CouldNotCheckForDefaultVariant(e.to_string()),
                )?;
            }
        }
    }

    let menu_name = variant_def.menu_name().map(|mn| format!("{mn} Clone"));

    // We need to duplicate the func because variant definitions and their functions have a
    // one-to-one relationship.
    let func = Func::get_by_id(&ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;
    let duplicated_func = func.duplicate(&ctx).await?;

    let variant_def = SchemaVariantDefinition::new(
        &ctx,
        name,
        menu_name,
        variant_def.category().to_string(),
        variant_def.link().map(|l| l.to_string()),
        variant_def.color().to_owned(),
        *variant_def.component_kind(),
        variant_def.description().map(|d| d.to_string()),
        *duplicated_func.id(),
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "clone_variant_def",
        serde_json::json!({
                    "variant_def_name": variant_def.name(),
                    "variant_def_category": variant_def.category(),
                    "variant_def_menu_name": variant_def.menu_name(),
                    "variant_def_id": variant_def.id(),
                    "variant_def_component_type": variant_def.component_type(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(CloneVariantDefResponse {
        id: *variant_def.id(),
        success: true,
    }))
}
