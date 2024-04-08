use super::ModuleResult;
use crate::server::extract::RawAccessToken;
use crate::server::tracking::track;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::module::ModuleError,
};
use axum::extract::{OriginalUri, Query};
use axum::Json;
use dal::Visibility;
use module_index_client::IndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RemoteModuleDetailsRequest {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub type RemoteModuleDetailsResponse = si_pkg::PkgSpec;

pub async fn remote_module_spec(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<RemoteModuleDetailsRequest>,
) -> ModuleResult<Json<RemoteModuleDetailsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(pkg_data)?;
    let spec = pkg.to_spec().await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "remote_module_spec",
        serde_json::json!({
                    "pkg_name": &spec.name,
        }),
    );

    Ok(Json(spec))
}
