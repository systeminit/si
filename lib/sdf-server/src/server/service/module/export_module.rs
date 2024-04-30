use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use crate::service::module::{ModuleError, ModuleResult};
use axum::extract::OriginalUri;
use axum::Json;
use dal::pkg::export::PkgExporter;
use dal::{HistoryActor, SchemaVariant, SchemaVariantId, User, Visibility};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportModuleRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_variants: Vec<SchemaVariantId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportModuleResponse {
    pub success: bool,
    pub full_path: String,
}

pub async fn export_module(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExportModuleRequest>,
) -> ModuleResult<Json<ExportModuleResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if request.name.trim().is_empty() {
        return Err(ModuleError::PackageNameEmpty);
    }

    if request.version.trim().is_empty() {
        return Err(ModuleError::PackageVersionEmpty);
    }

    if request.schema_variants.is_empty() {
        return Err(ModuleError::PackageExportEmpty);
    }

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured),
    };

    let user = match ctx.history_actor() {
        HistoryActor::User(user_pk) => User::get_by_pk(&ctx, *user_pk).await?,
        _ => None,
    };

    let (created_by_name, created_by_email) = user
        .map(|user| (user.name().to_owned(), user.email().to_owned()))
        .unwrap_or((
            "unauthenticated user name".into(),
            "unauthenticated user email".into(),
        ));

    info!("Packaging module");

    // XXX:rework frontend to send schema ids
    let mut schema_ids = vec![];
    for variant_id in &request.schema_variants {
        let schema = SchemaVariant::get_by_id(&ctx, *variant_id)
            .await?
            .schema(&ctx)
            .await?;
        schema_ids.push(schema.id());
    }

    let mut exporter = PkgExporter::new_module_exporter(
        &request.name,
        &request.version,
        request.description.as_ref(),
        &created_by_email,
        schema_ids,
        ctx.get_workspace_default_change_set_id().await?,
    );

    let module_payload = exporter.export_as_bytes(&ctx).await?;

    let index_client =
        module_index_client::IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let response = index_client
        .upload_module(request.name.trim(), request.version.trim(), module_payload)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "export_module",
        serde_json::json!({
                    "pkg_name": request.name,
                    "pkg_version": request.version,
                    "pkg_description": request.description,
                    "pkg_created_by_name": created_by_name,
                    "pkg_created_by_email": created_by_email,
                    "pkg_schema_count": request.schema_variants.len(),
                    "pkg_hash": response.latest_hash,
        }),
    );

    ctx.commit().await?;

    Ok(Json(ExportModuleResponse {
        success: true,
        full_path: "Get this from module-index service".to_owned(),
    }))
}
