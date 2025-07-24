//! A [`ManagementPrototype`] points to a Management [`Func`] for a schema variant

use std::{
    collections::HashMap,
    sync::Arc,
};

use serde::{
    Deserialize,
    Serialize,
};
use si_events::FuncRunId;
use si_id::OutputSocketId;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::{
    ManagementFuncStatus,
    ManagementResultSuccess,
};

use super::{
    ManagementError,
    ManagementGeometry,
    SocketRef,
};
use crate::{
    AttributeValue,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    FuncError,
    FuncId,
    HelperError,
    InputSocket,
    NodeWeightDiscriminants,
    OutputSocket,
    Schema,
    SchemaError,
    SchemaId,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    SocketArity,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    WsEventResult,
    WsPayload,
    attribute::attributes::{
        AttributeValueIdent,
        Source,
    },
    cached_module::{
        CachedModule,
        CachedModuleError,
    },
    component::socket::ComponentInputSocket,
    diagram::{
        DiagramError,
        geometry::Geometry,
        view::{
            View,
            ViewId,
        },
    },
    func::runner::{
        FuncRunner,
        FuncRunnerError,
        FuncRunnerValueChannel,
    },
    implement_add_edge_to,
    layer_db_types::{
        ManagementPrototypeContent,
        ManagementPrototypeContentV1,
    },
    workspace_snapshot::node_weight::{
        NodeWeight,
        traits::SiVersionedNodeWeight,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ManagementPrototypeError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<crate::attribute::prototype::AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(
        #[from] Box<crate::attribute::prototype::argument::AttributePrototypeArgumentError>,
    ),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<crate::attribute::value::AttributeValueError>),
    #[error("cached module error: {0}")]
    CachedModule(#[from] Box<CachedModuleError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("diagram error: {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("empty value within func run value (FuncId {0} and FuncRunId {1})")]
    EmptyValueWithinFuncRunValue(FuncId, FuncRunId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func execution failure error: {0}")]
    FuncExecutionFailure(String),
    #[error("func runner error: {0}")]
    FuncRunner(#[from] Box<FuncRunnerError>),
    #[error("func runner recv error")]
    FuncRunnerRecvError,
    #[error("helper error: {0}")]
    Helper(#[from] Box<HelperError>),
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<crate::socket::input::InputSocketError>),
    #[error("invalid prototype for component")]
    InvalidPrototypeForComponent(ManagementPrototypeId, ComponentId),
    #[error("layer db error: {0}")]
    LayerDbError(#[from] si_layer_cache::LayerDbError),
    #[error("management error: {0}")]
    Management(#[from] Box<ManagementError>),
    #[error("management func execution state error: {0}")]
    ManagementExecution(#[from] si_db::ManagementFuncExecutionError),
    #[error("management prototype {0} has no use edge to a function")]
    MissingFunction(ManagementPrototypeId),
    #[error("More than one connection to input socket with arity one: {0}")]
    MoreThanOneInputConnection(String),
    #[error("No pending management func execution found")]
    NoPendingManagementFuncExecution,
    #[error("management prototype {0} not found")]
    NotFound(ManagementPrototypeId),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] Box<crate::socket::output::OutputSocketError>),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("too few variants: {0}")]
    TooFewVariants(ManagementPrototypeId),
    #[error("too many variants: {0}")]
    TooManyVariants(ManagementPrototypeId),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

impl From<crate::attribute::value::AttributeValueError> for ManagementPrototypeError {
    fn from(value: crate::attribute::value::AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<CachedModuleError> for ManagementPrototypeError {
    fn from(value: CachedModuleError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for ManagementPrototypeError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DiagramError> for ManagementPrototypeError {
    fn from(value: DiagramError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncRunnerError> for ManagementPrototypeError {
    fn from(value: FuncRunnerError) -> Self {
        Box::new(value).into()
    }
}

impl From<HelperError> for ManagementPrototypeError {
    fn from(value: HelperError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::socket::input::InputSocketError> for ManagementPrototypeError {
    fn from(value: crate::socket::input::InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<ManagementError> for ManagementPrototypeError {
    fn from(value: ManagementError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::socket::output::OutputSocketError> for ManagementPrototypeError {
    fn from(value: crate::socket::output::OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaError> for ManagementPrototypeError {
    fn from(value: SchemaError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for ManagementPrototypeError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for ManagementPrototypeError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for ManagementPrototypeError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for ManagementPrototypeError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

pub type ManagementPrototypeResult<T> = Result<T, ManagementPrototypeError>;

pub use si_id::ManagementPrototypeId;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManagementPrototype {
    pub id: ManagementPrototypeId,
    pub name: String,
    pub description: Option<String>,
}

impl From<ManagementPrototype> for ManagementPrototypeContent {
    fn from(value: ManagementPrototype) -> Self {
        Self::V1(ManagementPrototypeContentV1 {
            name: value.name,
            description: value.description,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ManagementPrototypeExecution {
    pub func_run_id: FuncRunId,
    pub result: Option<ManagementResultSuccess>,
    pub manager_component_geometry: HashMap<String, ManagementGeometry>,
    pub managed_component_placeholders: HashMap<String, ComponentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedComponent {
    kind: String,
    properties: Option<serde_json::Value>,
    sources: HashMap<AttributeValueIdent, Source>,
    geometry: HashMap<String, ManagementGeometry>,
    incoming_connections: HashMap<String, SocketRefsAndValues>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SocketRefsAndValues {
    // If arity is one, we never serialize to a JS array
    One(Option<SocketRefAndValue>),
    // If arity is many, we *always* serialize to a JS array
    Many(Vec<SocketRefAndValue>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketRefAndValue {
    #[serde(flatten)]
    socket_ref: SocketRef,
    pub value: Option<serde_json::Value>,
}

async fn build_management_geometry_map(
    ctx: &DalContext,
    component_id: ComponentId,
    views: &HashMap<ViewId, String>,
) -> ManagementPrototypeResult<HashMap<String, ManagementGeometry>> {
    let mut geometry_map = HashMap::new();
    for (&view_id, view_name) in views {
        let Some(geometry) =
            Geometry::try_get_by_component_and_view(ctx, component_id, view_id).await?
        else {
            continue;
        };

        geometry_map.insert(view_name.clone(), geometry.into_raw().into());
    }

    Ok(geometry_map)
}

async fn build_incoming_connections(
    ctx: &DalContext,
    component_id: ComponentId,
) -> ManagementPrototypeResult<HashMap<String, SocketRefsAndValues>> {
    let mut incoming_connections = HashMap::new();
    for input_socket in ComponentInputSocket::list_for_component_id(ctx, component_id).await? {
        // Collect explicit connections for this input socket
        let mut socket_connections = Vec::new();
        for (from_component_id, from_socket_id, _) in input_socket.connections(ctx).await? {
            socket_connections
                .push(build_connection(ctx, from_component_id, from_socket_id).await?);
        }

        // Collect inferred connections for this input socket
        for from in input_socket.find_inferred_connections(ctx).await? {
            socket_connections
                .push(build_connection(ctx, from.component_id, from.output_socket_id).await?);
        }

        // Create an array or single connection value depending on arity
        let input_socket = InputSocket::get_by_id(ctx, input_socket.input_socket_id).await?;
        let name = input_socket.name().to_owned();
        let socket_connections = match input_socket.arity() {
            SocketArity::One if socket_connections.len() > 1 => {
                return Err(ManagementPrototypeError::MoreThanOneInputConnection(name));
            }
            SocketArity::One => SocketRefsAndValues::One(socket_connections.pop()),
            SocketArity::Many => SocketRefsAndValues::Many(socket_connections),
        };

        // If there are multiple sockets with the same name, this will only keep the last one.
        // This situation is supposed to be prevented elsewhere.
        incoming_connections.insert(name, socket_connections);
    }
    Ok(incoming_connections)
}

async fn build_connection(
    ctx: &DalContext,
    from_component_id: ComponentId,
    from_socket_id: OutputSocketId,
) -> ManagementPrototypeResult<SocketRefAndValue> {
    let component = from_component_id.to_string();
    let socket = OutputSocket::get_by_id(ctx, from_socket_id)
        .await?
        .name()
        .to_owned();
    let av_id =
        OutputSocket::component_attribute_value_id(ctx, from_socket_id, from_component_id).await?;
    let value = AttributeValue::view(ctx, av_id).await?;
    Ok(SocketRefAndValue {
        socket_ref: SocketRef { component, socket },
        value,
    })
}

impl ManagedComponent {
    pub async fn new(
        ctx: &DalContext,
        component_id: ComponentId,
        kind: &str,
        views: &HashMap<ViewId, String>,
    ) -> ManagementPrototypeResult<Self> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let properties = component.view(ctx).await?;
        let sources = Component::subscription_sources(ctx, component_id).await?;
        let geometry = build_management_geometry_map(ctx, component_id, views).await?;
        let incoming_connections = build_incoming_connections(ctx, component_id).await?;

        Ok(Self {
            kind: kind.to_owned(),
            properties,
            sources: sources.into_iter().collect(),
            geometry,
            incoming_connections,
        })
    }
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

        Ok(Some(SchemaVariant::schema_id(ctx, sv_id.into()).await?))
    }

    pub async fn new(
        ctx: &DalContext,
        name: String,
        description: Option<String>,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> ManagementPrototypeResult<Self> {
        let content = ManagementPrototypeContentV1 {
            name: name.clone(),
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
            name: inner.name,
            description: inner.description,
        })
    }

    pub async fn get_by_id_opt(
        ctx: &DalContext,
        id: ManagementPrototypeId,
    ) -> ManagementPrototypeResult<Option<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let Some(NodeWeight::ManagementPrototype(inner)) =
            workspace_snapshot.get_node_weight_opt(id).await
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
            name: content_inner.name,
            description: content_inner.description,
        }))
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        id: ManagementPrototypeId,
    ) -> ManagementPrototypeResult<Self> {
        Self::get_by_id_opt(ctx, id)
            .await?
            .ok_or(ManagementPrototypeError::NotFound(id))
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

    pub async fn start_execution(
        ctx: &DalContext,
        id: ManagementPrototypeId,
        manager_component_id: ComponentId,
        view_id: Option<ViewId>,
    ) -> ManagementPrototypeResult<(
        HashMap<String, ManagementGeometry>,
        HashMap<String, ComponentId>,
        FuncRunnerValueChannel,
        FuncRunId,
    )> {
        let views: HashMap<ViewId, String> = View::list(ctx)
            .await?
            .into_iter()
            .map(|view| (view.id(), view.name().to_owned()))
            .collect();

        let view_id = match view_id {
            Some(view_id) => view_id,
            None => View::get_id_for_default(ctx).await?,
        };
        let current_view = views.get(&view_id).cloned().unwrap_or(view_id.to_string());

        let management_func_id = ManagementPrototype::func_id(ctx, id).await?;
        let manager_component = Component::get_by_id(ctx, manager_component_id).await?;

        let mut managed_component_placeholders = HashMap::new();
        let mut managed_components = HashMap::new();
        for component_id in manager_component.get_managed(ctx).await? {
            let schema = Component::schema_for_component_id(ctx, component_id).await?;
            let managed_component =
                ManagedComponent::new(ctx, component_id, schema.name(), &views).await?;

            managed_components.insert(component_id, managed_component);
            let name = Component::name_by_id(ctx, component_id).await?;
            managed_component_placeholders.insert(name, component_id);
            managed_component_placeholders.insert(component_id.to_string(), component_id);
        }

        let this_schema = Component::schema_for_component_id(ctx, manager_component_id)
            .await?
            .name()
            .to_string();

        let manager_component =
            ManagedComponent::new(ctx, manager_component_id, &this_schema, &views).await?;
        let manager_component_geometry = manager_component.geometry.to_owned();

        let variant_socket_map = Self::variant_socket_map(ctx).await?;

        let args = serde_json::json!({
            "current_view": current_view,
            "this_component": manager_component,
            "components": managed_components,
            "variant_socket_map": variant_socket_map,
        });

        let (func_run_id, result_channel) =
            FuncRunner::run_management(ctx, id, manager_component_id, management_func_id, args)
                .await?;

        Ok((
            manager_component_geometry,
            managed_component_placeholders,
            result_channel,
            func_run_id,
        ))
    }

    pub async fn finalize_execution(
        ctx: &DalContext,
        manager_component_id: ComponentId,
        id: ManagementPrototypeId,
        manager_component_geometry: HashMap<String, ManagementGeometry>,
        managed_component_placeholders: HashMap<String, ComponentId>,
        func_run_channel: FuncRunnerValueChannel,
    ) -> ManagementPrototypeResult<ManagementPrototypeExecution> {
        let run_value = match func_run_channel.await {
            Ok(Err(FuncRunnerError::ResultFailure {
                kind: _,
                message,
                backend: _,
            })) => return Err(ManagementPrototypeError::FuncExecutionFailure(message)),
            other => other.map_err(|_| ManagementPrototypeError::FuncRunnerRecvError)??,
        };

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

        FuncRunner::update_run(ctx, func_run_id, |func_run| {
            func_run.set_success(None, maybe_value_address);
            func_run.set_action_result_state(Some(action_state));
        })
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
            manager_component_geometry,
            managed_component_placeholders,
        })
    }

    async fn variant_socket_map(
        ctx: &DalContext,
    ) -> ManagementPrototypeResult<HashMap<String, u32>> {
        // Count sockets on uninstalled schemas
        let mut variant_socket_map: HashMap<_, _> = CachedModule::latest_modules(ctx)
            .await?
            .into_iter()
            .filter_map(|module| {
                module
                    .package_summary
                    .as_ref()
                    .map(|summary| (module.schema_name.to_owned(), summary.socket_count))
            })
            .collect();

        // Count sockets on installed schemas (and overwrite the counts from uninstalled ones)
        for schema in Schema::list(ctx).await? {
            let variant_id = Schema::default_variant_id_opt(ctx, schema.id()).await?;
            if let Some(variant_id) = variant_id {
                let (output_sockets, input_sockets) =
                    SchemaVariant::list_all_socket_ids(ctx, variant_id).await?;

                let has_manage_prototype =
                    ManagementPrototype::variant_has_management_prototype(ctx, variant_id).await?;
                let manage_count: u32 = if has_manage_prototype { 1 } else { 0 };

                // 1 + for the Manager input socket
                variant_socket_map.insert(
                    schema.name().to_string(),
                    1 + manage_count + output_sockets.len() as u32 + input_sockets.len() as u32,
                );
            }
        }

        Ok(variant_socket_map)
    }

    pub async fn execute(
        &self,
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: Option<ViewId>,
    ) -> ManagementPrototypeResult<ManagementPrototypeExecution> {
        Self::execute_by_id(ctx, self.id(), component_id, view_id).await
    }

    /// Execute the management function defined by the `prototype_id` for the given component
    pub async fn execute_by_id(
        ctx: &DalContext,
        prototype_id: ManagementPrototypeId,
        component_id: ComponentId,
        view_id: Option<ViewId>,
    ) -> ManagementPrototypeResult<ManagementPrototypeExecution> {
        if !ManagementPrototype::is_valid_prototype_for_component(ctx, prototype_id, component_id)
            .await?
        {
            return Err(ManagementPrototypeError::InvalidPrototypeForComponent(
                prototype_id,
                component_id,
            ));
        }

        let (geometries, placeholders, run_channel, _) =
            ManagementPrototype::start_execution(ctx, prototype_id, component_id, view_id).await?;

        ManagementPrototype::finalize_execution(
            ctx,
            component_id,
            prototype_id,
            geometries,
            placeholders,
            run_channel,
        )
        .await
    }

    pub async fn is_valid_prototype_for_component(
        ctx: &DalContext,
        management_prototype_id: ManagementPrototypeId,
        component_id: ComponentId,
    ) -> ManagementPrototypeResult<bool> {
        let prototoype_schema_variant_id =
            Self::get_schema_variant_id(ctx, management_prototype_id).await?;
        let component_schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
        Ok(prototoype_schema_variant_id == component_schema_variant_id)
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
                    Self::get_by_id_opt(ctx, node_weight_id.into()).await?
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
                    Self::get_by_id_opt(ctx, node_weight_id.into()).await?
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
                    Self::get_by_id_opt(ctx, node_weight_id.into()).await?
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

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperationsCompletePayload {
    request_ulid: Option<ulid::Ulid>,
    func_name: String,
    status: ManagementFuncStatus,
    message: Option<String>,
    created_component_ids: Option<Vec<ComponentId>>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperationsFailedPayload {
    request_ulid: ulid::Ulid,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperationsInProgressPayload {
    request_ulid: ulid::Ulid,
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
            WsPayload::ManagementFuncExecuted(ManagementFuncExecutedPayload {
                prototype_id,
                manager_component_id,
                func_run_id,
            }),
        )
        .await
    }

    pub async fn management_operations_complete(
        ctx: &DalContext,
        request_ulid: Option<ulid::Ulid>,
        func_name: String,
        message: Option<String>,
        status: ManagementFuncStatus,
        created_component_ids: Option<Vec<ComponentId>>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ManagementOperationsComplete(ManagementOperationsCompletePayload {
                request_ulid,
                func_name,
                status,
                message,
                created_component_ids,
            }),
        )
        .await
    }

    pub async fn management_operations_failed(
        ctx: &DalContext,
        request_ulid: ulid::Ulid,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ManagementOperationsFailed(ManagementOperationsFailedPayload {
                request_ulid,
            }),
        )
        .await
    }

    pub async fn management_operations_in_progress(
        ctx: &DalContext,
        request_ulid: ulid::Ulid,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ManagementOperationsInProgress(ManagementOperationsInProgressPayload {
                request_ulid,
            }),
        )
        .await
    }
}
