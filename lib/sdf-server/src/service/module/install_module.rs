use axum::{
    extract::{Host, OriginalUri},
    response::IntoResponse,
    Json,
};
use dal::{
    pkg::{import_pkg_from_pkg, ImportOptions},
    ChangeSet, Func, SchemaVariant, Visibility, WsEvent,
};
use module_index_client::ModuleIndexClient;
use serde::{Deserialize, Serialize};
use si_pkg::SiPkg;
use ulid::Ulid;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken},
    service::module::ModuleError,
    track,
};

use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallModuleRequest {
    pub ids: Vec<Ulid>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[allow(clippy::panic)]
pub async fn install_module(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<InstallModuleRequest>,
) -> Result<impl IntoResponse, ModuleError> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured),
    };

    let mut variants = Vec::new();

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token);
    for id in request.ids {
        let module_details = module_index_client.module_details(id).await?;
        let pkg_data = module_index_client.download_module(id).await?;

        let pkg = SiPkg::load_from_bytes(pkg_data)?;

        let (schema_id, past_module_hashes) = if pkg.schemas()?.len() > 1 {
            (None, None)
        } else {
            (
                module_details.schema_id().map(Into::into),
                module_details.past_hashes,
            )
        };
        let metadata = pkg.metadata()?;
        let (_, svs, _) = match import_pkg_from_pkg(
            &ctx,
            &pkg,
            Some(ImportOptions {
                schema_id,
                past_module_hashes,
                ..Default::default()
            }),
        )
        .await
        {
            Ok(details) => details,
            Err(err) => {
                error!(si.error.message = ?err, "Cannot install pkg");
                continue;
            }
        };

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "install_module",
            serde_json::json!({
                "pkg_name": metadata.name().to_owned(),
            }),
        );

        if let Some(schema_variant_id) = svs.first() {
            let variant = SchemaVariant::get_by_id_or_error(&ctx, *schema_variant_id).await?;
            let schema_id = variant.schema(&ctx).await?.id();
            let front_end_variant = variant.into_frontend_type(&ctx, schema_id).await?;
            WsEvent::module_imported(&ctx, vec![front_end_variant.clone()])
                .await?
                .publish_on_commit(&ctx)
                .await?;
            for func_id in front_end_variant.func_ids.iter() {
                let func = Func::get_by_id_or_error(&ctx, (*func_id).into()).await?;
                let front_end_func = func.into_frontend_type(&ctx).await?;
                WsEvent::func_updated(&ctx, front_end_func, None)
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;
            }
            variants.push(front_end_variant);
        } else {
            return Err(ModuleError::SchemaNotFoundFromInstall(id));
        };
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&variants)?)?)
}
