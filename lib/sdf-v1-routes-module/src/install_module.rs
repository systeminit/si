use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Func,
    Schema,
    SchemaVariant,
    WsEvent,
    pkg::{
        ImportOptions,
        import_pkg_from_pkg,
    },
};
use module_index_client::ModuleIndexClient;
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    request::RawAccessToken,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;
use si_frontend_types::SchemaVariant as FrontendVariant;
use si_pkg::SiPkg;
use telemetry::prelude::*;
use ulid::Ulid;

use crate::ModuleError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallModuleRequest {
    pub ids: Vec<Ulid>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn install_module(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<InstallModuleRequest>,
) -> Result<ForceChangeSetResponse<Vec<FrontendVariant>>, ModuleError> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured),
    };

    let mut variants = Vec::new();

    let module_index_client =
        ModuleIndexClient::new(module_index_url.try_into()?, &raw_access_token)?;

    // Before we install the module(s), ensure that there are no unlocked variants.
    let mut ids_with_details = Vec::with_capacity(request.ids.len());
    for id in request.ids {
        let module_details = module_index_client.module_details(id).await?;

        if let Some(schema_id) = module_details.schema_id() {
            if Schema::exists_locally(&ctx, schema_id.into()).await?
                && SchemaVariant::get_unlocked_for_schema(&ctx, schema_id.into())
                    .await?
                    .is_some()
            {
                // TODO(nick): do not use the 500 toast for this.
                return Err(ModuleError::UnlockedSchemaVariantForModuleToInstall(
                    module_details.id,
                    schema_id.into(),
                ));
            }
        } else {
            // NOTE(nick): I don't love this. Basically, if you install an old module, it can
            // clobber your editing asset. On the other hand, I think erroring here is a bad idea
            // because we should still allow you to install it. Ideally, we'd give the user the
            // option to choose what to do when they want to install module that doesn't know its
            // schema id. Fortunately, I think these modules should not exist anymore at the time
            // of writing, so this is most likely just a typing issue in Rust rather than a real
            // world problem.
            warn!(
                module_id = %module_details.id,
                "found older module where its schema id is empty, so we will still install it, but we cannot triviallly determine if there are unlocked variants that may be clobbered"
            );
        }

        ids_with_details.push((id, module_details));
    }

    // After validating that we can install the modules, get on with it.
    for (id, module_details) in ids_with_details {
        let pkg_data = module_index_client.download_module(id).await?;

        let pkg = SiPkg::load_from_bytes(&pkg_data)?;

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
            return Err(ModuleError::SchemaNotFoundFromInstall(id));
        };
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, variants))
}
