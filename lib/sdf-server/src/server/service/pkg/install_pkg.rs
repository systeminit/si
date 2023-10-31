use std::str::FromStr;

use super::PkgResult;
use crate::server::extract::RawAccessToken;
use crate::server::tracking::track;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::pkg::PkgError,
};
use axum::extract::OriginalUri;
use axum::Json;
use dal::pkg::ModuleImported;
use dal::WorkspacePk;
use dal::{pkg::import_pkg_from_pkg, Visibility, WsEvent};
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::{SiPkg, SiPkgKind};
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgRequest {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallPkgResponse {
    pub success: bool,
    pub skipped_attributes: bool,
    pub skipped_edges: bool,
}

pub async fn install_pkg(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<InstallPkgRequest>,
) -> PkgResult<Json<InstallPkgResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(PkgError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(pkg_data)?;
    let metadata = pkg.metadata()?;
    let (_, svs, import_skips) = import_pkg_from_pkg(&ctx, &pkg, None).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "install_pkg",
        serde_json::json!({
                    "pkg_name": metadata.name().to_owned(),
        }),
    );

    WsEvent::module_imported(
        &ctx,
        match metadata.kind() {
            SiPkgKind::Module => ModuleImported::Module {
                schema_variant_ids: svs,
            },
            SiPkgKind::WorkspaceBackup => {
                let workspace_pk = match metadata.workspace_pk() {
                    Some(workspace_pk) => Some(WorkspacePk::from_str(workspace_pk)?),
                    None => None,
                };

                ModuleImported::WorkspaceBackup { workspace_pk }
            }
        },
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    let skipped_edges = import_skips
        .as_ref()
        .is_some_and(|skips| skips.iter().any(|skip| !skip.edge_skips.is_empty()));

    let skipped_attributes = import_skips
        .as_ref()
        .is_some_and(|skips| skips.iter().any(|skip| !skip.attribute_skips.is_empty()));

    Ok(Json(InstallPkgResponse {
        success: true,
        skipped_edges,
        skipped_attributes,
    }))
}
