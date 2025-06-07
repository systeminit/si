use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
        Query,
    },
};
use dal::{
    ChangeSetId,
    WorkspacePk,
};
use module_index_client::ModuleIndexClient;
use serde::{
    Deserialize,
    Serialize,
};
use si_pkg::SiPkg;
use ulid::Ulid;

use super::{
    ModuleAPIResult,
    ModulesAPIError,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
        request::RawAccessToken,
    },
    service::v2::AccessBuilder,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetRemoteModuleDetailsRequest {
    pub id: Ulid,
}

pub type RemoteModuleDetailsResponse = si_pkg::PkgSpec;

#[allow(clippy::too_many_arguments)]
pub async fn remote_module_by_id(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Query(request): Query<GetRemoteModuleDetailsRequest>,
) -> ModuleAPIResult<Json<RemoteModuleDetailsResponse>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModulesAPIError::ModuleIndexNotConfigured),
    };

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(&pkg_data)?;
    let spec = pkg.to_spec().await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "remote_module_spec",
        serde_json::json!({
                    "pkg_name": &spec.name,
        }),
    );

    Ok(Json(spec))
}
