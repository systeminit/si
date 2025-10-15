use std::{
    collections::HashMap,
    fmt::Debug,
    sync::Arc,
};

use petgraph::Outgoing;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
};
use si_id::{
    ActionPrototypeId,
    ManagementPrototypeId,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
pub use variant::{
    SchemaVariant,
    SchemaVariantId,
};

use crate::{
    DalContext,
    FuncError,
    FuncId,
    HelperError,
    SchemaVariantError,
    TransactionsError,
    action::prototype::{
        ActionPrototype,
        ActionPrototypeError,
    },
    cached_module::{
        CachedModule,
        CachedModuleError,
    },
    change_set::ChangeSetError,
    implement_add_edge_to,
    layer_db_types::{
        SchemaContent,
        SchemaContentDiscriminants,
        SchemaContentV1,
    },
    management::prototype::{
        ManagementPrototype,
        ManagementPrototypeError,
    },
    pkg::{
        ImportOptions,
        PkgError,
        import_pkg_from_pkg,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        edge_weight::{
            EdgeWeight,
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        node_weight::{
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
        },
    },
};

pub mod variant;

pub const SCHEMA_VERSION: SchemaContentDiscriminants = SchemaContentDiscriminants::V1;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("default schema variant not found for schema: {0}")]
    DefaultSchemaVariantNotFound(SchemaId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] Box<ManagementPrototypeError>),
    #[error("No default schema variant exists for {0}")]
    NoDefaultSchemaVariant(SchemaId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("No schema variant with name {0}")]
    NoSchemaVariantWithName(String),
    #[error("pkg error: {0}")]
    Pkg(#[from] Box<PkgError>),
    #[error("No schema installed after successful package import for {0}")]
    SchemaNotInstalledAfterImport(SchemaId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("uninstalled schema {0} not found")]
    UninstalledSchemaNotFound(SchemaId),
    #[error("uninstalled schema {0} not found")]
    UninstalledSchemaNotFoundByName(String),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

impl From<PkgError> for SchemaError {
    fn from(value: PkgError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for SchemaError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

pub type SchemaResult<T> = Result<T, SchemaError>;

pub use si_id::SchemaId;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Schema {
    id: SchemaId,
    #[serde(flatten)]
    timestamp: Timestamp,
    pub name: String,
    pub ui_hidden: bool,
    pub is_builtin: bool,
}

impl From<Schema> for SchemaContentV1 {
    fn from(value: Schema) -> Self {
        Self {
            timestamp: value.timestamp,
            name: value.name,
            ui_hidden: value.ui_hidden,
            is_builtin: value.is_builtin,
        }
    }
}

impl Schema {
    pub fn assemble(id: SchemaId, inner: SchemaContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            ui_hidden: inner.ui_hidden,
            is_builtin: inner.is_builtin,
        }
    }

    pub fn id(&self) -> SchemaId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn is_builtin(&self) -> bool {
        self.is_builtin
    }
    pub fn ui_hidden(&self) -> bool {
        self.ui_hidden
    }

    implement_add_edge_to!(
        source_id: SchemaId,
        destination_id: SchemaVariantId,
        add_fn: add_edge_to_variant,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: SchemaResult,
    );

    implement_add_edge_to!(
        source_id: SchemaId,
        destination_id: ActionPrototypeId,
        add_fn: add_edge_to_action_prototype,
        discriminant: EdgeWeightKindDiscriminants::ActionPrototype,
        result: SchemaResult,
    );

    implement_add_edge_to!(
        source_id: SchemaId,
        destination_id: ManagementPrototypeId,
        add_fn: add_edge_to_management_prototype,
        discriminant: EdgeWeightKindDiscriminants::ManagementPrototype,
        result: SchemaResult,
    );

    pub async fn new(ctx: &DalContext, name: impl Into<String>) -> SchemaResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        Self::new_with_id(ctx, id.into(), name).await
    }

    pub async fn new_with_id(
        ctx: &DalContext,
        id: SchemaId,
        name: impl Into<String>,
    ) -> SchemaResult<Self> {
        let content = SchemaContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            ui_hidden: false,
            is_builtin: false,
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(SchemaContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        // Lineage id has to match id here, otherwise every new schema will be
        // treated as a new node, even though it has the same id
        let node_weight =
            NodeWeight::new_content(id.into(), id.into(), ContentAddress::Schema(hash));

        workspace_snapshot.add_or_replace_node(node_weight).await?;

        let schema_category_index_id = workspace_snapshot
            .get_category_node_or_err(CategoryNodeKind::Schema)
            .await?;
        workspace_snapshot
            .add_edge(
                schema_category_index_id,
                EdgeWeight::new(EdgeWeightKind::new_use()),
                id,
            )
            .await?;

        Ok(Self::assemble(id, content))
    }

    pub async fn all_overlay_func_ids(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaResult<Vec<FuncId>> {
        let mut func_ids = vec![];
        let action_prototypes = ActionPrototype::for_schema(ctx, schema_id)
            .await
            .map_err(Box::new)?;
        for action_proto in action_prototypes {
            let func_id = ActionPrototype::func_id(ctx, action_proto.id())
                .await
                .map_err(Box::new)?;
            func_ids.push(func_id);
        }

        let management_prototypes = ManagementPrototype::list_for_schema_id(ctx, schema_id)
            .await
            .map_err(Box::new)?;
        for management_proto in management_prototypes {
            let func_id = ManagementPrototype::func_id(ctx, management_proto.id())
                .await
                .map_err(Box::new)?;
            func_ids.push(func_id);
        }

        Ok(func_ids)
    }

    pub async fn default_variant_id(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaResult<SchemaVariantId> {
        Self::default_variant_id_opt(ctx, schema_id)
            .await?
            .ok_or(SchemaError::DefaultSchemaVariantNotFound(schema_id))
    }

    pub async fn default_variant_id_opt(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaResult<Option<SchemaVariantId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let default_schema_variant_node_indicies = workspace_snapshot
            .edges_directed(schema_id, Outgoing)
            .await?;

        for (edge_weight, _, target_index) in default_schema_variant_node_indicies {
            if *edge_weight.kind() == EdgeWeightKind::new_use_default() {
                return Ok(Some(
                    workspace_snapshot
                        .get_node_weight(target_index)
                        .await?
                        .id()
                        .into(),
                ));
            }
        }

        Ok(None)
    }

    /// This method returns all [`SchemaVariantIds`](SchemaVariant) that are used by the [`Schema`]
    /// corresponding to the [`SchemaId`](Schema) passed in. This method will also include the
    /// default [`SchemaVariantId`](SchemaVariantId), if one exists.
    pub async fn list_schema_variant_ids(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaResult<Vec<SchemaVariantId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let schema_variant_ids: Vec<_> = workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                schema_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
            .into_iter()
            .map(|(_, _, target_id)| target_id.into())
            .collect();

        Ok(schema_variant_ids)
    }

    pub async fn set_default_variant_id(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        debug!(
            %schema_variant_id, "setting the default schema variant for schema: {}",
            self.id
        );

        // Our system will have edges as follows:
        //
        // Schema -> Use -> Schema Variant
        //
        // In order to make a schema variant the default for a schema, we need
        // to update the correct edge from Use to the default variant of Use,
        //
        // Schema -> Use {is_default = true} -> Schema Variant
        //
        // Therefore, when we are setting a default schema variant we need to
        // find any existing default Use edges and convert them back to uses AND we
        // need to find the existing Use edge between our nodes and change that
        // to be a default Use
        for (edge_weight, source_index, target_index) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                self.id,
                Outgoing,
                EdgeWeightKind::new_use_default().into(),
            )
            .await?
        {
            // We have found the existing Default edge between schema and schema variant
            // we now need to update that edge to be a Use
            workspace_snapshot
                .remove_edge(source_index, target_index, edge_weight.kind().into())
                .await?;

            Self::add_edge_to_variant(
                ctx,
                self.id,
                workspace_snapshot
                    .get_node_weight(target_index)
                    .await?
                    .id()
                    .into(),
                EdgeWeightKind::new_use(),
            )
            .await?;
        }

        workspace_snapshot
            .remove_edge(self.id, schema_variant_id, EdgeWeightKind::new_use().into())
            .await?;

        Self::add_edge_to_variant(
            ctx,
            self.id,
            schema_variant_id,
            EdgeWeightKind::new_use_default(),
        )
        .await?;

        Ok(())
    }

    /// Returns whether or not the [`Schema`] exists locally. This works because
    /// [`Schemas`](Schema) are unique across workspaces.
    pub async fn exists_locally(ctx: &DalContext, id: SchemaId) -> SchemaResult<bool> {
        Ok(ctx.workspace_snapshot()?.node_exists(id).await)
    }

    pub async fn get_by_id_opt(ctx: &DalContext, id: SchemaId) -> SchemaResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let Some(node_weight) = workspace_snapshot.get_node_weight_opt(id).await else {
            return Ok(None);
        };
        let hash = node_weight.content_hash();

        let content: SchemaContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let SchemaContent::V1(inner) = content;

        Ok(Some(Self::assemble(id, inner)))
    }

    pub async fn get_by_id(ctx: &DalContext, id: SchemaId) -> SchemaResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_weight = workspace_snapshot.get_node_weight(id).await?;
        let hash = node_weight.content_hash();

        let content: SchemaContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let SchemaContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> SchemaResult<Self>
    where
        L: FnOnce(&mut Self) -> SchemaResult<()>,
    {
        let mut schema = self;

        let before = SchemaContentV1::from(schema.clone());
        lambda(&mut schema)?;
        let updated = SchemaContentV1::from(schema.clone());

        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(SchemaContent::V1(updated.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;

            ctx.workspace_snapshot()?
                .update_content(schema.id.into(), hash)
                .await?;
        }

        Ok(schema)
    }

    pub async fn list(ctx: &DalContext) -> SchemaResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut schemas = vec![];
        let schema_category_index_id = workspace_snapshot
            .get_category_node_or_err(CategoryNodeKind::Schema)
            .await?;

        let schema_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_category_index_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut schema_node_weights = vec![];
        let mut schema_content_hashes = vec![];
        for index in schema_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::Schema)?;
            schema_content_hashes.push(node_weight.content_hash());
            schema_node_weights.push(node_weight);
        }

        let schema_contents: HashMap<ContentHash, SchemaContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(schema_content_hashes.as_slice())
            .await?;

        for node_weight in schema_node_weights {
            match schema_contents.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let SchemaContent::V1(inner) = content;

                    schemas.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(schemas)
    }

    /// Lists all [`Schemas`](Schema) by ID in the workspace.
    pub async fn list_ids(ctx: &DalContext) -> SchemaResult<Vec<SchemaId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let schema_category_index_id = workspace_snapshot
            .get_category_node_or_err(CategoryNodeKind::Schema)
            .await?;
        let schema_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_category_index_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut schema_ids = Vec::with_capacity(schema_node_indices.len());
        for index in schema_node_indices {
            let raw_id = workspace_snapshot.get_node_weight(index).await?.id();
            schema_ids.push(raw_id.into());
        }

        Ok(schema_ids)
    }

    // NOTE(nick): this assumes that schema names are unique.
    pub async fn get_by_name_opt(
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> SchemaResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let schema_node_indices = {
            let schema_category_index_id = workspace_snapshot
                .get_category_node_or_err(CategoryNodeKind::Schema)
                .await?;
            workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    schema_category_index_id,
                    EdgeWeightKindDiscriminants::Use,
                )
                .await?
        };

        // NOTE(nick): this algorithm could be better.
        for schema_node_index in schema_node_indices {
            let schema_node_weight = {
                workspace_snapshot
                    .get_node_weight(schema_node_index)
                    .await?
                    .get_content_node_weight_of_kind(ContentAddressDiscriminants::Schema)?
            };
            let schema = Self::get_by_id(ctx, schema_node_weight.id().into()).await?;
            if schema.name == name.as_ref() {
                return Ok(Some(schema));
            }
        }
        Ok(None)
    }

    pub async fn get_by_name(ctx: &DalContext, name: impl AsRef<str>) -> SchemaResult<Self> {
        Self::get_by_name_opt(ctx, name.as_ref())
            .await?
            .ok_or_else(|| SchemaError::NoSchemaVariantWithName(name.as_ref().to_string()))
    }

    pub async fn get_or_install_by_name(ctx: &DalContext, name: &str) -> SchemaResult<Schema> {
        // If there's an installed schema, return it
        Ok(match Self::get_by_name_opt(ctx, name).await? {
            Some(schema) => schema,
            None => {
                let uninstalled_module = CachedModule::find_latest_for_schema_name(ctx, name)
                    .await?
                    .ok_or(SchemaError::UninstalledSchemaNotFoundByName(name.into()))?;

                Self::install_from_module(ctx, uninstalled_module).await?
            }
        })
    }

    pub async fn is_name_taken(ctx: &DalContext, name: &String) -> SchemaResult<bool> {
        Ok(Self::list(ctx).await?.iter().any(|s| s.name.eq(name)))
    }

    /// Returns the default [`SchemaVariantId`] for the provided [`SchemaId`]
    /// *if* this schema is installed. If this schema is not installed, it looks
    /// for it in the local module cache, and if it exists there, it installs it, then
    /// returns the newly installed default [`SchemaVariantId`].
    #[instrument(
        name = "schema.get_or_install_default_variant",
        level = "info",
        skip_all
    )]
    pub async fn get_or_install_default_variant(
        ctx: &DalContext,
        schema_id: SchemaId,
    ) -> SchemaResult<SchemaVariantId> {
        Self::ensure_installed(ctx, schema_id).await?;
        Self::default_variant_id(ctx, schema_id).await
    }

    async fn ensure_installed(ctx: &DalContext, schema_id: SchemaId) -> SchemaResult<()> {
        // Install the schema, if it isn't already
        if !Self::exists_locally(ctx, schema_id).await? {
            let module = CachedModule::find_latest_for_schema_id(ctx, schema_id)
                .await?
                .ok_or(SchemaError::UninstalledSchemaNotFound(schema_id))?;
            Self::install_from_module(ctx, module).await?;
        }
        Ok(())
    }

    #[instrument(name = "schema.install_from_module", level = "info", skip_all)]
    async fn install_from_module(
        ctx: &DalContext,
        mut module: CachedModule,
    ) -> SchemaResult<Schema> {
        let si_pkg = module.si_pkg(ctx).await?;
        import_pkg_from_pkg(
            ctx,
            &si_pkg,
            Some(ImportOptions {
                schema_id: Some(module.schema_id.into()),
                ..Default::default()
            }),
        )
        .await?;
        Self::get_by_id_opt(ctx, module.schema_id)
            .await?
            .ok_or(SchemaError::UninstalledSchemaNotFound(module.schema_id))
    }
}
