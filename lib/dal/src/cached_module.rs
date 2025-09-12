use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
    time::Duration,
};

use chrono::{
    DateTime,
    Utc,
};
use edda_client::EddaClient;
use itertools::Itertools;
use module_index_client::{
    ModuleDetailsResponse,
    ModuleIndexClient,
    ModuleIndexClientError,
};
use postgres_types::ToSql;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::{
    PgError,
    PgRow,
};
use si_db::HistoryActor;
pub use si_id::CachedModuleId;
use si_id::UserPk;
use si_pkg::{
    SiPkg,
    SiPkgError,
    SiPkgSchemaData,
    SiPkgSchemaVariantData,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::task::JoinSet;
use ulid::Ulid;

use crate::{
    ComponentType,
    DalContext,
    SchemaId,
    TransactionsError,
    slow_rt::{
        self,
        SlowRuntimeError,
    },
};

const PLACEHOLDER_OWNER_USER_ID: &str = "-";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CachedModuleError {
    #[error("edda client error: {0}")]
    EddaClient(#[from] edda_client::ClientError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
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
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

pub type CachedModuleResult<T> = Result<T, CachedModuleError>;

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
    pub package_summary: Option<PackageSummary>,
    pub latest_hash: String,
    pub created_at: DateTime<Utc>,
    pub package_data: Option<Vec<u8>>,
    pub scoped_to_user_pk: Option<UserPk>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackageSummary {
    pub socket_count: u32,
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
        let component_type = component_type_string.parse()?;
        let package_summary: Option<serde_json::Value> = row.try_get("package_summary")?;
        let package_summary = package_summary.map(serde_json::from_value).transpose()?;

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
            package_summary,
            latest_hash: row.try_get("latest_hash")?,
            created_at: row.try_get("created_at")?,
            package_data: row.try_get("package_data")?,
            scoped_to_user_pk: row.try_get("scoped_to_user_pk")?,
        })
    }
}

const CACHED_MODULE_GET_FIELDS: &str = "
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
    package_data,
    scoped_to_user_pk,
    package_summary
";

const CACHED_MODULE_LIST_FIELDS: &str = "
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
    NULL::bytea AS package_data,
    scoped_to_user_pk,
    package_summary
";

const BATCH_SIZE: usize = 10;
const WAIT_BETWEEN_BATCHES: Duration = Duration::from_millis(100);

impl CachedModule {
    pub async fn si_pkg(&mut self, ctx: &DalContext) -> CachedModuleResult<SiPkg> {
        self.package_data(ctx).await?;
        if let Some(package_data) = self.package_data.take() {
            Ok(slow_rt::spawn(async move { SiPkg::load_from_bytes(&package_data) })?.await??)
        } else {
            Err(CachedModuleError::NoPackageData)
        }
    }

    async fn package_data(&mut self, ctx: &DalContext) -> CachedModuleResult<&[u8]> {
        if self.package_data.is_none() {
            let query = "SELECT package_data FROM cached_modules where id = $1";
            let row = ctx.txns().await?.pg().query_one(query, &[&self.id]).await?;

            let bytes: Option<Vec<u8>> = row.try_get("package_data")?;
            self.package_data = bytes;
        }

        let Some(package_data) = &self.package_data else {
            return Err(CachedModuleError::NoPackageData);
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
        Ok(rows
            .into_iter()
            .map(|row| row.try_get("hash"))
            .try_collect()?)
    }

    pub async fn create_private_module(
        ctx: &DalContext,
        module_details: ModuleDetailsResponse,
        payload: Vec<u8>,
    ) -> CachedModuleResult<Option<Self>> {
        let user_pk: UserPk = module_details.owner_user_id.parse()?;
        let maybe_module =
            Self::insert(ctx, &module_details, Arc::new(payload), Some(user_pk)).await?;

        Ok(maybe_module)
    }

    /// Calls out to the module index server to fetch the latest module set, and
    /// updates the cache for any new builtin modules
    pub async fn update_cached_modules(
        ctx: &DalContext,
    ) -> CachedModuleResult<(Vec<CachedModuleId>, Vec<CachedModuleId>)> {
        let module_index_client = {
            let services_context = ctx.services_context();
            let module_index_url = services_context
                .module_index_url()
                .ok_or(CachedModuleError::ModuleIndexUrlNotSet)?;

            ModuleIndexClient::unauthenticated_client(module_index_url.try_into()?)?
        };

        let modules: HashMap<_, _> = module_index_client
            .list_builtins()
            .await?
            .modules
            .iter()
            .map(|builtin| (builtin.latest_hash.to_owned(), builtin.to_owned()))
            .collect();

        // We need to remove any schemas that are in the cache but no longer in the builtin list
        let removed_module_ids = Self::remove_unused(ctx, &modules).await?;

        let ctx_clone = ctx.clone();
        ctx_clone.commit_no_rebase().await?;

        let new_module_ids =
            Self::cache_modules(ctx, &modules, module_index_client).await?.iter().map(|module| module.id).collect();

        // Now check and fix up any missing package summaries
        Self::update_missing_package_summaries(ctx).await?;

        Ok((new_module_ids, removed_module_ids))
    }

    async fn cache_modules(
        ctx: &DalContext,
        modules: &HashMap<String, ModuleDetailsResponse>,
        module_index_client: ModuleIndexClient,
    ) -> CachedModuleResult<Vec<CachedModule>> {
        let hashes = modules.keys().map(ToOwned::to_owned).collect_vec();
        let uncached_hashes = CachedModule::find_missing_entries(ctx, hashes).await?;

        let mut new_modules = vec![];
        for hash_chunk in uncached_hashes.chunks(BATCH_SIZE) {
            let ctx = ctx.clone();
            let mut join_set = JoinSet::new();

            for uncached_hash in hash_chunk {
                let Some(module) = modules.get(uncached_hash).cloned() else {
                    continue;
                };

                let module_index = module_index_client.clone();
                join_set.spawn(async move {
                    let module_id = module.id.to_owned();
                    Ok::<(ModuleDetailsResponse, Arc<Vec<u8>>), CachedModuleError>((
                        module,
                        Arc::new(
                            module_index
                                .get_builtin(Ulid::from_string(&module_id).unwrap_or_default())
                                .await?,
                        ),
                    ))
                });
            }

            while let Some(res) = join_set.join_next().await {
                let (module, module_bytes) = res??;
                if let Some(new_cached_module) =
                    Self::insert(&ctx, &module, module_bytes, None).await?
                {
                    new_modules.push(new_cached_module);
                }
            }

            if !uncached_hashes.is_empty() {
                ctx.commit_no_rebase().await?;
            }
            tokio::time::sleep(WAIT_BETWEEN_BATCHES).await;
        }

        Ok(new_modules)
    }

    async fn remove_unused(
        ctx: &DalContext,
        module_details_by_hash: &HashMap<String, ModuleDetailsResponse>,
    ) -> CachedModuleResult<Vec<CachedModuleId>> {
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

        // Look at all schema IDs in the cache and determine if any of them are no longer builtins.
        // If they aren't, ALL modules corresponding to them get remove.
        let mut removed_module_ids = Vec::new();
        for lm in CachedModule::latest_user_independent_modules(ctx).await? {
            if !builtin_schema_ids.contains(&lm.schema_id) {
                removed_module_ids.extend(CachedModule::remove_all_for_schema_id(ctx, lm.schema_id).await?);
            }
        }

        Ok(removed_module_ids)
    }

    #[instrument(name = "cached_module.install_from_module", level = "info", skip_all)]
    pub async fn find_latest_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> CachedModuleResult<Option<CachedModule>> {
        let query = format!(
            "
                SELECT DISTINCT ON (schema_id)
                    {CACHED_MODULE_GET_FIELDS}
                FROM cached_modules
                WHERE schema_id = $1
                ORDER BY schema_id, created_at DESC
            "
        );

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(&query, &[&schema_id])
            .await?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn find_latest_for_schema_name(
        ctx: &DalContext,
        schema_name: &str,
    ) -> CachedModuleResult<Option<CachedModule>> {
        let query = format!(
            "
                SELECT DISTINCT ON (schema_id)
                    {CACHED_MODULE_GET_FIELDS}
                FROM cached_modules
                WHERE schema_name = $1
                ORDER BY schema_id, created_at DESC
            "
        );

        let row = ctx
            .txns()
            .await?
            .pg()
            .query_opt(&query, &[&schema_name])
            .await?;
        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> CachedModuleResult<Vec<CachedModule>> {
        let query = format!(
            "
                SELECT
                    {CACHED_MODULE_LIST_FIELDS}
                FROM cached_modules
                WHERE schema_id = $1
                ORDER BY schema_id, created_at DESC
            "
        );

        let rows = ctx.txns().await?.pg().query(&query, &[&schema_id]).await?;
        rows.into_iter().map(TryInto::try_into).try_collect()
    }

    // TODO most likely, we should always be including user modules
    pub async fn latest_user_independent_modules(
        ctx: &DalContext,
    ) -> CachedModuleResult<Vec<CachedModule>> {
        let query = format!(
            "
                SELECT DISTINCT ON (schema_id)
                    {CACHED_MODULE_LIST_FIELDS}
                FROM cached_modules
                WHERE scoped_to_user_pk IS NULL
                ORDER BY schema_id, created_at DESC
            "
        );

        let rows = ctx.txns().await?.pg().query(&query, &[]).await?;
        rows.into_iter().map(TryInto::try_into).try_collect()
    }

    pub async fn latest_modules(ctx: &DalContext) -> CachedModuleResult<Vec<CachedModule>> {
        let HistoryActor::User(user_pk) = ctx.history_actor() else {
            return Self::latest_user_independent_modules(ctx).await;
        };

        let query = format!(
            "
                SELECT DISTINCT ON (schema_id)
                    {CACHED_MODULE_LIST_FIELDS}
                FROM cached_modules
                WHERE scoped_to_user_pk IS NULL OR scoped_to_user_pk = $1
                ORDER BY schema_id, created_at DESC
            "
        );

        let rows = ctx.txns().await?.pg().query(&query, &[&user_pk]).await?;
        rows.into_iter().map(TryInto::try_into).try_collect()
    }

    async fn insert(
        ctx: &DalContext,
        module_details: &ModuleDetailsResponse,
        pkg_bytes: Arc<Vec<u8>>,
        scoped_to_user_pk: Option<UserPk>,
    ) -> CachedModuleResult<Option<Self>> {
        let query = format!(
            "
                INSERT INTO cached_modules (
                    schema_id,
                    schema_name,
                    display_name,
                    category,
                    link,
                    color,
                    description,
                    component_type,
                    package_summary,
                    latest_hash,
                    created_at,
                    package_data,
                    scoped_to_user_pk
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10, $11, $12, $13
                ) RETURNING
                    {CACHED_MODULE_LIST_FIELDS}
            "
        );

        let Some(schema_id) = module_details.schema_id() else {
            warn!("builtin module {} has no schema id", module_details.id);
            return Ok(None);
        };
        let schema_id: SchemaId = schema_id.into();

        let Some(package) = PackageData::load(&module_details.id, &pkg_bytes).await? else {
            return Ok(None);
        };

        info!(
            "{} for {} - {} ({:?})",
            if scoped_to_user_pk.is_some() {
                "Creating private module cache entry"
            } else {
                "Updating sdf module cache"
            },
            module_details.name,
            package
                .schema_name()
                .unwrap_or(module_details.name.as_str()),
            package.category(),
        );

        let bytes_ref = pkg_bytes.as_slice();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                &query,
                &[
                    &schema_id,
                    &package
                        .schema_name()
                        .unwrap_or(module_details.name.as_str()),
                    &package.display_name(),
                    &package.category(),
                    &package.link(),
                    &package.color(),
                    &package.description(),
                    &package.component_type().to_string(),
                    &package.package_summary(),
                    &module_details.latest_hash,
                    &module_details.created_at,
                    &bytes_ref,
                    &scoped_to_user_pk,
                ],
            )
            .await?;

        Ok(Some(row.try_into()?))
    }

    pub async fn remove_all_for_schema_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> CachedModuleResult<Vec<CachedModuleId>> {
        let query = r#"
            DELETE FROM cached_modules
            WHERE schema_id = $1
            RETURNING id
            "#;

        let rows = ctx.txns().await?.pg().query(query, &[&schema_id]).await?;

        let mut deleted_ids = Vec::with_capacity(rows.len());

        for row in rows {
            deleted_ids.push(row.try_get("id")?);
        }

        Ok(deleted_ids)
    }

    // module_id is just for debugging purposes; we only want one record per hash
    pub async fn update_missing_package_summaries(ctx: &DalContext) -> CachedModuleResult<()> {
        // The inner query narrows the search down to only the latest hash for each module;
        // we'd be here forever if we tried to update old versions that don't matter anymore.
        let query = "
            SELECT DISTINCT ON (latest_hash)
                id,
                latest_hash
            FROM (
                SELECT DISTINCT ON (schema_id)
                    id,
                    latest_hash,
                    created_at,
                    package_summary
                FROM cached_modules
                ORDER BY schema_id, created_at DESC
            ) AS latest_modules
            WHERE package_summary IS NULL AND latest_hash IS NOT NULL
            ORDER BY latest_hash, created_at DESC
        ";

        let hashes_without_summaries = ctx.txns().await?.pg().query(query, &[]).await?;
        if hashes_without_summaries.is_empty() {
            return Ok(());
        }
        info!(
            "Updating {} hashes without summaries",
            hashes_without_summaries.len()
        );
        for hash_without_summary in hashes_without_summaries {
            let module_id: String = hash_without_summary.try_get("id")?;
            let hash: String = hash_without_summary.try_get("latest_hash")?;
            Self::update_package_summary_for_hash(ctx, &module_id, &hash).await?;
            ctx.commit_no_rebase().await?;
        }

        Ok(())
    }

    async fn update_package_summary_for_hash(
        ctx: &DalContext,
        module_id: &str, // For debug messages to make it easier to find where it came from
        hash: &str,
    ) -> CachedModuleResult<()> {
        let Some(pkg_bytes) = Self::package_data_for_hash(ctx, hash).await? else {
            // This shouldn't happen because we already checked for package_data IS NOT NULL,
            // but if it does, it's fine, we'll just skip on the assumption it won't
            // happen again.
            return Ok(());
        };
        // This is more problematic because we'll end up retrying summaries all the time
        let Some(summary) = PackageData::load(module_id, &Arc::new(pkg_bytes)).await? else {
            return Ok(());
        };
        let query = "
            UPDATE cached_modules
            SET package_summary = $1
            WHERE latest_hash = $2
        ";
        info!(
            "module {} ({}): {}",
            module_id,
            summary.schema_name().unwrap_or("<no name in package>"),
            summary.package_summary()
        );
        ctx.txns()
            .await?
            .pg()
            .execute(query, &[&summary.package_summary(), &hash])
            .await?;
        Ok(())
    }

    async fn package_data_for_hash(
        ctx: &DalContext,
        hash: &str,
    ) -> CachedModuleResult<Option<Vec<u8>>> {
        let query = "
            SELECT DISTINCT package_data
            FROM cached_modules
            WHERE latest_hash = $1 AND package_data IS NOT NULL
            LIMIT 1
        ";
        let Some(matching_data) = ctx.txns().await?.pg().query_opt(query, &[&hash]).await? else {
            return Ok(None);
        };
        Ok(matching_data.try_get("package_data")?)
    }
}

struct PackageData {
    schema: Option<SiPkgSchemaData>,
    variant: Option<SiPkgSchemaVariantData>,
    package_summary: serde_json::Value,
}

impl PackageData {
    async fn load(
        module_id: &str, // just for debug messages so we can find the broken rows
        pkg_bytes: &Arc<Vec<u8>>,
    ) -> CachedModuleResult<Option<Self>> {
        let pkg_bytes = pkg_bytes.clone();
        let pkg = slow_rt::spawn(async move { SiPkg::load_from_bytes(&pkg_bytes) })?.await??;

        let Some(schema) = pkg.schemas()?.into_iter().next() else {
            warn!("builtin module {} has no schema", module_id);
            return Ok(None);
        };

        let Some(variant) = schema.variants()?.into_iter().next() else {
            warn!("builtin module {} has a schema with no variant", module_id);
            return Ok(None);
        };

        let package_summary = PackageSummary {
            socket_count: variant.sockets()?.len() as u32
                + variant.management_funcs()?.len().max(1) as u32,
        };

        Ok(Some(Self {
            schema: schema.data,
            variant: variant.data,
            package_summary: serde_json::to_value(&package_summary)?,
        }))
    }

    fn schema_name(&self) -> Option<&str> {
        self.schema.as_ref().map(|data| data.name())
    }
    fn display_name(&self) -> Option<&str> {
        self.schema.as_ref().and_then(|data| data.category_name())
    }
    fn category(&self) -> &str {
        self.schema
            .as_ref()
            .map(|data| data.category())
            .unwrap_or("")
    }
    fn link(&self) -> Option<String> {
        self.variant
            .as_ref()
            .and_then(|data| data.link().map(ToString::to_string))
    }
    fn color(&self) -> Option<&str> {
        self.variant.as_ref().and_then(|data| data.color())
    }
    fn description(&self) -> Option<&str> {
        self.variant.as_ref().and_then(|data| data.description())
    }
    fn component_type(&self) -> ComponentType {
        self.variant
            .as_ref()
            .map(|data| data.component_type().into())
            .unwrap_or_default()
    }
    fn package_summary(&self) -> &serde_json::Value {
        &self.package_summary
    }
}
