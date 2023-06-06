use super::{get_new_pkg_path, PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{SchemaVariantId, Visibility, WsEvent};
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPkgRequest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub schema_variants: Vec<SchemaVariantId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportPkgResponse {
    pub success: bool,
    pub full_path: String,
}

pub async fn export_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExportPkgRequest>,
) -> PkgResult<Json<ExportPkgResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if request.name.trim().is_empty() {
        return Err(PkgError::PackageNameEmpty);
    }

    if request.version.trim().is_empty() {
        return Err(PkgError::PackageVersionEmpty);
    }

    if request.schema_variants.is_empty() {
        return Err(PkgError::PackageExportEmpty);
    }

    let module_index_url = match ctx.module_index_url() {
        Some(url) => format!("{url}/modules"),
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    info!("Packaging module");
    let module_payload = dal::pkg::export_pkg_as_bytes(
        &ctx,
        request.name.clone(),
        request.version.clone(),
        request.description,
        "Sally Signup".to_string(),
        request.schema_variants.clone(),
    )
    .await?;

    info!("Building module-index request");
    let request_part = reqwest::multipart::Part::bytes(module_payload).file_name(format!(
        "{}_{}.tar",
        request.name.trim(),
        request.version.trim()
    ));
    let module_index_uploader = reqwest::Client::new();
    info!("Uploading to module-index");
    let upload_response = module_index_uploader
        .post(dbg!(reqwest::Url::parse(&module_index_url))?)
        .multipart(reqwest::multipart::Form::new().part("module bundle", request_part))
        .send()
        .await?;
    // TODO: Actually do something reasonable here.
    let response_status = dbg!(upload_response.status());
    dbg!(upload_response.text().await?);
    if !response_status.is_success() {
        return Err(PkgError::PackageExportEmpty);
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "export_pkg",
        serde_json::json!({
                    "pkg_name": request.name,
                    "pkg_version": request.version,
                    "pkg_schema_count": request.schema_variants.len(),
        }),
    );

    // TODO: Is this really the WsEvent we want to send right now?
    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(ExportPkgResponse {
        success: true,
        full_path: "Get this from module-index service".to_owned(),
    }))
}
