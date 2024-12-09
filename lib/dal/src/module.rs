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
use crate::pkg::export::PkgExporter;
use crate::pkg::PkgError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::traits::SiNodeWeight;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    ChangeSetError, DalContext, Func, FuncError, HistoryActor, Schema, SchemaError, SchemaId,
    SchemaVariant, SchemaVariantError, SchemaVariantId, Timestamp, TransactionsError, User,
    UserError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("found empty metadata (name: '{0}') (version: '{1}')")]
    EmptyMetadata(String, String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("module missing schema id (module id: {0}) (module hash: {1})")]
    MissingSchemaId(String, String),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("pkg error: {0}")]
    Pkg(#[from] Box<PkgError>),
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
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ModuleResult<T> = Result<T, ModuleError>;

pub use si_id::ModuleId;

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

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ModuleContent::V2(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;
        let node_weight = NodeWeight::new_content(id, lineage_id, ContentAddress::Module(hash));

        workspace_snapshot.add_or_replace_node(node_weight).await?;

        let schema_module_index_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Module)
            .await?;
        workspace_snapshot
            .add_edge(
                schema_module_index_id,
                EdgeWeight::new(EdgeWeightKind::new_use()),
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
                EdgeWeight::new(EdgeWeightKind::new_use()),
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
                    let schema = Schema::get_by_id_or_error(ctx, inner.id().into()).await?;
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
            if let NodeWeight::SchemaVariant(variant_weight) = &node_weight {
                let variant =
                    SchemaVariant::get_by_id_or_error(ctx, variant_weight.id().into()).await?;
                all_schema_variants.push(variant);
            } else if let NodeWeight::Content(inner) = &node_weight {
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
        builtin_modules: Vec<frontend_types::ModuleDetails>,
        all_modules: Vec<frontend_types::ModuleDetails>,
    ) -> ModuleResult<frontend_types::SyncedModules> {
        let start = Instant::now();

        // Initialize result struct
        let mut synced_modules = frontend_types::SyncedModules::new();

        // Collect all user facing schema variants. We need to see what can be upgraded.
        let schema_variants = SchemaVariant::list_user_facing(ctx).await?;

        // Check the locally found schema_variant_ids to see if it's contributable
        // Contributable means that it's not avilable in the module index NOR is it a builtin
        // we check it's a builtin because it's hash would be in the past_hashes_by_module_id
        for schema_variant in &schema_variants {
            let schema_variant_id: SchemaVariantId = schema_variant.schema_variant_id;
            let schema_id: SchemaId = schema_variant.schema_id;
            let is_default = SchemaVariant::is_default_by_id(ctx, schema_variant_id).await?;
            let is_locked = SchemaVariant::is_locked_by_id(ctx, schema_variant_id).await?;

            let module = Module::find_for_module_schema_id(ctx, schema_id.into())
                .await
                .map_err(|e| SchemaVariantError::Module(e.to_string()))?;
            if let Some(m) = module {
                if is_default && is_locked {
                    let matches_existing_builtin = builtin_modules
                        .iter()
                        .any(|md| md.latest_hash == m.root_hash);

                    let matches_existing_module = all_modules.iter().any(|md| {
                        md.schema_variant_id.as_ref() == Some(&schema_variant_id.to_string())
                            && md.schema_variant_version.as_ref() == Some(&schema_variant.version)
                    });

                    if !matches_existing_module && !matches_existing_builtin {
                        synced_modules.contributable.push(schema_variant_id)
                    }
                }
            }
        }

        // Build a list of the past hashes for all of the modules
        let past_hashes_for_module_id = builtin_modules
            .into_iter()
            .filter_map(|m| {
                if let Some(past_hashes) = m.past_hashes {
                    Some((m.id, HashSet::from_iter(past_hashes.into_iter())))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        // For each latest module
        // if not for existing schema, is installable
        // if for existing locked schema variant, and hash != hash for module of variant, mark variant as upgradeable
        // if for existing schema variants, but they are not upgradeable, do nothing
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

            // Compute upgradeable variants

            // Due to rust lifetimes, we need to actually create a set here to reference in unwrap.
            // The Alternative would be to clone the existing hashset every loop, which is way worse.
            let empty_set = HashSet::new();
            let past_hashes = past_hashes_for_module_id
                .get(&latest_module.id)
                .unwrap_or(&empty_set);

            // A module is for a schema if its schema id matches the other schema
            // or if the hash of the package of that schema is contained in the latest hashes list
            let mut possible_upgrade_targets = vec![];
            for schema_variant in &schema_variants {
                let this_schema_id: SchemaId = schema_variant.schema_id;
                let variant_id: SchemaVariantId = schema_variant.schema_variant_id;

                let Some(variant_module) = Self::find_for_member_id(ctx, variant_id).await? else {
                    continue;
                };

                if this_schema_id == schema_id || past_hashes.contains(&variant_module.root_hash) {
                    possible_upgrade_targets.push((schema_variant, variant_module));
                }
            }

            if possible_upgrade_targets.is_empty() {
                synced_modules.installable.push(latest_module.to_owned());
                continue;
            }

            for (upgradeable_variant, variant_module) in possible_upgrade_targets {
                if latest_module.latest_hash != variant_module.root_hash
                    && upgradeable_variant.is_locked
                {
                    synced_modules.upgradeable.insert(
                        upgradeable_variant.schema_variant_id,
                        latest_module.to_owned(),
                    );
                }
            }
        }

        debug!(?synced_modules.installable, "collected installable modules");
        debug!(?synced_modules.upgradeable, "collected upgradeable modules");
        debug!(?synced_modules.contributable, "collected contributable modules");
        debug!("syncing modules took: {:?}", start.elapsed());

        Ok(synced_modules)
    }

    /// Prepares a given [`SchemaId`] and its corresponding [`Module`] for contribution.
    #[allow(clippy::type_complexity)]
    #[instrument(
        name = "module.prepare_contribution"
        level = "info",
        skip_all,
        fields(
            name = name.as_ref(),
            version = version.as_ref(),
            %schema_variant_id
        )
    )]
    pub async fn prepare_contribution(
        ctx: &DalContext,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
        schema_variant_id: SchemaVariantId,
    ) -> ModuleResult<(
        String,
        String,
        Option<String>,
        Option<SchemaId>,
        Vec<u8>,
        String,
        String,
        String,
    )> {
        let user = match ctx.history_actor() {
            HistoryActor::User(user_pk) => User::get_by_pk(ctx, *user_pk).await?,
            _ => None,
        };
        let (created_by_name, created_by_email) = user
            .map(|user| (user.name().to_owned(), user.email().to_owned()))
            .unwrap_or((
                "unauthenticated user name".into(),
                "unauthenticated user email".into(),
            ));
        debug!(%created_by_name, %created_by_email, "preparing module contribution");

        // Sanitize and validate metadata.
        let name = name.as_ref().trim();
        let version = version.as_ref().trim();
        if name.is_empty() || version.is_empty() {
            return Err(ModuleError::EmptyMetadata(
                name.to_string(),
                version.to_string(),
            ));
        }

        // The frontend will send us the schema variant as this is what we care about from
        // there. We can then use that schema variant to be able to understand the associated
        // schema for it.
        let variant = SchemaVariant::get_by_id_or_error(ctx, schema_variant_id).await?;
        let associated_schema = variant.schema(ctx).await?;

        // Create module payload.
        let mut exporter = PkgExporter::new_for_module_contribution(
            name,
            version,
            &created_by_email,
            associated_schema.id(),
        );
        let module_payload = exporter.export_as_bytes(ctx).await.map_err(Box::new)?;

        // Check if local information exists for contribution metadata.
        let (local_module_based_on_hash, local_module_schema_id) =
            match Module::find_for_member_id(ctx, associated_schema.id()).await? {
                Some(module) => (
                    Some(module.root_hash().to_string()),
                    module.schema_id().map(|id| id.into()),
                ),
                None => (None, None),
            };

        Ok((
            name.to_string(),
            version.to_string(),
            local_module_based_on_hash,
            local_module_schema_id,
            module_payload,
            created_by_name,
            created_by_email,
            variant.version().to_string(),
        ))
    }
}
