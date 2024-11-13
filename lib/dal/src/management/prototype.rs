//! A [`ManagementPrototype`] points to a Management [`Func`] for a schema variant

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use si_events::FuncRunId;
use thiserror::Error;
use veritech_client::ManagementResultSuccess;

use crate::diagram::view::View;
use crate::{
    cached_module::{CachedModule, CachedModuleError},
    func::runner::{FuncRunner, FuncRunnerError},
    id, implement_add_edge_to,
    layer_db_types::{ManagementPrototypeContent, ManagementPrototypeContentV1},
    workspace_snapshot::node_weight::{traits::SiVersionedNodeWeight, NodeWeight},
    Component, ComponentError, ComponentId, DalContext, EdgeWeightKind,
    EdgeWeightKindDiscriminants, FuncId, HelperError, NodeWeightDiscriminants, Schema, SchemaError,
    SchemaId, SchemaVariant, SchemaVariantError, SchemaVariantId, TransactionsError,
    WorkspaceSnapshotError, WsEventError, WsEventResult,
};
use crate::{diagram::DiagramError, WsEvent};

use super::NumericGeometry;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementPrototypeError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] CachedModuleError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("diagram error: {0}")]
    Diagram(#[from] DiagramError),
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
    #[error("management prototype {0} not found")]
    NotFound(ManagementPrototypeId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
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
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
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

impl From<ManagementPrototype> for ManagementPrototypeContent {
    fn from(value: ManagementPrototype) -> Self {
        Self::V1(ManagementPrototypeContentV1 {
            name: value.name,
            managed_schemas: value.managed_schemas,
            description: value.description,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ManagementPrototypeExecution {
    pub func_run_id: FuncRunId,
    pub result: Option<ManagementResultSuccess>,
    pub manager_component_geometry: NumericGeometry,
    pub managed_schema_map: HashMap<String, SchemaId>,
    pub placeholders: HashMap<String, ComponentId>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagedComponent {
    kind: String,
    properties: Option<serde_json::Value>,
    geometry: NumericGeometry,
}

impl ManagementPrototype {
    implement_add_edge_to!(
        source_id: ManagementPrototypeId,
        destination_id: FuncId,
        add_fn: add_edge_to_func,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ManagementPrototypeResult,
    );

    pub fn id(&self) -> ManagementPrototypeId {
        self.id
    }

    pub fn managed_schemas(&self) -> Option<&HashSet<SchemaId>> {
        self.managed_schemas.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub async fn schema_id(&self, ctx: &DalContext) -> ManagementPrototypeResult<Option<SchemaId>> {
        let snapshot = ctx.workspace_snapshot()?;

        let Some(sv_source_idx) = snapshot
            .incoming_sources_for_edge_weight_kind(
                self.id,
                EdgeWeightKindDiscriminants::ManagementPrototype,
            )
            .await?
            .first()
            .copied()
        else {
            return Ok(None);
        };

        let sv_id = snapshot.get_node_weight(sv_source_idx).await?.id();

        Ok(Some(
            SchemaVariant::schema_id_for_schema_variant_id(ctx, sv_id.into()).await?,
        ))
    }

    /// Generates a map between the schema name and its schema id (even for
    /// uninstalled schemas), and a reverse mapping. These names will be
    /// provided to the management executor and operator so that management
    /// functions can create components of specific schema kinds.
    pub async fn managed_schemas_map(
        &self,
        ctx: &DalContext,
    ) -> ManagementPrototypeResult<(HashMap<String, SchemaId>, HashMap<SchemaId, String>)> {
        let mut managed_schemas_map = HashMap::new();
        let mut reverse_map = HashMap::new();

        let mut managed_schemas = self.managed_schemas().cloned().unwrap_or_default();
        if let Some(schema_id) = self.schema_id(ctx).await? {
            managed_schemas.insert(schema_id);
        }

        for schema_id in managed_schemas {
            let schema_name = match Schema::get_by_id(ctx, schema_id).await? {
                Some(schema) => schema.name().to_owned(),
                None => {
                    let Some(cached_module) =
                        CachedModule::latest_by_schema_id(ctx, schema_id).await?
                    else {
                        continue;
                    };

                    cached_module.schema_name
                }
            };

            managed_schemas_map.insert(schema_name.clone(), schema_id);
            reverse_map.insert(schema_id, schema_name);
        }

        Ok((managed_schemas_map, reverse_map))
    }

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

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ManagementPrototypeContent::V1(content).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

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

    pub async fn modify<L>(self, ctx: &DalContext, lambda: L) -> ManagementPrototypeResult<Self>
    where
        L: FnOnce(&mut Self) -> ManagementPrototypeResult<()>,
    {
        let mut proto = self;
        let before: ManagementPrototypeContent = proto.clone().into();
        let proto_id = proto.id;
        lambda(&mut proto)?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        let updated: ManagementPrototypeContent = proto.into();

        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new((updated.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            workspace_snapshot
                .update_content(proto_id.into(), hash)
                .await?;
        }

        let ManagementPrototypeContent::V1(inner) = updated;

        Ok(Self {
            id: proto_id,
            managed_schemas: inner.managed_schemas,
            name: inner.name,
            description: inner.description,
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
    ) -> ManagementPrototypeResult<ManagementPrototypeExecution> {
        Self::execute_by_id(ctx, self.id, manager_component_id).await
    }

    pub async fn execute_by_id(
        ctx: &DalContext,
        id: ManagementPrototypeId,
        manager_component_id: ComponentId,
    ) -> ManagementPrototypeResult<ManagementPrototypeExecution> {
        let prototype = Self::get_by_id(ctx, id)
            .await?
            .ok_or(ManagementPrototypeError::NotFound(id))?;

        let (managed_schema_map, reverse_map) = prototype.managed_schemas_map(ctx).await?;

        let management_func_id = ManagementPrototype::func_id(ctx, id).await?;
        let manager_component = Component::get_by_id(ctx, manager_component_id).await?;
        let manager_component_view = manager_component.view(ctx).await?;
        let default_view_id = View::get_id_for_default(ctx).await?;
        let manager_geometry: NumericGeometry = manager_component
            .geometry(ctx, default_view_id)
            .await?
            .into_raw()
            .into();

        let managed_schema_names: Vec<String> = managed_schema_map.keys().cloned().collect();

        let mut managed_components = HashMap::new();
        for component_id in manager_component.get_managed(ctx).await? {
            let component = Component::get_by_id(ctx, component_id).await?;
            let component_view = component.view(ctx).await?;
            let component_geometry: NumericGeometry = component
                .geometry(ctx, default_view_id)
                .await?
                .into_raw()
                .into();
            let schema_id = component.schema(ctx).await?.id();

            if let Some(managed_schema_name) = reverse_map.get(&schema_id) {
                managed_components.insert(
                    component_id,
                    ManagedComponent {
                        kind: managed_schema_name.clone(),
                        properties: component_view,
                        geometry: component_geometry
                            .offset_by(-manager_geometry.x, -manager_geometry.y),
                    },
                );
            }
        }

        let placeholders: HashMap<_, _> = managed_components
            .keys()
            .copied()
            .map(|id| (id.to_string(), id))
            .collect();

        let args = serde_json::json!({
            "this_component": {
                "properties": manager_component_view,
                "geometry": NumericGeometry {
                    x: 0.0,
                    y: 0.0,
                    width: manager_geometry.width,
                    height: manager_geometry.height,
                }
            },
            "managed_schemas": managed_schema_names,
            "components": managed_components,
        });

        let result_channel =
            FuncRunner::run_management(ctx, id, manager_component_id, management_func_id, args)
                .await?;

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
                    )?
                    .0,
            ),
            None => None,
        };

        let maybe_run_result: Option<ManagementResultSuccess> = match run_value.value() {
            Some(value) => Some(serde_json::from_value(value.clone())?),
            None => None,
        };

        // Wart: We are hijacking the action result state to record the management result state.
        let action_state = maybe_run_result
            .as_ref()
            .map(|result| match result.health {
                veritech_client::ManagementFuncStatus::Ok => si_events::ActionResultState::Success,
                veritech_client::ManagementFuncStatus::Error => {
                    si_events::ActionResultState::Failure
                }
            })
            .unwrap_or(si_events::ActionResultState::Unknown);

        ctx.layer_db()
            .func_run()
            .set_management_result_success(
                func_run_id,
                action_state,
                None,
                maybe_value_address,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        // We publish this immediately because the management "operator" could
        // fail because of a bad function, but we stil want to know that the
        // function executed
        WsEvent::management_func_executed(ctx, id, manager_component_id, func_run_id)
            .await?
            .publish_immediately(ctx)
            .await?;

        Ok(ManagementPrototypeExecution {
            func_run_id,
            result: maybe_run_result,
            manager_component_geometry: manager_geometry,
            managed_schema_map,
            placeholders,
        })
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

    /// Is there at least one management prototype for this schema variant?
    pub async fn variant_has_management_prototype(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ManagementPrototypeResult<bool> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        Ok(!workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                schema_variant_id,
                EdgeWeightKindDiscriminants::ManagementPrototype,
            )
            .await?
            .is_empty())
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

    pub async fn prototype_id_for_func_id(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> ManagementPrototypeResult<Option<ManagementPrototypeId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let use_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(func_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        for use_source in use_sources {
            let node_weight = workspace_snapshot.get_node_weight(use_source).await?;
            let node_weight_id = node_weight.id();
            if NodeWeightDiscriminants::ManagementPrototype == node_weight.into() {
                if let Some(management_prototype) =
                    Self::get_by_id(ctx, node_weight_id.into()).await?
                {
                    return Ok(Some(management_prototype.id));
                }
            }
        }

        Ok(None)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagementFuncExecutedPayload {
    prototype_id: ManagementPrototypeId,
    manager_component_id: ComponentId,
    func_run_id: FuncRunId,
}

impl WsEvent {
    pub async fn management_func_executed(
        ctx: &DalContext,
        prototype_id: ManagementPrototypeId,
        manager_component_id: ComponentId,
        func_run_id: FuncRunId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            crate::WsPayload::ManagementFuncExecuted(ManagementFuncExecutedPayload {
                prototype_id,
                manager_component_id,
                func_run_id,
            }),
        )
        .await
    }
}
