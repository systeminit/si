use std::collections::{hash_map::Entry, HashMap};

use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::time::Instant;

use dal::{
    module::Module, ChangeSetId, DalContext, Schema, SchemaId, SchemaVariant, SchemaVariantId,
    WorkspacePk,
};
use module_index_client::IndexClient;
use si_frontend_types as frontend_types;

use crate::{
    server::{
        extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken},
        state::AppState,
        tracking::track,
    },
    service::ApiError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaVariantsAPIError {
    #[error("latest module not found for hash: {0}")]
    LatestModuleNotFoundForHash(String),
    #[error("too many modules found when attempting to find latest module for hash: {0}")]
    LatestModuleTooManyForHash(String),
    #[error("too many latest modules for schema: {0} (at least two hashes found: {1} and {2})")]
    LatestModuleTooManyForSchema(SchemaId, String, String),
    #[error("module error: {0}")]
    Module(#[from] dal::module::ModuleError),
    #[error("module index error: {0}")]
    ModuleIndex(#[from] module_index_client::IndexClientError),
    #[error("module index not configured")]
    ModuleIndexNotConfigured,
    #[error("module missing schema id (module id: {0}) (module hash: {1})")]
    ModuleMissingSchemaId(String, String),
    #[error("module not found for schema: {0}")]
    ModuleNotFoundForSchema(SchemaId),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl IntoResponse for SchemaVariantsAPIError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            // Return 409 when we see a conflict
            Self::Transactions(dal::TransactionsError::ConflictsOccurred(_)) => {
                StatusCode::CONFLICT
            }
            // When a graph node cannot be found for a schema variant, it is not found
            Self::SchemaVariant(dal::SchemaVariantError::NotFound(_)) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

pub fn v2_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_schema_variants))
        .route("/:schema_variant_id", get(get_variant))
}

pub async fn list_schema_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<Vec<frontend_types::SchemaVariant>>, SchemaVariantsAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let mut schema_variants = HashMap::new();

    for schema_id in Schema::list_ids(&ctx).await? {
        let default_schema_variant = SchemaVariant::get_default_for_schema(&ctx, schema_id).await?;
        if !default_schema_variant.ui_hidden() {
            schema_variants.insert(
                default_schema_variant.id,
                default_schema_variant
                    .into_frontend_type(&ctx, schema_id)
                    .await?,
            );
        }

        if let Some(unlocked) = SchemaVariant::get_unlocked_for_schema(&ctx, schema_id).await? {
            if !unlocked.ui_hidden() {
                schema_variants.insert(
                    unlocked.id,
                    unlocked.into_frontend_type(&ctx, schema_id).await?,
                );
            }
        }

        for schema_variant in SchemaVariant::list_for_schema(&ctx, schema_id).await? {
            if !SchemaVariant::list_component_ids(&ctx, schema_variant.id())
                .await?
                .is_empty()
            {
                schema_variants.insert(
                    schema_variant.id,
                    schema_variant.into_frontend_type(&ctx, schema_id).await?,
                );
            }
        }
    }

    if let Err(err) = determine_can_update_many(&ctx, &mut schema_variants, &raw_access_token).await
    {
        error!(?err, "could not perform 'determine_can_update_many'");
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_schema_variants",
        serde_json::json!({}),
    );

    Ok(Json(schema_variants.into_values().collect()))
}

pub async fn get_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
) -> Result<Json<frontend_types::SchemaVariant>, SchemaVariantsAPIError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let schema_variant = SchemaVariant::get_by_id_or_error(&ctx, schema_variant_id).await?;
    let schema_id = SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
    let mut schema_variant = schema_variant.into_frontend_type(&ctx, schema_id).await?;

    if let Err(err) = determine_can_update_one(&ctx, &mut schema_variant, &raw_access_token).await {
        error!(?err, "could not perform 'determine_can_update_one'");
    }

    // Ported from `lib/sdf-server/src/server/service/variant/get_variant.rs`, so changes may be
    // desired here...

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_variant",
        serde_json::json!({
                    "schema_name": &schema_variant.schema_name,
                    "variant_category": &schema_variant.category,
                    "variant_menu_name": schema_variant.display_name,
                    "variant_id": schema_variant.schema_variant_id,
                    "schema_id": schema_variant.schema_id,
                    "variant_component_type": schema_variant.component_type,
        }),
    );

    Ok(Json(schema_variant))
}

#[instrument(
    name = "sdf.v2.variant.determine_can_update_many",
    level = "debug",
    skip_all
)]
async fn determine_can_update_many(
    ctx: &DalContext,
    schema_variants: &mut HashMap<SchemaVariantId, frontend_types::SchemaVariant>,
    raw_access_token: &str,
) -> Result<(), SchemaVariantsAPIError> {
    let start = Instant::now();

    // Find all local hashes.
    let mut local_hashes = HashMap::new();
    for schema_variant in schema_variants.values() {
        let schema_id: SchemaId = schema_variant.schema_id.into();
        if let Entry::Vacant(entry) = local_hashes.entry(schema_id) {
            let local_module = Module::find_for_module_schema_id(ctx, schema_id.into())
                .await?
                .ok_or(SchemaVariantsAPIError::ModuleNotFoundForSchema(schema_id))?;
            entry.insert(local_module.root_hash().to_owned());
        }
    }

    // Collect the latest modules.
    let latest_modules = {
        let module_index_url = ctx
            .module_index_url()
            .ok_or(SchemaVariantsAPIError::ModuleIndexNotConfigured)?;
        let module_index_client = IndexClient::new(module_index_url.try_into()?, raw_access_token);
        module_index_client
            .list_latest_modules(local_hashes.values().map(|h| h.to_owned()).collect())
            .await?
    };

    // Find the latest hashes from the latest modules.
    let mut latest_hashes: HashMap<SchemaId, String> = HashMap::new();
    for latest_module in latest_modules.modules {
        let schema_id: SchemaId = latest_module
            .schema_id()
            .ok_or(SchemaVariantsAPIError::ModuleMissingSchemaId(
                latest_module.id.to_owned(),
                latest_module.latest_hash.to_owned(),
            ))?
            .into();
        match latest_hashes.entry(schema_id) {
            Entry::Occupied(entry) => {
                let existing_hash: String = entry.get().to_owned();
                return Err(SchemaVariantsAPIError::LatestModuleTooManyForSchema(
                    schema_id,
                    existing_hash,
                    latest_module.latest_hash,
                ));
            }
            Entry::Vacant(entry) => {
                entry.insert(latest_module.latest_hash);
            }
        }
    }

    // Determine if we can update based on the latest modules.
    for schema_variant in schema_variants.values_mut() {
        let schema_id: SchemaId = schema_variant.schema_id.into();
        match (latest_hashes.get(&schema_id), local_hashes.get(&schema_id)) {
            (Some(latest_hash), Some(local_hash)) => {
                if latest_hash != local_hash {
                    schema_variant.can_update = true;
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

    debug!(
        "function 'determine_can_update_many' took: {:?}",
        start.elapsed()
    );

    Ok(())
}

#[instrument(
    name = "sdf.v2.variant.determine_can_update_one",
    level = "debug",
    skip_all
)]
async fn determine_can_update_one(
    ctx: &DalContext,
    schema_variant: &mut frontend_types::SchemaVariant,
    raw_access_token: &str,
) -> Result<(), SchemaVariantsAPIError> {
    let start = Instant::now();

    // Find the local hash.
    let schema_id: SchemaId = schema_variant.schema_id.into();
    let local_module = Module::find_for_module_schema_id(ctx, schema_id.into())
        .await?
        .ok_or(SchemaVariantsAPIError::ModuleNotFoundForSchema(schema_id))?;
    let local_hash = local_module.root_hash().to_owned();

    // Collect the latest module.
    let latest_modules = {
        let module_index_url = ctx
            .module_index_url()
            .ok_or(SchemaVariantsAPIError::ModuleIndexNotConfigured)?;
        let module_index_client = IndexClient::new(module_index_url.try_into()?, raw_access_token);
        module_index_client
            .list_latest_modules(vec![local_hash.to_owned()])
            .await?
    };
    if latest_modules.modules.len() > 1 {
        return Err(SchemaVariantsAPIError::LatestModuleTooManyForHash(
            local_hash.to_owned(),
        ));
    }
    let latest_module = latest_modules
        .modules
        .first()
        .ok_or(SchemaVariantsAPIError::LatestModuleNotFoundForHash(
            local_hash.to_owned(),
        ))?
        .to_owned();

    // Determine if we can update based on the latest module.
    if latest_module.latest_hash != local_hash {
        schema_variant.can_update = true;
    }

    debug!(
        "function 'determine_can_update_one' took: {:?}",
        start.elapsed()
    );

    Ok(())
}
