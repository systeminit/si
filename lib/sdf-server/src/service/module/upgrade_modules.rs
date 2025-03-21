use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    cached_module::CachedModule,
    pkg::{import_pkg_from_pkg, ImportOptions},
    ChangeSet, Func, Schema, SchemaId, SchemaVariant, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::SchemaVariant as FrontendVariant;

use crate::{
    extract::{v1::AccessBuilder, HandlerContext, PosthogClient},
    service::{force_change_set_response::ForceChangeSetResponse, module::ModuleError},
    track,
};

use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeModulesRequest {
    pub schema_ids: Vec<SchemaId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn upgrade_modules(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<UpgradeModulesRequest>,
) -> Result<ForceChangeSetResponse<Vec<FrontendVariant>>, ModuleError> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut variants = Vec::new();

    for schema_id in request.schema_ids {
        let schema_exists_locally = Schema::exists_locally(&ctx, schema_id).await?;
        let at_least_one_unlocked_variant = SchemaVariant::get_unlocked_for_schema(&ctx, schema_id)
            .await?
            .is_some();
        if schema_exists_locally && at_least_one_unlocked_variant {
            warn!(%schema_id, %schema_exists_locally, %at_least_one_unlocked_variant, "cannot upgrade module for schema since it exists locally and has at least one unlocked variant");
            continue;
        }

        let Some(mut cached_module) =
            CachedModule::find_latest_for_schema_id(&ctx, schema_id).await?
        else {
            warn!(%schema_id, "no cached module found for schema");
            continue;
        };

        let si_pkg = cached_module.si_pkg(&ctx).await?;

        let metadata = si_pkg.metadata()?;
        let (_, schema_variant_ids, _) = match import_pkg_from_pkg(
            &ctx,
            &si_pkg,
            Some(ImportOptions {
                schema_id: Some(schema_id),
                ..Default::default()
            }),
        )
        .await
        {
            Ok(details) => details,
            Err(err) => {
                error!(si.error.message = ?err, %schema_id, cached_module_id = %cached_module.id, "cannot install pkg");
                continue;
            }
        };

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "upgrade_modules",
            serde_json::json!({
                "pkg_name": metadata.name().to_owned(),
            }),
        );

        if let Some(schema_variant_id) = schema_variant_ids.first() {
            let variant = SchemaVariant::get_by_id(&ctx, *schema_variant_id).await?;
            let schema_id = variant.schema(&ctx).await?.id();
            let front_end_variant = variant.into_frontend_type(&ctx, schema_id).await?;
            WsEvent::module_imported(&ctx, vec![front_end_variant.clone()])
                .await?
                .publish_on_commit(&ctx)
                .await?;
            for func_id in front_end_variant.func_ids.iter() {
                let func = Func::get_by_id(&ctx, *func_id).await?;
                let front_end_func = func.into_frontend_type(&ctx).await?;
                WsEvent::func_updated(&ctx, front_end_func, None)
                    .await?
                    .publish_on_commit(&ctx)
                    .await?;
            }
            variants.push(front_end_variant);
        } else {
            return Err(ModuleError::SchemaNotFoundFromInstall(schema_id));
        };
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, variants))
}
