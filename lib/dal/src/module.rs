use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use si_events::ContentHash;
use si_frontend_types as frontend_types;
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
use tokio::time::Instant;

use crate::layer_db_types::{ModuleContent, ModuleContentV2};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, ChangeSetError, DalContext, Func, FuncError, Schema, SchemaError, SchemaId, SchemaVariant,
    SchemaVariantError, Timestamp, TransactionsError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("module missing schema id (module id: {0}) (module hash: {1})")]
    MissingSchemaId(String, String),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("too many latest modules for schema: {0} (at least two hashes found: {1} and {2})")]
    TooManyLatestModulesForSchema(SchemaId, String, String),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ModuleResult<T> = Result<T, ModuleError>;

pk!(ModuleId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Module {
    id: ModuleId,
    #[serde(flatten)]
    timestamp: Timestamp,
    name: String,
    root_hash: String,
    version: String,
    description: String,
    created_by_email: String,
    created_at: DateTime<Utc>,
    schema_id: Option<Ulid>,
}

impl Module {
    pub fn assemble(id: ModuleId, inner: ModuleContentV2) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            root_hash: inner.root_hash,
            version: inner.version,
            description: inner.description,
            created_by_email: inner.created_by_email,
            created_at: inner.created_at,
            schema_id: inner.schema_id,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_by_email(&self) -> &str {
        &self.created_by_email
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn root_hash(&self) -> &str {
        &self.root_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// This is the "module" schema id. It's a unique id that all variants of a
    /// single schema get in the module index database. If this is the first
    /// time installing the asset, the schema will get this, but this is not
    /// guaranteed to be the id of the schema in workspaces that have assets
    /// installed before this feature was added!
    pub fn schema_id(&self) -> Option<Ulid> {
        self.schema_id
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        root_hash: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        created_by_email: impl Into<String>,
        created_at: impl Into<DateTime<Utc>>,
        schema_id: Option<Ulid>,
    ) -> ModuleResult<Self> {
        let content = ModuleContentV2 {
            timestamp: Timestamp::now(),
            name: name.into(),
            root_hash: root_hash.into(),
            version: version.into(),
            description: description.into(),
            created_by_email: created_by_email.into(),
            created_at: created_at.into(),
            schema_id,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ModuleContent::V2(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight = NodeWeight::new_content(
            ctx.vector_clock_id()?,
            id,
            lineage_id,
            ContentAddress::Module(hash),
        )?;

        workspace_snapshot.add_node(node_weight).await?;

        let schema_module_index_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Module)
            .await?;
        workspace_snapshot
            .add_edge(
                schema_module_index_id,
                EdgeWeight::new(ctx.vector_clock_id()?, EdgeWeightKind::new_use())?,
                id,
            )
            .await?;

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: ModuleId) -> ModuleResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: ModuleContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // Add any extra migrations here!
        let inner = match content {
            ModuleContent::V1(v1_inner) => v1_inner.into(),
            ModuleContent::V2(inner) => inner,
        };

        Ok(Self::assemble(id, inner))
    }

    pub async fn find<P>(ctx: &DalContext, predicate: P) -> ModuleResult<Option<Self>>
    where
        P: FnMut(&Module) -> bool,
    {
        let mut predicate = predicate;
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let module_node_indices = {
            let module_category_index_id = workspace_snapshot
                .get_category_node_or_err(None, CategoryNodeKind::Module)
                .await?;
            workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    module_category_index_id,
                    EdgeWeightKindDiscriminants::Use,
                )
                .await?
        };

        for module_node_index in module_node_indices {
            let module_node_weight = workspace_snapshot
                .get_node_weight(module_node_index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Module)?;

            let module: Module = Self::get_by_id(ctx, module_node_weight.id().into()).await?;
            if predicate(&module) {
                return Ok(Some(module));
            }
        }

        Ok(None)
    }

    pub async fn find_by_root_hash(
        ctx: &DalContext,
        root_hash: impl AsRef<str>,
    ) -> ModuleResult<Option<Self>> {
        Self::find(ctx, |module| module.root_hash() == root_hash.as_ref()).await
    }

    pub async fn find_for_module_schema_id(
        ctx: &DalContext,
        module_schema_id: Ulid,
    ) -> ModuleResult<Option<Self>> {
        Self::find(ctx, |module| module.schema_id() == Some(module_schema_id)).await
    }

    /// Find [Module](Self) based on the id of an entity that it contains. May return [None](None) if
    /// entity is not linked to a [Module](Self)
    pub async fn find_for_member_id(
        ctx: &DalContext,
        id: impl Into<Ulid>,
    ) -> ModuleResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for source_idx in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(source_idx).await?;
            if let NodeWeight::Content(content_node_weight) = node_weight {
                if ContentAddressDiscriminants::Module
                    == content_node_weight.content_address().into()
                {
                    let module = Self::get_by_id(ctx, content_node_weight.id().into()).await?;
                    return Ok(Some(module));
                }
            }
        }

        Ok(None)
    }

    pub async fn create_association(&self, ctx: &DalContext, target_id: Ulid) -> ModuleResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot
            .add_edge(
                self.id,
                EdgeWeight::new(ctx.vector_clock_id()?, EdgeWeightKind::new_use())?,
                target_id,
            )
            .await?;

        Ok(())
    }

    pub async fn list_associated_funcs(&self, ctx: &DalContext) -> ModuleResult<Vec<Func>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut all_funcs = vec![];

        let node_weights = workspace_snapshot.all_outgoing_targets(self.id).await?;
        for node_weight in node_weights {
            if let NodeWeight::Func(inner) = &node_weight {
                let func = Func::get_by_id_or_error(ctx, inner.id().into()).await?;
                all_funcs.push(func);
            }
        }

        Ok(all_funcs)
    }

    pub async fn list_associated_schemas(&self, ctx: &DalContext) -> ModuleResult<Vec<Schema>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut all_schemas = vec![];

        let node_weights = workspace_snapshot.all_outgoing_targets(self.id).await?;
        for node_weight in node_weights {
            if let NodeWeight::Content(inner) = &node_weight {
                let inner_addr_discrim: ContentAddressDiscriminants =
                    inner.content_address().into();

                if inner_addr_discrim == ContentAddressDiscriminants::Schema {
                    let schema = Schema::get_by_id(ctx, inner.id().into()).await?;
                    all_schemas.push(schema);
                }
            }
        }

        Ok(all_schemas)
    }

    pub async fn find_matching_module(&self, ctx: &DalContext) -> ModuleResult<Option<Self>> {
        let mut maybe_mod = None;

        if let Some(module_schema_id) = self.schema_id() {
            maybe_mod = Self::find_for_module_schema_id(ctx, module_schema_id).await?;
        }

        if maybe_mod.is_none() {
            maybe_mod = Self::find_by_root_hash(ctx, self.root_hash()).await?;
        }

        Ok(maybe_mod)
    }

    pub async fn list_associated_schema_variants(
        &self,
        ctx: &DalContext,
    ) -> ModuleResult<Vec<SchemaVariant>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut all_schema_variants = vec![];

        let node_weights = workspace_snapshot.all_outgoing_targets(self.id).await?;
        for node_weight in node_weights {
            if let NodeWeight::Content(inner) = &node_weight {
                let inner_addr_discrim: ContentAddressDiscriminants =
                    inner.content_address().into();

                if inner_addr_discrim == ContentAddressDiscriminants::SchemaVariant {
                    let variant = SchemaVariant::get_by_id_or_error(ctx, inner.id().into()).await?;
                    all_schema_variants.push(variant);
                }
            }
        }

        Ok(all_schema_variants)
    }

    pub async fn list_installed(ctx: &DalContext) -> ModuleResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut modules = vec![];
        let module_category_index_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Module)
            .await?;

        let module_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                module_category_index_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut content_hashes = vec![];
        for module_node_index in module_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(module_node_index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Module)?;
            content_hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let content_map: HashMap<ContentHash, ModuleContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(content_hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(module_content) => modules.push(Self::assemble(
                    node_weight.id().into(),
                    module_content.inner(),
                )),
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(modules)
    }

    /// Takes in a list of [`LatestModules`](si_frontend_types::LatestModule) and creates a
    /// [`SyncedModules`](si_frontend_types::SyncedModules) object with them. The object enables callers to know what
    /// [`Modules`](Module) can be upgraded and installed.
    #[instrument(
        name = "module.sync"
        level = "info",
        skip_all,
        fields(
            latest_modules_count = latest_modules.len()
        )
    )]
    pub async fn sync(
        ctx: &DalContext,
        latest_modules: Vec<frontend_types::LatestModule>,
    ) -> ModuleResult<frontend_types::SyncedModules> {
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
                match Self::find_for_member_id(ctx, schema_id).await? {
                    Some(found_local_module) => {
                        entry.insert(found_local_module.root_hash().to_owned());
                    }
                    None => {
                        error!(%schema_id, %schema_variant.schema_variant_id, "found orphaned schema (has no corresponding module)");
                    }
                }
            }
        }

        // Begin populating synced modules.
        let mut synced_modules = frontend_types::SyncedModules::new();

        // Group the latest hashes by schema. Populate installable modules along the way.
        let mut latest_modules_by_schema: HashMap<SchemaId, frontend_types::LatestModule> =
            HashMap::new();
        for latest_module in latest_modules {
            let schema_id: SchemaId = latest_module
                .schema_id()
                .ok_or(ModuleError::MissingSchemaId(
                    latest_module.id.to_owned(),
                    latest_module.latest_hash.to_owned(),
                ))?
                .into();
            match latest_modules_by_schema.entry(schema_id) {
                Entry::Occupied(entry) => {
                    let existing: frontend_types::LatestModule = entry.get().to_owned();
                    return Err(ModuleError::TooManyLatestModulesForSchema(
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
        debug!(?synced_modules.installable, "collected installable modules");

        // Populate upgradeable modules.
        for schema_variant in schema_variants {
            let schema_id: SchemaId = schema_variant.schema_id.into();
            match (
                latest_modules_by_schema.get(&schema_id),
                local_hashes.get(&schema_id),
            ) {
                (Some(latest_module), Some(local_hash)) => {
                    debug!(?latest_module, %local_hash, schema_variant.is_locked, "comparing hashes");
                    if &latest_module.latest_hash != local_hash && schema_variant.is_locked {
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
        debug!(?synced_modules.upgradeable, "collected upgradeable modules");

        info!("syncing modules took: {:?}", start.elapsed());

        Ok(synced_modules)
    }
}
