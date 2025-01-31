use std::{collections::HashMap, str::FromStr};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use postgres_types::ToSql;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    slow_rt::{self, SlowRuntimeError},
    ComponentType, DalContext, SchemaId, TransactionsError,
};
use module_index_client::{ModuleDetailsResponse, ModuleIndexClient, ModuleIndexClientError};
use si_data_pg::{PgError, PgRow};
use si_pkg::{SiPkg, SiPkgError};

pub use si_id::CachedModuleId;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CachedModuleError {
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("missing schema data for: {0}")]
    MissingSchemaData(String),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] ModuleIndexClientError),
    #[error("No module index url set on the services context")]
    ModuleIndexUrlNotSet,
    #[error("package data None")]
    NoPackageData,
    #[error("schema name None")]
    NoSchemaName,
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

pub type CachedModuleResult<T> = Result<T, CachedModuleError>;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CachedModule {
    pub id: CachedModuleId,
    pub schema_id: SchemaId,
    schema_name: Option<String>,
    pub display_name: Option<String>,
    pub category: Option<String>,
    pub link: Option<String>,
    pub color: Option<String>,
    pub description: Option<String>,
    component_type: Option<ComponentType>,
    pub latest_hash: String,
    pub created_at: DateTime<Utc>,
    package_data: Option<Vec<u8>>,
}

impl From<CachedModule> for si_frontend_types::UninstalledVariant {
    fn from(value: CachedModule) -> Self {
        Self {
            schema_id: value.schema_id,
            schema_name: value.schema_name.unwrap_or_default(),
            display_name: value.display_name,
            category: value.category,
            link: value.link,
            color: value.color,
            description: value.description,
            component_type: value.component_type.unwrap_or_default().into(),
        }
    }
}

impl TryFrom<PgRow> for CachedModule {
    type Error = CachedModuleError;

    fn try_from(row: PgRow) -> Result<Self, Self::Error> {
        let component_type_string: String = row.try_get("component_type")?;
        let component_type = Some(ComponentType::from_str(&component_type_string)?);

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
        Ok(SiPkg::load_from_bytes(package_data)?)
    }

    async fn package_data(&mut self, ctx: &DalContext) -> CachedModuleResult<&[u8]> {
        if self.package_data.is_none() {
            let query = "SELECT package_data FROM cached_modules where id = $1";
            let row = ctx.txns().await?.pg().query_one(query, &[&self.id]).await?;

            let bytes: Option<Vec<u8>> = row.try_get("package_data")?;
            self.package_data = bytes;
        }

        if self.package_data.is_none() {
            self.update_missing_pkg_data(ctx).await?;
        };

        let Some(package_data) = &self.package_data else {
            return Err(CachedModuleError::NoPackageData);
        };

        Ok(package_data.as_slice())
    }

    pub async fn schema_name(&mut self, ctx: &DalContext) -> CachedModuleResult<String> {
        if self.schema_name.is_none() {
            self.update_missing_pkg_data(ctx).await?;
        };
        let Some(schema_name) = &self.schema_name else {
            return Err(CachedModuleError::NoSchemaName);
        };
        Ok(schema_name.to_string())
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
        let services_context = ctx.services_context();
        let module_index_url = services_context
            .module_index_url()
            .ok_or(CachedModuleError::ModuleIndexUrlNotSet)?;

        let module_index_client =
            ModuleIndexClient::unauthenticated_client(module_index_url.try_into()?);

        let modules: HashMap<_, _> = module_index_client
            .list_builtins()
            .await?
            .modules
            .iter()
            .map(|builtin| (builtin.latest_hash.to_owned(), builtin.to_owned()))
            .collect();

        let hashes: Vec<_> = modules.keys().map(ToOwned::to_owned).collect();
        let uncached_hashes = CachedModule::find_missing_entries(ctx, hashes).await?;
        let mut new_modules = vec![];

        for uncached_hash in &uncached_hashes {
            let Some(module) = modules.get(uncached_hash).cloned() else {
                continue;
            };
            if let Some(new_cached_module) = Self::insert(ctx, &module).await? {
                new_modules.push(new_cached_module);
            }
        }

        if !uncached_hashes.is_empty() {
            ctx.commit_no_rebase().await?;
        }

        Ok(new_modules)
    }

    pub async fn latest_by_schema_id(
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

    pub async fn insert(
        ctx: &DalContext,
        module_details: &ModuleDetailsResponse,
    ) -> CachedModuleResult<Option<Self>> {
        let query = "
            INSERT INTO cached_modules (
                schema_id,
                latest_hash,
                created_at,
            ) VALUES (
                $1, $2, $3
            ) RETURNING
                id,
                schema_id,
                latest_hash,
                created_at,
        ";

        let Some(schema_id) = module_details.schema_id() else {
            warn!("builtin module {} has no schema id", module_details.id);
            return Ok(None);
        };
        let schema_id: SchemaId = schema_id.into();

        info!(
            "Updating sdf module cache for {} - {schema_id})",
            module_details.name
        );

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                query,
                &[
                    &schema_id,
                    &module_details.latest_hash,
                    &module_details.created_at,
                ],
            )
            .await?;

        Ok(Some(row.try_into()?))
    }

    async fn update_missing_pkg_data(&mut self, ctx: &DalContext) -> CachedModuleResult<()> {
        let services_context = ctx.services_context();
        let module_index_url = services_context
            .module_index_url()
            .ok_or(CachedModuleError::ModuleIndexUrlNotSet)?;
        let client = ModuleIndexClient::unauthenticated_client(module_index_url.try_into()?);
        let bytes = client.get_module_by_hash(&self.latest_hash).await?;

        let bytes_clone = bytes.clone();
        let pkg = slow_rt::spawn(async move { SiPkg::load_from_bytes(&bytes_clone) })?.await??;

        let pkg_schema = pkg
            .schemas()?
            .first()
            .cloned()
            .ok_or_else(|| CachedModuleError::MissingSchemaData(self.schema_id.to_string()))?;

        let pkg_variant = pkg_schema
            .variants()?
            .first()
            .cloned()
            .ok_or_else(|| CachedModuleError::MissingSchemaData(self.schema_id.to_string()))?;

        let schema_name = pkg_schema.data().map(|data| data.name()).unwrap_or("");
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
        let query = "
                 UPDATE cached_modules (
                    schema_name,
                    display_name,
                    category,
                    link,
                    color,
                    description,
                    component_type.
                    NULL::bytea AS package_data
                ) VALUES (
                    $1, $2, $3, $4
                    $5, $6, $7, $8
                ) WHERE
                    id = $9
            ";

        info!(
            "Updating sdf module cache for {:?} - {:?} ({:?})",
            self.display_name, self.schema_name, self.category
        );

        ctx.txns()
            .await?
            .pg()
            .query_one(
                query,
                &[
                    &schema_name,
                    &display_name,
                    &category,
                    &link,
                    &color,
                    &description,
                    &component_type.to_string(),
                    &bytes.as_slice(),
                    &self.id,
                ],
            )
            .await?;

        self.schema_name = Some(schema_name.to_string());
        self.display_name = display_name.map(ToString::to_string);
        self.category = Some(category.to_string());
        self.link = link;
        self.color = description.map(ToString::to_string);
        self.component_type = Some(component_type);

        Ok(())
    }
}
