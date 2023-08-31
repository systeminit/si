use super::{PkgError, PkgResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::server::tracking::track;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{HistoryActor, SchemaVariant, SchemaVariantId, StandardModel, User, Visibility, WsEvent};
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
    RawAccessToken(raw_access_token): RawAccessToken,
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
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
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
        let schema = SchemaVariant::get_by_id(&ctx, variant_id)
            .await?
            .ok_or(PkgError::SchemaVariantNotFound(*variant_id))?
            .schema(&ctx)
            .await?
            .ok_or(PkgError::SchemaNotFoundForVariant(*variant_id))?;
        schema_ids.push(*schema.id());
    }

    let mut exporter = dal::pkg::PkgExporter::new_module_exporter(
        &request.name,
        &request.version,
        request.description.as_ref(),
        &created_by_email,
        schema_ids,
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
        "export_pkg",
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
