use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinSet;
use ulid::Ulid;

use crate::{
    slow_rt::{self, SlowRuntimeError},
    ComponentType, DalContext, SchemaId, TransactionsError,
};
use module_index_client::{ModuleDetailsResponse, ModuleIndexClient, ModuleIndexClientError};
use si_data_pg::{PgError, PgRow};
use si_pkg::{SiPkg, SiPkgError};

pub use si_id::CachedModuleId;

const PLACEHOLDER_OWNER_USER_ID: &str = "-";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CachedModuleError {
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] ModuleIndexClientError),
    #[error("No module index url set on the services context")]
    ModuleIndexUrlNotSet,
    #[error("package data None")]
    NoPackageData,
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("si-pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("strum parse error: {0}")]
    StrumParse(#[from] strum::ParseError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type CachedModuleResult<T> = Result<T>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CachedModule {
    pub id: CachedModuleId,
    pub schema_id: SchemaId,
    pub schema_name: String,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub link: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    pub component_type: ComponentType,
    pub latest_hash: String,
    pub created_at: DateTime<Utc>,
    pub package_data: Option<Vec<u8>>,
}

impl From<CachedModule> for si_frontend_types::UninstalledVariant {
    fn from(value: CachedModule) -> Self {
        Self {
            schema_id: value.schema_id,
            schema_name: value.schema_name,
            display_name: value.display_name,
            category: value.category,
            link: value.link,
            color: value.color,
            description: value.description,
            component_type: value.component_type.into(),
        }
    }
}

// NOTE(nick): the frontend type's shape might be able to be refactored now that syncing only
// relies on the cache.
impl From<CachedModule> for si_frontend_types::LatestModule {
    fn from(value: CachedModule) -> Self {
        Self {
            id: value.id.to_string(),
            name: value.schema_name,
            description: value.description,
            owner_user_id: PLACEHOLDER_OWNER_USER_ID.to_string(),
            owner_display_name: None,
            metadata: serde_json::Value::Null,
            latest_hash: value.latest_hash,
            latest_hash_created_at: value.created_at,
            created_at: value.created_at,
            schema_id: Some(value.schema_id.to_string()),
        }
    }
}

impl TryFrom<PgRow> for CachedModule {
    type Error = CachedModuleError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        let component_type_string: String = row.try_get("component_type")?;
        let component_type = ComponentType::from_str(&component_type_string)?;

        Ok(Self {
            id: row.try_get("id")?,
            schema_id: row.try_get("schema_id")?,
            schema_name: row.try_get("schema_name")?,
            display_name: row.try_get("display_name")?,
            category: row.try_get("category")?,
            link: row.try_get("link")?,
            color: row.try_get("color")?,
            description: row.try_get("description")?,
            component_type,
            latest_hash: row.try_get("latest_hash")?,
            created_at: row.try_get("created_at")?,
            package_data: row.try_get("package_data")?,
        })
    }
}

impl CachedModule {
    pub async fn si_pkg(&mut self, ctx: &DalContext) -> CachedModuleResult<SiPkg> {
        let package_data = self.package_data(ctx).await?;
        // slow_rt, and cache this
        SiPkg::load_from_bytes(package_data)
    }

    async fn package_data(&mut self, ctx: &DalContext) -> CachedModuleResult<&[u8]> {
        if self.package_data.is_none() {
            let query = "SELECT package_data FROM cached_modules where id = $1";
            let row = ctx.txns().await?.pg().query_one(query, &[&self.id]).await?;

            let bytes: Option<Vec<u8>> = row.try_get("package_data")?;
            self.package_data = bytes;
        }

        let Some(package_data) = &self.package_data else {
            return Err(CachedModuleError::NoPackageData.into());
        };

        Ok(package_data.as_slice())
    }

    pub async fn find_missing_entries(
        ctx: &DalContext,
        hashes: Vec<String>,
    ) -> CachedModuleResult<Vec<String>> {
        // Constructs a list of parameters like '($1), ($2), ($3), ($4)' for
        // each input value so they can be used as a table expression in the
        // query, for the left join
        let values_expr = hashes
            .iter()
            .enumerate()
            .map(|(idx, _)| format!("(${})", idx + 1))
            .join(",");

        let params: Vec<_> = hashes
            .iter()
            .map(|hash| hash as &(dyn ToSql + Sync))
            .collect();

        let query = format!(
            "
            SELECT hashes.hash
                FROM (VALUES {values_expr}) AS hashes(hash)
            LEFT JOIN cached_modules on cached_modules.latest_hash = hashes.hash
            WHERE cached_modules.latest_hash IS NULL
            "
        );

        let rows = ctx.txns().await?.pg().query(&query, &params).await?;

        let mut result = vec![];

        for row in rows {
            result.push(row.try_get("hash")?);
        }

        Ok(result)
    }

    /// Calls out to the module index server to fetch the latest module set, and
    /// updates the cache for any new builtin modules
    pub async fn update_cached_modules(ctx: &DalContext) -> CachedModuleResult<Vec<CachedModule>> {
        let module_index_client = {
            let services_context = ctx.services_context();
            let module_index_url = services_context
                .module_index_url()
                .ok_or(CachedModuleError::ModuleIndexUrlNotSet)?;

            ModuleIndexClient::unauthenticated_client(module_index_url.try_into()?)
        };

        let modules: HashMap<_, _> = module_index_client
            .list_builtins()
            .await?
            .modules
            .iter()
            .map(|builtin| (builtin.latest_hash.to_owned(), builtin.to_owned()))
            .collect();

        // We need to remove any schemas that are in the cache but no longer in the builtin list
        Self::remove_unused(ctx, &modules).await?;

        let ctx_clone = ctx.clone();
        ctx_clone.commit_no_rebase().await?;

        let hashes: Vec<_> = modules.keys().map(ToOwned::to_owned).collect();
        let uncached_hashes = CachedModule::find_missing_entries(ctx, hashes).await?;

        let batch_size = 10;
        let mut new_modules = vec![];
        for hash_chunk in uncached_hashes.chunks(batch_size) {
            let ctx = ctx.clone();
            let mut join_set = JoinSet::new();

            for uncached_hash in hash_chunk {
                let Some(module) = modules.get(uncached_hash).cloned() else {
                    continue;
                };

                let client = module_index_client.clone();
                join_set.spawn(async move {
                    let module_id = module.id.to_owned();
                    Ok::<(ModuleDetailsResponse, Arc<Vec<u8>>), CachedModuleError>((
                        module,
                        Arc::new(
                            client
                                .get_builtin(Ulid::from_string(&module_id).unwrap_or_default())
                                .await?,
                        ),
                    ))
                });
            }

            while let Some(res) = join_set.join_next().await {
                let (module, module_bytes) = res??;
                if let Some(new_cached_module) = Self::insert(&ctx, &module, module_bytes).await? {
                    new_modules.push(new_cached_module);
                }
            }

            if !uncached_hashes.is_empty() {
                ctx.commit_no_rebase().await?;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(new_modules)
    }

    async fn remove_unused(
        ctx: &DalContext,
        module_details_by_hash: &HashMap<String, ModuleDetailsResponse>,
    ) -> CachedModuleResult<()> {
        let builtin_schema_ids: HashSet<SchemaId> = module_details_by_hash
            .values()
            .filter_map(|module| {
                module
                    .schema_id
                    .as_ref()
                    .and_then(|id_string| Ulid::from_string(id_string.as_str()).ok())
            })
            .map(Into::into)
            .collect();
        let cached_schema_ids: Vec<SchemaId> = CachedModule::latest_modules(ctx)
            .await?
            .iter()
            .map(|lm| lm.schema_id)
            .collect();

        // Look at all schema IDs in the cache and determine if any of them are no longer builtins.
        // If they aren't, ALL modules corresponding to them get remove.
        for schema_id in cached_schema_ids {
            if !builtin_schema_ids.contains(&schema_id) {
                CachedModule::remove_all_for_schema_id(ctx, schema_id).await?;
            }
        }
        Ok(())
    }

    pub async fn find_latest_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> CachedModuleResult<Option<CachedModule>> {
        let query = "
            SELECT DISTINCT ON (schema_id)
                id,
                schema_id,
                schema_name,
                display_name,
                category,
                link,
                color,
                description,
                component_type,
                latest_hash,
                created_at,
                package_data
            FROM cached_modules
            WHERE schema_id = $1
            ORDER BY schema_id, created_at DESC
        ";

        let maybe_row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(query, &[&schema_id])
            .await?;

        Ok(match maybe_row {
            Some(row) => Some(row.try_into()?),
            None => None,
        })
    }

    pub async fn list_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> CachedModuleResult<Vec<CachedModule>> {
        let query = "
            SELECT
                id,
                schema_id,
                schema_name,
                display_name,
                category,
                link,
                color,
                description,
                component_type,
                latest_hash,
                created_at,
                NULL::bytea AS package_data
            FROM cached_modules
            WHERE schema_id = $1
            ORDER BY schema_id, created_at DESC
        ";

        let rows = ctx.txns().await?.pg().query(query, &[&schema_id]).await?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(row.try_into()?);
        }
        Ok(result)
    }

    pub async fn latest_modules(ctx: &DalContext) -> CachedModuleResult<Vec<CachedModule>> {
        let query = "
            SELECT DISTINCT ON (schema_id)
                id,
                schema_id,
                schema_name,
                display_name,
                category,
                link,
                color,
                description,
                component_type,
                latest_hash,
                created_at,
                NULL::bytea AS package_data
            FROM cached_modules
            ORDER BY schema_id, created_at DESC
        ";

        let rows = ctx.txns().await?.pg().query(query, &[]).await?;

        let mut result = vec![];

        for row in rows {
            result.push(row.try_into()?);
        }

        Ok(result)
    }

    async fn insert(
        ctx: &DalContext,
        module_details: &ModuleDetailsResponse,
        pkg_bytes: Arc<Vec<u8>>,
    ) -> CachedModuleResult<Option<Self>> {
        let bytes_clone = pkg_bytes.clone();
        let pkg = slow_rt::spawn(async move { SiPkg::load_from_bytes(&bytes_clone) })?.await??;

        let query = "
            INSERT INTO cached_modules (
                schema_id,
                schema_name,
                display_name,
                category,
                link,
                color,
                description,
                component_type,
                latest_hash,
                created_at,
                package_data
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10, $11
            ) RETURNING
                id,
                schema_id,
                schema_name,
                display_name,
                category,
                link,
                color,
                description,
                component_type,
                latest_hash,
                created_at,
                NULL::bytea AS package_data
        ";

        let Some(schema_id) = module_details.schema_id() else {
            warn!("builtin module {} has no schema id", module_details.id);
            return Ok(None);
        };
        let schema_id: SchemaId = schema_id.into();

        let Some(pkg_schema) = pkg.schemas()?.first().cloned() else {
            warn!("builtin module {} has no schema", module_details.id);
            return Ok(None);
        };

        let Some(pkg_variant) = pkg_schema.variants()?.first().cloned() else {
            warn!(
                "builtin module {} has a schema with no variant",
                module_details.id
            );
            return Ok(None);
        };

        let schema_name = pkg_schema
            .data()
            .map(|data| data.name())
            .unwrap_or(module_details.name.as_str());
        let display_name = pkg_schema.data().and_then(|data| data.category_name());
        let category = pkg_schema.data().map(|data| data.category()).unwrap_or("");
        let link = pkg_variant
            .data()
            .and_then(|data| data.link().map(ToString::to_string));
        let color = pkg_variant.data().and_then(|data| data.color());
        let description = pkg_variant.data().and_then(|data| data.description());
        let component_type: ComponentType = pkg_variant
            .data()
            .map(|data| data.component_type().into())
            .unwrap_or_default();

        info!(
            "Updating sdf module cache for {} - {schema_name} ({category:?})",
            module_details.name
        );

        let bytes_ref = pkg_bytes.as_slice();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                query,
                &[
                    &schema_id,
                    &schema_name,
                    &display_name,
                    &category,
                    &link,
                    &color,
                    &description,
                    &component_type.to_string(),
                    &module_details.latest_hash,
                    &module_details.created_at,
                    &bytes_ref,
                ],
            )
            .await?;

        Ok(Some(row.try_into()?))
    }

    pub async fn remove_all_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> CachedModuleResult<()> {
        let query = r#"
            DELETE FROM cached_modules WHERE schema_id = $1
            "#;

        ctx.txns().await?.pg().query(query, &[&schema_id]).await?;

        Ok(())
    }
}
