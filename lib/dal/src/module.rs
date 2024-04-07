use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use si_events::ContentHash;
use thiserror::Error;
use tokio::sync::TryLockError;
use ulid::Ulid;

use si_layer_cache::LayerDbError;
use telemetry::prelude::*;

use crate::layer_db_types::{ModuleContent, ModuleContentV1};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, ChangeSetError, DalContext, Func, FuncError, Schema, SchemaError, SchemaVariant,
    SchemaVariantError, Timestamp, TransactionsError,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
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
}

impl Module {
    pub fn assemble(id: ModuleId, inner: ModuleContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            name: inner.name,
            root_hash: inner.root_hash,
            version: inner.version,
            description: inner.description,
            created_by_email: inner.created_by_email,
            created_at: inner.created_at,
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

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        root_hash: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        created_by_email: impl Into<String>,
        created_at: impl Into<DateTime<Utc>>,
    ) -> ModuleResult<Self> {
        let content = ModuleContentV1 {
            timestamp: Timestamp::now(),
            name: name.into(),
            root_hash: root_hash.into(),
            version: version.into(),
            description: description.into(),
            created_by_email: created_by_email.into(),
            created_at: created_at.into(),
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ModuleContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;

        let node_weight = NodeWeight::new_content(change_set, id, ContentAddress::Module(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        let schema_module_index_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::Module)
            .await?;
        workspace_snapshot
            .add_edge(
                schema_module_index_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
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
        let ModuleContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    pub async fn find_by_root_hash(
        ctx: &DalContext,
        root_hash: impl AsRef<str>,
    ) -> ModuleResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let module_node_indices = {
            let module_category_index_id = workspace_snapshot
                .get_category_node(None, CategoryNodeKind::Module)
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
            if module.root_hash == root_hash.as_ref() {
                return Ok(Some(module));
            }
        }

        Ok(None)
    }

    pub async fn create_association(&self, ctx: &DalContext, target_id: Ulid) -> ModuleResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot
            .add_edge(
                self.id,
                EdgeWeight::new(ctx.change_set()?, EdgeWeightKind::new_use())?,
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
                let func = Func::get_by_id(ctx, inner.id().into()).await?;
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
                    let variant = SchemaVariant::get_by_id(ctx, inner.id().into()).await?;
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
            .get_category_node(None, CategoryNodeKind::Module)
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
                Some(module_content) => {
                    let ModuleContent::V1(inner) = module_content;

                    modules.push(Self::assemble(node_weight.id().into(), inner.to_owned()))
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(modules)
    }
}
