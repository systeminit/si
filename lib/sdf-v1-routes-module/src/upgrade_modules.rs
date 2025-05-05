use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
    http::Uri,
};
use dal::{
    ChangeSet,
    DalContext,
    Func,
    Schema,
    SchemaId,
    SchemaVariant,
    WsEvent,
    cached_module::CachedModule,
    pkg::{
        ImportOptions,
        import_pkg_from_pkg,
    },
};
use sdf_core::{
    async_route::handle_error,
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use telemetry::prelude::*;
use ulid::Ulid;

use crate::{
    ModuleError,
    ModuleResult,
};

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
) -> ModuleResult<ForceChangeSetResponse<Ulid>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let task_id = Ulid::new();

    tokio::task::spawn(async move {
        if let Err(err) = upgrade_modules_inner(
            &ctx,
            &original_uri,
            &host_name,
            PosthogClient(posthog_client),
            request.schema_ids,
        )
        .await
        {
            return handle_error(&ctx, original_uri, task_id, err).await;
        };

        let event = match WsEvent::async_finish_workspace(&ctx, task_id).await {
            Ok(event) => event,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        if let Err(err) = event.publish_immediately(&ctx).await {
            handle_error(&ctx, original_uri, task_id, err).await;
        };
    });

    Ok(ForceChangeSetResponse::new(force_change_set_id, task_id))
}

pub async fn upgrade_modules_inner(
    ctx: &DalContext,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): PosthogClient,
    schema_ids: Vec<SchemaId>,
) -> ModuleResult<()> {
    for schema_id in schema_ids {
        let schema_exists_locally = Schema::exists_locally(ctx, schema_id).await?;

        let at_least_one_unlocked_variant = SchemaVariant::get_unlocked_for_schema(ctx, schema_id)
            .await?
            .is_some();
        if schema_exists_locally && at_least_one_unlocked_variant {
            warn!(%schema_id, %schema_exists_locally, %at_least_one_unlocked_variant, "cannot upgrade module for schema since it exists locally and has at least one unlocked variant");
            continue;
        }

        let Some(mut cached_module) =
            CachedModule::find_latest_for_schema_id(ctx, schema_id).await?
        else {
            warn!(%schema_id, "no cached module found for schema");
            continue;
        };

        let si_pkg = cached_module.si_pkg(ctx).await?;

        let metadata = si_pkg.metadata()?;
        let (_, schema_variant_ids, _) = match import_pkg_from_pkg(
            ctx,
            &si_pkg,
            Some(ImportOptions {
                schema_id: Some(schema_id.into()),
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
            ctx,
            original_uri,
            host_name,
            "upgrade_modules",
            serde_json::json!({
                "pkg_name": metadata.name().to_owned(),
            }),
        );

        if let Some(schema_variant_id) = schema_variant_ids.first() {
            let variant = SchemaVariant::get_by_id(ctx, *schema_variant_id).await?;
            let schema_id = variant.clone().schema(ctx).await?.id();
            let front_end_variant = variant.clone().into_frontend_type(ctx, schema_id).await?;
            WsEvent::module_imported(ctx, vec![front_end_variant.clone()])
                .await?
                .publish_on_commit(ctx)
                .await?;
            WsEvent::schema_variant_updated(ctx, schema_id, variant)
                .await?
                .publish_on_commit(ctx)
                .await?;
            for func_id in front_end_variant.func_ids.iter() {
                let func = Func::get_by_id(ctx, *func_id).await?;
                let front_end_func = func.into_frontend_type(ctx).await?;
                WsEvent::func_updated(ctx, front_end_func, None)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            }
        } else {
            return Err(ModuleError::SchemaNotFoundFromInstall(schema_id.into()));
        };
    }

    WsEvent::modules_updated(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit().await?;
    Ok(())
}
