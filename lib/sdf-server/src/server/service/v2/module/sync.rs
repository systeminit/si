use std::collections::{hash_map::Entry, HashMap, HashSet};

use axum::{
    extract::{OriginalUri, Path},
    Json,
};
use telemetry::prelude::*;
use tokio::time::Instant;

use dal::{module::Module, ChangeSetId, DalContext, SchemaId, SchemaVariant, WorkspacePk};
use module_index_client::ModuleIndexClient;
use si_frontend_types as frontend_types;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken},
    tracking::track,
};

use super::ModulesAPIError;

pub async fn sync(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<frontend_types::SyncedModules>, ModulesAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let synced_modules = sync_inner(&ctx, &raw_access_token).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "sync",
        serde_json::json!({}),
    );

    Ok(Json(synced_modules))
}

#[instrument(
    name = "sdf.v2.module.sync.sync_inner"
    skip_all,
    level = "debug",
)]
async fn sync_inner(
    ctx: &DalContext,
    raw_access_token: &str,
) -> Result<frontend_types::SyncedModules, ModulesAPIError> {
    let start = Instant::now();

    // Collect all user facing schema variants. We need to see what can be upgraded.
    let schema_variants = SchemaVariant::list_user_facing(ctx).await?;

    // Find all local hashes and mark all seen schemas.
    let mut local_hashes = HashMap::new();
    let mut seen_schema_ids = HashSet::new();
    for schema_variant in &schema_variants {
        let schema_id: SchemaId = schema_variant.schema_id.into();
        seen_schema_ids.insert(schema_id);

        if let Entry::Vacant(entry) = local_hashes.entry(schema_id) {
            let local_module = Module::find_for_member_id(ctx, schema_id)
                .await?
                .ok_or(ModulesAPIError::ModuleNotFoundForSchema(schema_id))?;
            entry.insert(local_module.root_hash().to_owned());
        }
    }

    // Collect the latest modules.
    let latest_modules = {
        let module_index_url = ctx
            .module_index_url()
            .ok_or(ModulesAPIError::ModuleIndexNotConfigured)?;
        let module_index_client =
            ModuleIndexClient::new(module_index_url.try_into()?, raw_access_token);
        module_index_client.list_latest_modules().await?
    };

    // Begin populating synced modules.
    let mut synced_modules = frontend_types::SyncedModules::new();

    // Group the latest hashes by schema. Populate installable modules along the way.
    let mut latest_modules_by_schema: HashMap<SchemaId, frontend_types::LatestModule> =
        HashMap::new();
    for latest_module in latest_modules.modules {
        let schema_id: SchemaId = latest_module
            .schema_id()
            .ok_or(ModulesAPIError::ModuleMissingSchemaId(
                latest_module.id.to_owned(),
                latest_module.latest_hash.to_owned(),
            ))?
            .into();
        match latest_modules_by_schema.entry(schema_id) {
            Entry::Occupied(entry) => {
                let existing: frontend_types::LatestModule = entry.get().to_owned();
                return Err(ModulesAPIError::LatestModuleTooManyForSchema(
                    schema_id,
                    existing.latest_hash,
                    latest_module.latest_hash,
                ));
            }
            Entry::Vacant(entry) => {
                entry.insert(latest_module.to_owned());
            }
        }

        if !seen_schema_ids.contains(&schema_id) {
            synced_modules.installable.push(latest_module.to_owned());
        }
    }

    // Populate upgradeable modules.
    for schema_variant in schema_variants {
        let schema_id: SchemaId = schema_variant.schema_id.into();
        match (
            latest_modules_by_schema.get(&schema_id),
            local_hashes.get(&schema_id),
        ) {
            (Some(latest_module), Some(local_hash)) => {
                if &latest_module.latest_hash != local_hash {
                    synced_modules
                        .upgradeable
                        .insert(schema_variant.schema_variant_id, latest_module.to_owned());
                }
            }
            (maybe_latest, maybe_local) => {
                trace!(
                    %schema_id,
                    %schema_variant.schema_variant_id,
                    %schema_variant.schema_name,
                    ?maybe_latest,
                    ?maybe_local,
                    "skipping since there's incomplete data for determining if schema variant can be updated (perhaps module was not prompted to builtin?)"
                );
            }
        }
    }

    debug!("syncing modules took: {:?}", start.elapsed());

    Ok(synced_modules)
}
