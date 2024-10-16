//! A [`ManagementPrototype`] points to a Management [`Func`] for a schema variant

use std::{collections::HashSet, sync::Arc};

use serde::{Deserialize, Serialize};
use si_events::FuncRunId;
use thiserror::Error;
use veritech_client::{ComponentKind, ManagementResultSuccess};

use crate::{
    func::runner::{FuncRunner, FuncRunnerError},
    id, implement_add_edge_to,
    layer_db_types::{ManagementPrototypeContent, ManagementPrototypeContentV1},
    workspace_snapshot::node_weight::{traits::SiVersionedNodeWeight, NodeWeight},
    Component, ComponentError, ComponentId, DalContext, EdgeWeightKind,
    EdgeWeightKindDiscriminants, FuncId, HelperError, NodeWeightDiscriminants, SchemaId,
    SchemaVariant, SchemaVariantError, SchemaVariantId, TransactionsError, WorkspaceSnapshotError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementPrototypeError {
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("func runner recv error")]
    FuncRunnerRecvError,
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("layer db error: {0}")]
    LayerDbError(#[from] si_layer_cache::LayerDbError),
    #[error("management prototype {0} has no use edge to a function")]
    MissingFunction(ManagementPrototypeId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("too few variants: {0}")]
    TooFewVariants(ManagementPrototypeId),
    #[error("too many variants: {0}")]
    TooManyVariants(ManagementPrototypeId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type ManagementPrototypeResult<T> = Result<T, ManagementPrototypeError>;

id!(ManagementPrototypeId);

impl From<ManagementPrototypeId> for si_events::ManagementPrototypeId {
    fn from(value: ManagementPrototypeId) -> Self {
        si_events::ManagementPrototypeId::from_raw_id(value.into())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagementPrototype {
    pub id: ManagementPrototypeId,
    pub managed_schemas: Option<HashSet<SchemaId>>,
    pub name: String,
    pub description: Option<String>,
}

impl ManagementPrototype {
    pub async fn new(
        ctx: &DalContext,
        name: String,
        description: Option<String>,
        func_id: FuncId,
        managed_schemas: Option<HashSet<SchemaId>>,
        schema_variant_id: SchemaVariantId,
    ) -> ManagementPrototypeResult<Self> {
        let content = ManagementPrototypeContentV1 {
            name: name.clone(),
            managed_schemas: managed_schemas
                .clone()
                .map(|schemas| schemas.into_iter().map(Into::into).collect()),
            description: description.clone(),
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ManagementPrototypeContent::V1(content).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;

        let node_weight = NodeWeight::new_management_prototype(id, lineage_id, hash);
        workspace_snapshot.add_or_replace_node(node_weight).await?;

        Self::add_edge_to_func(ctx, id.into(), func_id, EdgeWeightKind::new_use()).await?;
        SchemaVariant::add_edge_to_management_prototype(
            ctx,
            schema_variant_id,
            id.into(),
            EdgeWeightKind::ManagementPrototype,
        )
        .await?;

        Ok(ManagementPrototype {
            id: id.into(),
            name,
            managed_schemas,
            description,
        })
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        id: ManagementPrototypeId,
    ) -> ManagementPrototypeResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let Some(idx) = workspace_snapshot.get_node_index_by_id_opt(id).await else {
            return Ok(None);
        };

        let NodeWeight::ManagementPrototype(inner) =
            workspace_snapshot.get_node_weight(idx).await?
        else {
            return Ok(None);
        };

        let content_hash = inner.content_hash();
        let content: ManagementPrototypeContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        let ManagementPrototypeContent::V1(content_inner) = content;

        Ok(Some(Self {
            id,
            managed_schemas: content_inner
                .managed_schemas
                .map(|schemas| schemas.into_iter().map(Into::into).collect()),
            name: content_inner.name,
            description: content_inner.description,
        }))
    }

    pub async fn func_id(
        ctx: &DalContext,
        id: ManagementPrototypeId,
    ) -> ManagementPrototypeResult<FuncId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let node_weight_id = node_weight.id();
            if NodeWeightDiscriminants::Func == node_weight.into() {
                return Ok(node_weight_id.into());
            }
        }

        Err(ManagementPrototypeError::MissingFunction(id))
    }

    pub async fn execute(
        &self,
        ctx: &DalContext,
        manager_component_id: ComponentId,
    ) -> ManagementPrototypeResult<(Option<ManagementResultSuccess>, FuncRunId)> {
        Self::execute_by_id(ctx, self.id, manager_component_id).await
    }

    pub async fn execute_by_id(
        ctx: &DalContext,
        id: ManagementPrototypeId,
        manager_component_id: ComponentId,
    ) -> ManagementPrototypeResult<(Option<ManagementResultSuccess>, FuncRunId)> {
        let management_func_id = ManagementPrototype::func_id(ctx, id).await?;
        let manager_component = Component::get_by_id(ctx, manager_component_id).await?;
        let manager_component_view = manager_component.view(ctx).await?;
        let geometry = manager_component.geometry(ctx).await?;

        let args = serde_json::json!({
            "this_component": {
                "kind": ComponentKind::Standard,
                "properties": manager_component_view,
                "geometry": geometry,
            }
        });

        let result_channel =
            FuncRunner::run_management(ctx, manager_component_id, management_func_id, args).await?;

        let run_value = result_channel
            .await
            .map_err(|_| ManagementPrototypeError::FuncRunnerRecvError)??;

        let func_run_id = run_value.func_run_id();
        let maybe_value: Option<si_events::CasValue> =
            run_value.value().cloned().map(|value| value.into());

        let maybe_value_address = match maybe_value {
            Some(value) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )
                    .await?
                    .0,
            ),
            None => None,
        };

        ctx.layer_db()
            .func_run()
            .set_values_and_set_state_to_success(
                func_run_id,
                None,
                maybe_value_address,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let maybe_run_result: Option<ManagementResultSuccess> = match run_value.value() {
            Some(value) => Some(serde_json::from_value(value.clone())?),
            None => None,
        };

        Ok((maybe_run_result, func_run_id))
    }

    pub async fn get_schema_variant_id(
        ctx: &DalContext,
        management_prototype_id: ManagementPrototypeId,
    ) -> ManagementPrototypeResult<SchemaVariantId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let node_indexes = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                management_prototype_id,
                EdgeWeightKindDiscriminants::ManagementPrototype,
            )
            .await?;
        if node_indexes.len() > 1 {
            return Err(ManagementPrototypeError::TooManyVariants(
                management_prototype_id,
            ));
        }
        let Some(node_index) = node_indexes.first() else {
            return Err(ManagementPrototypeError::TooFewVariants(
                management_prototype_id,
            ));
        };
        let node_weight = workspace_snapshot.get_node_weight(*node_index).await?;
        let schema_variant_id = node_weight.id();

        Ok(schema_variant_id.into())
    }

    pub async fn list_for_variant_id(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ManagementPrototypeResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut management_prototypes = Vec::new();
        for node_index in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::ManagementPrototype,
            )
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let node_weight_id = node_weight.id();
            if NodeWeightDiscriminants::ManagementPrototype == node_weight.into() {
                if let Some(management_prototype) =
                    Self::get_by_id(ctx, node_weight_id.into()).await?
                {
                    management_prototypes.push(management_prototype);
                }
            }
        }

        Ok(management_prototypes)
    }

    pub async fn list_ids_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> ManagementPrototypeResult<Vec<ManagementPrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut management_prototype_ids = Vec::new();
        for node_index in workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
            let node_weight_id = node_weight.id();
            if NodeWeightDiscriminants::ManagementPrototype == node_weight.into() {
                if let Some(management_prototype) =
                    Self::get_by_id(ctx, node_weight_id.into()).await?
                {
                    management_prototype_ids.push(management_prototype.id);
                }
            }
        }

        Ok(management_prototype_ids)
    }

    pub async fn remove(
        ctx: &DalContext,
        management_prototype_id: ManagementPrototypeId,
    ) -> ManagementPrototypeResult<()> {
        ctx.workspace_snapshot()?
            .remove_node_by_id(management_prototype_id)
            .await?;

        Ok(())
    }

    implement_add_edge_to!(
        source_id: ManagementPrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ManagementPrototypeResult,
    );
}
