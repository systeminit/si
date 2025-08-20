use std::collections::{
    HashMap,
    HashSet,
    VecDeque,
    hash_map,
};

use prototype::ManagementPrototypeExecution;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    ActorView,
    HistoryEventMetadata,
};
use si_events::audit_log::AuditLogKind;
use si_id::AttributeValueId;
use telemetry::prelude::*;
use thiserror::Error;
use veritech_client::{
    ManagementFuncStatus,
    ManagementResultSuccess,
};

use crate::{
    AttributeValue,
    Component,
    ComponentError,
    ComponentId,
    ComponentType,
    DalContext,
    EdgeWeightKind,
    Func,
    FuncError,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropKind,
    Schema,
    SchemaError,
    SchemaId,
    SchemaVariantId,
    TransactionsError,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventError,
    action::{
        Action,
        ActionError,
        prototype::{
            ActionKind,
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    attribute::{
        attributes::AttributeSources,
        prototype::argument::{
            AttributePrototypeArgument,
            AttributePrototypeArgumentError,
        },
        value::AttributeValueError,
    },
    change_status::ChangeStatus::Added,
    component::{
        Connection,
        ControllingFuncData,
        delete::{
            ComponentDeletionStatus,
            delete_components,
        },
        frame::{
            Frame,
            FrameError,
            InferredEdgeChanges,
        },
        resource::ResourceData,
    },
    dependency_graph::DependencyGraph,
    diagram::{
        DiagramError,
        SummaryDiagramEdge,
        SummaryDiagramInferredEdge,
        SummaryDiagramManagementEdge,
        geometry::{
            Geometry,
            RawGeometry,
        },
        view::{
            View,
            ViewComponentsUpdateSingle,
            ViewId,
            ViewView,
        },
    },
    prop::{
        PropError,
        PropPath,
    },
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
};

pub mod prototype;

#[derive(Debug, Error)]
pub enum ManagementError {
    #[error("action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("attributes error: {0}")]
    Attributes(#[from] Box<crate::attribute::attributes::AttributesError>),
    #[error("cannot create component with 'self' as a placeholder")]
    CannotCreateComponentWithSelfPlaceholder,
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error(
        "cannot add an action of kind {0} because component {1} does not have an action of that kind"
    )]
    ComponentDoesNotHaveAction(ActionKind, ComponentId),
    #[error(
        "cannot add a manual action named {0} because component {1} does not have a manual action with that name"
    )]
    ComponentDoesNotHaveManualAction(String, ComponentId),
    #[error("Component somehow not created! This is a bug.")]
    ComponentNotCreated,
    #[error("Component with management placeholder {0} could not be found")]
    ComponentWithPlaceholderNotFound(String),
    #[error("Diagram Error {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("Duplicate component placeholder {0}")]
    DuplicateComponentPlaceholder(String),
    #[error("frame error: {0}")]
    Frame(#[from] Box<FrameError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("Component {0} does not have an input socket with name {1}")]
    InputSocketDoesNotExist(ComponentId, String),
    #[error("No existing or created view could be found named: {0}")]
    NoSuchView(String),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] Box<OutputSocketError>),
    #[error("Component {0} does not have an output socket with name {1}")]
    OutputSocketDoesNotExist(ComponentId, String),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecodeError(#[from] ulid::DecodeError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] Box<WsEventError>),
}

pub type ManagementResult<T> = Result<T, ManagementError>;

const DEFAULT_FRAME_WIDTH: f64 = 950.0;
const DEFAULT_FRAME_HEIGHT: f64 = 750.0;

/// Geometry type for deserialization lang-js, so even if we should only care about integers,
/// until we implement custom deserialization we can't merge it with [RawGeometry](RawGeometry)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ManagementGeometry {
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl ManagementGeometry {
    pub fn offset_by(&self, x_off: Option<f64>, y_off: Option<f64>) -> Self {
        let mut x_off = x_off.unwrap_or(0.0);
        let mut y_off = y_off.unwrap_or(0.0);

        if !x_off.is_normal() {
            x_off = 0.0;
        }
        if !y_off.is_normal() {
            y_off = 0.0;
        }

        let self_x = self.x.unwrap_or(0.0);
        let self_y = self.y.unwrap_or(0.0);

        let x = if self_x.is_normal() {
            self_x + x_off
        } else {
            x_off
        };

        let y = if self_y.is_normal() {
            self_y + y_off
        } else {
            y_off
        };

        Self {
            x: Some(x),
            y: Some(y),
            width: self.width,
            height: self.height,
        }
    }
}

#[inline(always)]
#[allow(unused)]
fn avoid_nan_string(n: f64, fallback: f64) -> String {
    if n.is_normal() { n.round() } else { fallback }.to_string()
}

impl From<ManagementGeometry> for RawGeometry {
    fn from(value: ManagementGeometry) -> Self {
        Self {
            x: value.x.unwrap_or(0.0) as isize,
            y: value.y.unwrap_or(0.0) as isize,
            width: value.width.map(|w| w as isize),
            height: value.height.map(|h| h as isize),
        }
    }
}

impl From<RawGeometry> for ManagementGeometry {
    fn from(value: RawGeometry) -> Self {
        Self {
            x: Some(value.x as f64),
            y: Some(value.y as f64),
            width: value.width.map(|w| w as f64),
            height: value.height.map(|h| h as f64),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketRef {
    pub component: String,
    pub socket: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum ManagementConnection {
    Input { from: SocketRef, to: String },
    Output { from: String, to: SocketRef },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateConnections {
    add: Option<Vec<ManagementConnection>>,
    remove: Option<Vec<ManagementConnection>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateOperation {
    properties: Option<serde_json::Value>,
    attributes: Option<AttributeSources>,
    geometry: Option<HashMap<String, ManagementGeometry>>,
    connect: Option<ManagementUpdateConnections>,
    parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ManagementCreateGeometry {
    #[serde(rename_all = "camelCase")]
    WithViews(HashMap<String, ManagementGeometry>),
    #[serde(rename_all = "camelCase")]
    CurrentView(ManagementGeometry),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementCreateOperation {
    kind: Option<String>,
    properties: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<AttributeSources>,
    geometry: Option<ManagementCreateGeometry>,
    connect: Option<Vec<ManagementConnection>>,
    parent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ActionIdentifier {
    pub kind: ActionKind,
    pub manual_func_name: Option<String>,
}

impl From<&str> for ActionIdentifier {
    fn from(action_str: &str) -> Self {
        match action_str.to_lowercase().as_str() {
            "create" => ActionIdentifier {
                kind: ActionKind::Create,
                manual_func_name: None,
            },
            "destroy" => ActionIdentifier {
                kind: ActionKind::Destroy,
                manual_func_name: None,
            },
            "refresh" => ActionIdentifier {
                kind: ActionKind::Refresh,
                manual_func_name: None,
            },
            "update" => ActionIdentifier {
                kind: ActionKind::Update,
                manual_func_name: None,
            },
            _ => ActionIdentifier {
                kind: ActionKind::Manual,
                manual_func_name: Some(action_str.to_string()),
            },
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManagementActionOperation {
    add: Option<Vec<String>>,
    remove: Option<Vec<String>>,
}

pub type ManagementCreateOperations = HashMap<String, ManagementCreateOperation>;
pub type ManagementUpdateOperations = HashMap<String, ManagementUpdateOperation>;
pub type ManagementActionOperations = HashMap<String, ManagementActionOperation>;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementViewOperations {
    create: Option<Vec<String>>,
    remove: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperations {
    create: Option<ManagementCreateOperations>,
    update: Option<ManagementUpdateOperations>,
    // marks as to delete, enqueues a delete action
    delete: Option<Vec<String>>,
    // delete the component even if it has a resource, do not automatically
    // enqueue an action
    erase: Option<Vec<String>>,
    // remove components from views. Keyed by view, then array of component
    // placeholders
    remove: Option<HashMap<String, Vec<String>>>,
    actions: Option<ManagementActionOperations>,
    views: Option<ManagementViewOperations>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementFuncReturn {
    pub status: ManagementFuncStatus,
    pub operations: Option<ManagementOperations>,
    pub message: Option<String>,
    pub error: Option<String>,
}

impl TryFrom<ManagementResultSuccess> for ManagementFuncReturn {
    type Error = serde_json::Error;

    fn try_from(value: ManagementResultSuccess) -> Result<Self, Self::Error> {
        Ok(ManagementFuncReturn {
            status: value.health,
            operations: match value.operations {
                Some(ops) => serde_json::from_value(ops)?,
                None => None,
            },
            message: value.message,
            error: value.error,
        })
    }
}

const SELF_ID: &str = "self";

struct ComponentSchemaMap {
    variants: HashMap<ComponentId, SchemaVariantId>,
    schemas: HashMap<ComponentId, SchemaId>,
}

impl ComponentSchemaMap {
    pub fn new() -> Self {
        Self {
            variants: HashMap::new(),
            schemas: HashMap::new(),
        }
    }

    pub async fn schema_for_component_id(
        &mut self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ManagementResult<SchemaId> {
        Ok(match self.schemas.entry(component_id) {
            hash_map::Entry::Occupied(occupied_entry) => *occupied_entry.get(),
            hash_map::Entry::Vacant(vacant_entry) => {
                let schema_id = Component::schema_for_component_id(ctx, component_id)
                    .await?
                    .id();
                *vacant_entry.insert(schema_id)
            }
        })
    }

    pub async fn variant_for_component_id(
        &mut self,
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ManagementResult<SchemaVariantId> {
        Ok(match self.variants.entry(component_id) {
            hash_map::Entry::Occupied(occupied_entry) => *occupied_entry.get(),
            hash_map::Entry::Vacant(vacant_entry) => {
                let variant_id = Component::schema_variant_id(ctx, component_id).await?;
                *vacant_entry.insert(variant_id)
            }
        })
    }
}

struct VariantSocketMap {
    input_sockets: HashMap<SchemaVariantId, HashMap<String, InputSocketId>>,
    output_sockets: HashMap<SchemaVariantId, HashMap<String, OutputSocketId>>,
}

impl VariantSocketMap {
    pub fn new() -> Self {
        Self {
            input_sockets: HashMap::new(),
            output_sockets: HashMap::new(),
        }
    }

    fn add_input_socket_for_id(
        &mut self,
        variant_id: SchemaVariantId,
        name: &str,
        socket_id: InputSocketId,
    ) {
        self.input_sockets
            .entry(variant_id)
            .or_default()
            .insert(name.to_owned(), socket_id);
    }

    fn add_output_socket_for_id(
        &mut self,
        variant_id: SchemaVariantId,
        name: &str,
        socket_id: OutputSocketId,
    ) {
        self.output_sockets
            .entry(variant_id)
            .or_default()
            .insert(name.to_owned(), socket_id);
    }

    pub async fn add_sockets_for_variant(
        &mut self,
        ctx: &DalContext,
        variant_id: SchemaVariantId,
    ) -> ManagementResult<()> {
        if self.input_sockets.contains_key(&variant_id) {
            return Ok(());
        }

        for socket in InputSocket::list(ctx, variant_id).await? {
            self.add_input_socket_for_id(variant_id, socket.name(), socket.id());
        }

        for socket in OutputSocket::list(ctx, variant_id).await? {
            self.add_output_socket_for_id(variant_id, socket.name(), socket.id());
        }

        Ok(())
    }

    pub fn output_socket_id(
        &self,
        variant_id: SchemaVariantId,
        name: &str,
    ) -> Option<OutputSocketId> {
        self.output_sockets
            .get(&variant_id)
            .and_then(|sockets| sockets.get(name))
            .copied()
    }

    pub fn input_socket_id(
        &self,
        variant_id: SchemaVariantId,
        name: &str,
    ) -> Option<InputSocketId> {
        self.input_sockets
            .get(&variant_id)
            .and_then(|sockets| sockets.get(name))
            .copied()
    }
}

pub struct ManagementOperator<'a> {
    ctx: &'a DalContext,
    manager_component_id: ComponentId,
    manager_component_geometry: HashMap<ViewId, ManagementGeometry>,
    manager_schema_id: SchemaId,
    last_component_geometry: HashMap<ViewId, ManagementGeometry>,
    operations: ManagementOperations,
    managed_component_id_placeholders: HashMap<String, ComponentId>,
    component_id_placeholders: HashMap<String, ComponentId>,
    component_schema_map: ComponentSchemaMap,
    socket_map: VariantSocketMap,
    current_view_id: ViewId,
    view_placeholders: HashMap<String, ViewId>,
    created_components: HashMap<ComponentId, Vec<ViewId>>,
    updated_components: HashMap<ComponentId, Vec<ViewId>>,
    request_ulid: ulid::Ulid,
}

#[derive(Clone, Debug)]
struct PendingConnect {
    component_id: ComponentId,
    connection: ManagementConnection,
}

#[derive(Clone, Debug)]
struct PendingParent {
    child_component_id: ComponentId,
    parent: String,
}

#[derive(Clone, Debug)]
struct PendingManage {
    managed_component_id: ComponentId,
    managed_component_schema_id: SchemaId,
}

#[derive(Clone, Debug)]
enum PendingOperation {
    Connect(PendingConnect),
    InferredEdgeRemoveEvent(Vec<SummaryDiagramInferredEdge>),
    InferredEdgeUpsertEvent(Vec<SummaryDiagramInferredEdge>),
    Manage(PendingManage),
    Parent(PendingParent),
    RemoveConnection(PendingConnect),
}

struct CreatedComponent {
    component: Component,
    geometry: HashMap<ViewId, ManagementGeometry>,
    schema_id: SchemaId,
}

impl<'a> ManagementOperator<'a> {
    pub async fn new(
        ctx: &'a DalContext,
        manager_component_id: ComponentId,
        operations: ManagementOperations,
        management_execution: ManagementPrototypeExecution,
        view_id: Option<ViewId>,
        request_ulid: ulid::Ulid,
    ) -> ManagementResult<Self> {
        let mut component_id_placeholders = HashMap::new();
        component_id_placeholders.insert(SELF_ID.to_string(), manager_component_id);

        let mut component_schema_map = ComponentSchemaMap::new();
        let manager_schema_id = component_schema_map
            .schema_for_component_id(ctx, manager_component_id)
            .await?;
        component_schema_map
            .variant_for_component_id(ctx, manager_component_id)
            .await?;

        let current_view_id = match view_id {
            Some(view_id) => view_id,
            None => View::get_id_for_default(ctx).await?,
        };

        let mut manager_component_geometry_in_view: HashMap<ViewId, ManagementGeometry> =
            HashMap::new();

        let mut views = HashMap::new();
        for view in View::list(ctx).await? {
            views.insert(view.name().to_owned(), view.id());
            if let Some(geo) =
                Geometry::try_get_by_component_and_view(ctx, manager_component_id, view.id())
                    .await?
                    .map(|geo| geo.into_raw().into())
            {
                manager_component_geometry_in_view.insert(view.id(), geo);
            }
        }

        Ok(Self {
            ctx,
            manager_component_id,
            manager_schema_id,
            last_component_geometry: manager_component_geometry_in_view.clone(),
            manager_component_geometry: manager_component_geometry_in_view,
            operations,
            managed_component_id_placeholders: management_execution.managed_component_placeholders,
            component_id_placeholders,
            component_schema_map,
            socket_map: VariantSocketMap::new(),
            current_view_id,
            view_placeholders: views,
            created_components: HashMap::new(),
            updated_components: HashMap::new(),
            request_ulid,
        })
    }

    fn get_auto_geometry_for_view(&self, view_id: ViewId) -> ManagementGeometry {
        let mut geo = self
            .last_component_geometry
            .get(&view_id)
            .cloned()
            .unwrap_or(ManagementGeometry {
                x: Some(0.0),
                y: Some(0.0),
                width: None,
                height: None,
            })
            .offset_by(Some(75.0), Some(75.0));

        geo.height.take();
        geo.width.take();

        geo
    }

    async fn create_component(
        &self,
        placeholder: &str,
        operation: &ManagementCreateOperation,
    ) -> ManagementResult<CreatedComponent> {
        let schema_id = match &operation.kind {
            Some(kind) => Schema::get_or_install_by_name(self.ctx, kind).await?.id(),
            None => self.manager_schema_id,
        };
        let variant_id = Schema::default_variant_id(self.ctx, schema_id).await?;
        let mut created_geometries = HashMap::new();

        let component = match &operation.geometry {
            Some(ManagementCreateGeometry::WithViews(geometries)) => {
                let mut component: Option<Component> = None;
                let mut will_be_frame = None;

                for (view_placeholder, geometry) in geometries {
                    let geometry_view_id = self
                        .view_placeholders
                        .get(view_placeholder)
                        .copied()
                        .ok_or(ManagementError::NoSuchView(view_placeholder.to_owned()))?;

                    let mut comp = match component.as_ref() {
                        Some(component) => component.to_owned(),
                        None => {
                            let comp =
                                Component::new(self.ctx, placeholder, variant_id, geometry_view_id)
                                    .await?;
                            will_be_frame = Some(
                                component_will_be_frame(
                                    self.ctx,
                                    &comp,
                                    operation.properties.as_ref(),
                                )
                                .await?,
                            );

                            comp
                        }
                    };

                    let auto_geometry = self.get_auto_geometry_for_view(geometry_view_id);

                    // If the manager component exists in this view, then use
                    // that as the origin. Otherwise, use the position of the
                    // manager component in the view the function was executed
                    // in as the origin for relative geometry
                    let origin_geometry = self
                        .manager_component_geometry
                        .get(&geometry_view_id)
                        .or_else(|| self.manager_component_geometry.get(&self.current_view_id))
                        .copied();

                    let geometry = process_geometry(
                        geometry,
                        auto_geometry.x,
                        auto_geometry.y,
                        origin_geometry.and_then(|geo| geo.x),
                        origin_geometry.and_then(|geo| geo.y),
                        will_be_frame.unwrap_or(false),
                    );

                    created_geometries.insert(geometry_view_id, geometry);

                    match component.as_ref().map(|c| c.id()) {
                        Some(component_id) => {
                            Component::add_to_view(
                                self.ctx,
                                component_id,
                                geometry_view_id,
                                geometry.into(),
                            )
                            .await?;
                        }
                        None => {
                            comp.set_raw_geometry(self.ctx, geometry.into(), geometry_view_id)
                                .await?;
                            component = Some(comp);
                        }
                    }
                }

                component
            }
            Some(ManagementCreateGeometry::CurrentView(geometry)) => {
                let mut component =
                    Component::new(self.ctx, placeholder, variant_id, self.current_view_id).await?;
                let will_be_frame =
                    component_will_be_frame(self.ctx, &component, operation.properties.as_ref())
                        .await?;

                let auto_geometry = self.get_auto_geometry_for_view(self.current_view_id);
                let geometry = process_geometry(
                    geometry,
                    auto_geometry.x,
                    auto_geometry.y,
                    self.manager_component_geometry
                        .get(&self.current_view_id)
                        .and_then(|geo| geo.x),
                    self.manager_component_geometry
                        .get(&self.current_view_id)
                        .and_then(|geo| geo.y),
                    will_be_frame,
                );

                created_geometries.insert(self.current_view_id, geometry);

                component
                    .set_raw_geometry(self.ctx, geometry.into(), self.current_view_id)
                    .await?;

                Some(component)
            }
            None => {
                let mut component =
                    Component::new(self.ctx, placeholder, variant_id, self.current_view_id).await?;
                let will_be_frame =
                    component_will_be_frame(self.ctx, &component, operation.properties.as_ref())
                        .await?;

                let auto_geometry = self.get_auto_geometry_for_view(self.current_view_id);

                let geometry =
                    process_geometry(&auto_geometry, None, None, None, None, will_be_frame);

                created_geometries.insert(self.current_view_id, geometry);

                component
                    .set_raw_geometry(self.ctx, geometry.into(), self.current_view_id)
                    .await?;

                Some(component)
            }
        };

        let component = component.ok_or(ManagementError::ComponentNotCreated)?;

        Ok(CreatedComponent {
            component,
            geometry: created_geometries,
            schema_id,
        })
    }

    async fn input_socket_id(
        &mut self,
        component_id: ComponentId,
        socket: &str,
    ) -> ManagementResult<InputSocketId> {
        let variant_id = self
            .component_schema_map
            .variant_for_component_id(self.ctx, component_id)
            .await?;
        // if the map was already constructed this does nothing
        self.socket_map
            .add_sockets_for_variant(self.ctx, variant_id)
            .await?;
        let socket_id = self.socket_map.input_socket_id(variant_id, socket).ok_or(
            ManagementError::InputSocketDoesNotExist(component_id, socket.to_owned()),
        )?;
        Ok(socket_id)
    }

    async fn output_socket_id(
        &mut self,
        component_id: ComponentId,
        socket: &str,
    ) -> ManagementResult<OutputSocketId> {
        let variant_id = self
            .component_schema_map
            .variant_for_component_id(self.ctx, component_id)
            .await?;
        // if the map was already constructed this does nothing
        self.socket_map
            .add_sockets_for_variant(self.ctx, variant_id)
            .await?;
        let socket_id = self.socket_map.output_socket_id(variant_id, socket).ok_or(
            ManagementError::OutputSocketDoesNotExist(component_id, socket.to_owned()),
        )?;
        Ok(socket_id)
    }

    async fn prepare_for_connection(
        &mut self,
        component_id: ComponentId,
        connection: &ManagementConnection,
    ) -> ManagementResult<(ComponentId, OutputSocketId, ComponentId, InputSocketId)> {
        // Figure out socket direction
        let (from_component_id, from_socket, to_component_id, to_socket) = match connection {
            ManagementConnection::Input { from, to } => (
                self.get_managed_or_external_component_id(&from.component)?,
                &from.socket,
                component_id,
                to,
            ),
            ManagementConnection::Output { from, to } => (
                component_id,
                from,
                self.get_managed_or_external_component_id(&to.component)?,
                &to.socket,
            ),
        };

        // Look up the sockets and return
        Ok((
            from_component_id,
            self.output_socket_id(from_component_id, from_socket)
                .await?,
            to_component_id,
            self.input_socket_id(to_component_id, to_socket).await?,
        ))
    }

    async fn create_connection(
        &mut self,
        component_id: ComponentId,
        connection: &ManagementConnection,
    ) -> ManagementResult<()> {
        let (
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        ) = self
            .prepare_for_connection(component_id, connection)
            .await?;

        if let Some(connection_apa_id) = Component::connect(
            self.ctx,
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        )
        .await?
        {
            let apa = AttributePrototypeArgument::get_by_id(self.ctx, connection_apa_id).await?;
            let created_info = {
                let history_actor = self.ctx.history_actor();
                let actor = ActorView::from_history_actor(self.ctx, *history_actor).await?;
                HistoryEventMetadata {
                    actor,
                    timestamp: apa.timestamp().created_at,
                }
            };
            let incoming_connection = Connection {
                attribute_prototype_argument_id: connection_apa_id,
                to_component_id: destination_component_id,
                to_input_socket_id: destination_input_socket_id,
                from_component_id: source_component_id,
                from_output_socket_id: source_output_socket_id,
                created_info,
                deleted_info: None,
            };
            let edge = SummaryDiagramEdge::assemble_just_added(incoming_connection)?;

            WsEvent::connection_upserted(self.ctx, edge.into())
                .await?
                .publish_on_commit(self.ctx)
                .await?;
        }

        Ok(())
    }

    async fn remove_connection(
        &mut self,
        component_id: ComponentId,
        connection: &ManagementConnection,
    ) -> ManagementResult<()> {
        let (
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        ) = self
            .prepare_for_connection(component_id, connection)
            .await?;

        Component::remove_connection(
            self.ctx,
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        )
        .await?;

        WsEvent::connection_deleted(
            self.ctx,
            source_component_id,
            destination_component_id,
            source_output_socket_id,
            destination_input_socket_id,
        )
        .await?
        .publish_on_commit(self.ctx)
        .await?;

        Ok(())
    }

    async fn set_parent(
        &self,
        child_id: ComponentId,
        parent_placeholder: &str,
    ) -> ManagementResult<(ComponentId, Option<InferredEdgeChanges>)> {
        let new_parent_id = self.get_managed_component_id(parent_placeholder)?;
        let inferred_edges =
            Frame::upsert_parent_no_events(self.ctx, child_id, new_parent_id).await?;

        Ok((new_parent_id, inferred_edges))
    }

    async fn manage(
        &self,
        component_id: ComponentId,
        managed_schema_id: SchemaId,
    ) -> ManagementResult<()> {
        let cycle_check_guard = self.ctx.workspace_snapshot()?.enable_cycle_check().await;
        Component::add_manages_edge_to_component(
            self.ctx,
            self.manager_component_id,
            component_id,
            EdgeWeightKind::Manages,
        )
        .await?;
        drop(cycle_check_guard);

        let edge = SummaryDiagramManagementEdge::new(
            self.manager_schema_id,
            managed_schema_id,
            self.manager_component_id,
            component_id,
        );

        WsEvent::connection_upserted(self.ctx, edge.into())
            .await?
            .publish_on_commit(self.ctx)
            .await?;

        Ok(())
    }

    async fn creates(&mut self) -> ManagementResult<Vec<PendingOperation>> {
        // We take here to avoid holding on to an immutable ref to self throughout the loop
        let creates = self.operations.create.take();

        let mut pending_operations = vec![];

        if let Some(creates) = creates {
            // Create the components in a separate pass from update, so we can make connections
            // between the created components.
            let mut created = vec![];
            for (placeholder, operation) in creates {
                if placeholder == SELF_ID {
                    return Err(ManagementError::CannotCreateComponentWithSelfPlaceholder);
                }

                if self.component_id_placeholders.contains_key(&placeholder) {
                    return Err(ManagementError::DuplicateComponentPlaceholder(placeholder));
                }

                let CreatedComponent {
                    component,
                    geometry,
                    schema_id,
                } = self.create_component(&placeholder, &operation).await?;

                self.created_components
                    .insert(component.id(), geometry.keys().copied().collect());

                self.component_id_placeholders
                    .insert(placeholder, component.id());

                created.push((component.id(), geometry, schema_id, operation))
            }

            // Now that all components have been created, handle properties, attributes and
            // connections between them.
            for (
                component_id,
                geometry,
                schema_id,
                ManagementCreateOperation {
                    kind: _,
                    properties,
                    attributes,
                    geometry: _,
                    connect,
                    parent,
                },
            ) in created
            {
                if let Some(properties) = properties {
                    let controlling_funcs =
                        Component::list_av_controlling_func_ids_for_id(self.ctx, component_id)
                            .await?;

                    update_component(
                        self.ctx,
                        component_id,
                        properties,
                        &[&["root", "si", "name"]],
                        controlling_funcs,
                    )
                    .await?;
                }
                if let Some(attributes) = attributes {
                    crate::update_attributes_without_validation(self.ctx, component_id, attributes)
                        .await?;
                }

                if let Some(connections) = connect {
                    for create in connections {
                        pending_operations.push(PendingOperation::Connect(PendingConnect {
                            component_id,
                            connection: create.to_owned(),
                        }));
                    }
                }

                self.last_component_geometry.extend(geometry);

                if let Some(parent) = parent {
                    pending_operations.push(PendingOperation::Parent(PendingParent {
                        child_component_id: component_id,
                        parent,
                    }));
                }

                pending_operations.push(PendingOperation::Manage(PendingManage {
                    managed_component_id: component_id,
                    managed_component_schema_id: schema_id,
                }));
            }
        }

        Ok(pending_operations)
    }

    fn get_managed_component_id_opt(&self, placeholder: &str) -> Option<ComponentId> {
        self.component_id_placeholders
            .get(placeholder)
            .or_else(|| self.managed_component_id_placeholders.get(placeholder))
            .copied()
    }

    fn get_managed_component_id(&self, placeholder: &str) -> ManagementResult<ComponentId> {
        self.get_managed_component_id_opt(placeholder).ok_or(
            ManagementError::ComponentWithPlaceholderNotFound(placeholder.to_owned()),
        )
    }

    fn get_managed_or_external_component_id(
        &self,
        placeholder_or_id: &str,
    ) -> ManagementResult<ComponentId> {
        match self.get_managed_component_id_opt(placeholder_or_id) {
            Some(component_id) => Ok(component_id),
            // If we don't find the component by name, see if it's a component id and use that!
            None => placeholder_or_id.parse().map_err(|_| {
                ManagementError::ComponentWithPlaceholderNotFound(placeholder_or_id.to_owned())
            }),
        }
    }

    async fn updates(&mut self) -> ManagementResult<Vec<PendingOperation>> {
        let mut pending = vec![];

        let updates = self.operations.update.take();
        let Some(updates) = updates else {
            return Ok(pending);
        };

        for (placeholder, operation) in updates {
            let ManagementUpdateOperation {
                properties,
                attributes,
                geometry,
                connect,
                parent,
            } = operation;
            let component_id = self.get_managed_component_id(&placeholder)?;
            let mut component = Component::get_by_id(self.ctx, component_id).await?;
            let mut view_ids = HashSet::new();

            let will_be_frame =
                component_will_be_frame(self.ctx, &component, properties.as_ref()).await?;

            for geometry_id in Geometry::list_ids_by_component(self.ctx, component_id).await? {
                let view_id = Geometry::get_view_id_by_id(self.ctx, geometry_id).await?;
                view_ids.insert(view_id);
            }

            // we have to ensure frames get a size
            let geometries = if will_be_frame
                && geometry
                    .as_ref()
                    .is_none_or(|geometries| geometries.is_empty())
            {
                let mut geometries = HashMap::new();
                for &view_id in &view_ids {
                    if let Some(geo) =
                        Geometry::try_get_by_component_and_view(self.ctx, component_id, view_id)
                            .await?
                    {
                        geometries.insert(view_id, (geo.into_raw().into(), true));
                    }
                }
                geometries
            } else if let Some(update_geometries) = geometry {
                let mut geometries = HashMap::new();

                for (view_name, geometry) in update_geometries {
                    let view_id = self
                        .view_placeholders
                        .get(&view_name)
                        .copied()
                        .ok_or(ManagementError::NoSuchView(view_name.to_owned()))?;
                    geometries.insert(view_id, (geometry, false));
                }

                geometries
            } else {
                HashMap::new()
            };

            for (view_id, (mut view_geometry, is_current)) in geometries {
                let current_geometry: ManagementGeometry = if is_current {
                    view_geometry
                } else {
                    match Geometry::try_get_by_component_and_view(self.ctx, component_id, view_id)
                        .await?
                    {
                        Some(geometry) => geometry.into_raw().into(),
                        None => {
                            view_ids.insert(view_id);
                            Component::add_to_view(
                                self.ctx,
                                component_id,
                                view_id,
                                view_geometry.into(),
                            )
                            .await?;
                            view_geometry
                        }
                    }
                };

                view_geometry
                    .x
                    .get_or_insert(current_geometry.x.unwrap_or(0.0));
                view_geometry
                    .y
                    .get_or_insert(current_geometry.y.unwrap_or(0.0));
                if let Some(current_width) = current_geometry.width {
                    view_geometry.width.get_or_insert(current_width);
                }
                if let Some(current_height) = current_geometry.height {
                    view_geometry.width.get_or_insert(current_height);
                }

                // Ensure frames have a width and height
                if view_geometry.width.zip(view_geometry.height).is_none() && will_be_frame {
                    view_geometry.width = Some(500.0);
                    view_geometry.height = Some(500.0);
                }

                component
                    .set_raw_geometry(self.ctx, view_geometry.into(), view_id)
                    .await?;
            }
            let controlling_avs =
                Component::list_av_controlling_func_ids_for_id(self.ctx, component_id).await?;
            if let Some(properties) = properties {
                update_component(self.ctx, component_id, properties, &[], controlling_avs).await?;
            }
            if let Some(attributes) = attributes {
                crate::update_attributes_without_validation(self.ctx, component_id, attributes)
                    .await?;
            }

            if let Some(update_conns) = connect {
                if let Some(remove_conns) = &update_conns.remove {
                    for to_remove in remove_conns {
                        pending.push(PendingOperation::RemoveConnection(PendingConnect {
                            component_id,
                            connection: to_remove.to_owned(),
                        }));
                    }
                }

                if let Some(add_conns) = &update_conns.add {
                    for to_add in add_conns {
                        pending.push(PendingOperation::Connect(PendingConnect {
                            component_id,
                            connection: to_add.to_owned(),
                        }));
                    }
                }
            }

            if let Some(new_parent) = parent {
                pending.push(PendingOperation::Parent(PendingParent {
                    child_component_id: component_id,
                    parent: new_parent.to_owned(),
                }));
            }

            self.updated_components
                .insert(component_id, view_ids.iter().copied().collect());
        }

        Ok(pending)
    }

    async fn actions(&self) -> ManagementResult<()> {
        if let Some(actions) = &self.operations.actions {
            for (placeholder, operations) in actions {
                let component_id = self.get_managed_component_id(placeholder)?;

                operate_actions(self.ctx, component_id, operations).await?;
            }

            WsEvent::action_list_updated(self.ctx)
                .await?
                .publish_on_commit(self.ctx)
                .await?;
        }

        Ok(())
    }

    // Using the dep graph to ensure we send ws events for components in parent
    // to child order, so that parents exist in the frontend before their
    // children / parents are rendered as frames before their children report
    // their parentage. We have to send an update for every view for which the
    // component has a geometry.
    async fn send_component_ws_events(
        &mut self,
        mut parentage_graph: DependencyGraph<ComponentId>,
        inferred_edges_by_component_id: HashMap<ComponentId, Vec<SummaryDiagramInferredEdge>>,
    ) -> ManagementResult<()> {
        loop {
            let independent_ids = parentage_graph.independent_ids();
            if independent_ids.is_empty() {
                break;
            }
            for id in independent_ids {
                if let Some(view_ids) = self.created_components.get(&id) {
                    self.send_created_event(
                        id,
                        view_ids,
                        inferred_edges_by_component_id.get(&id).cloned(),
                    )
                    .await?;
                    self.created_components.remove(&id);
                } else if let Some(view_ids) = self.updated_components.get(&id) {
                    self.send_updated_event(id, view_ids).await?;
                    self.updated_components.remove(&id);
                }
                parentage_graph.remove_id(id);
            }
        }

        for (&created_id, view_ids) in &self.created_components {
            self.send_created_event(
                created_id,
                view_ids,
                inferred_edges_by_component_id.get(&created_id).cloned(),
            )
            .await?;
        }

        for (&updated_id, view_ids) in &self.updated_components {
            self.send_updated_event(updated_id, view_ids).await?;
        }

        Ok(())
    }

    async fn send_created_event(
        &self,
        id: ComponentId,
        view_ids: &[ViewId],
        inferred_edges: Option<Vec<SummaryDiagramInferredEdge>>,
    ) -> ManagementResult<()> {
        let component = Component::get_by_id(self.ctx, id).await?;

        for &view_id in view_ids {
            WsEvent::component_created_with_inferred_edges(
                self.ctx,
                component
                    .into_frontend_type(
                        self.ctx,
                        Some(&component.geometry(self.ctx, view_id).await?),
                        Added,
                        &mut HashMap::new(),
                    )
                    .await?,
                inferred_edges.clone(),
            )
            .await?
            .publish_on_commit(self.ctx)
            .await?;
        }

        Ok(())
    }
    async fn send_updated_event(
        &self,
        id: ComponentId,
        view_ids: &[ViewId],
    ) -> ManagementResult<()> {
        let component = Component::get_by_id(self.ctx, id).await?;
        for &view_id in view_ids {
            WsEvent::component_updated(
                self.ctx,
                component
                    .into_frontend_type(
                        self.ctx,
                        Some(&component.geometry(self.ctx, view_id).await?),
                        component.change_status(self.ctx).await?,
                        &mut HashMap::new(),
                    )
                    .await?,
            )
            .await?
            .publish_on_commit(self.ctx)
            .await?;
        }

        Ok(())
    }

    async fn create_views(&mut self) -> ManagementResult<()> {
        let Some(create_views) = self
            .operations
            .views
            .as_ref()
            .and_then(|view_ops| view_ops.create.to_owned())
        else {
            return Ok(());
        };

        for new_view_name in create_views {
            if View::find_by_name(self.ctx, &new_view_name)
                .await?
                .is_some()
            {
                // view already exists, just skip it
                continue;
            }

            let new_view = View::new(self.ctx, &new_view_name).await?;
            let view_id = new_view.id();
            let view_view = ViewView::from_view(self.ctx, new_view).await?;

            self.ctx
                .write_audit_log(
                    AuditLogKind::CreateView { view_id },
                    new_view_name.to_owned(),
                )
                .await?;

            WsEvent::view_created(self.ctx, view_view)
                .await?
                .publish_on_commit(self.ctx)
                .await?;

            self.view_placeholders.insert(new_view_name, view_id);
        }

        Ok(())
    }

    async fn remove_views(&mut self) -> ManagementResult<()> {
        let Some(remove_views) = self
            .operations
            .views
            .as_ref()
            .and_then(|view_ops| view_ops.remove.to_owned())
        else {
            return Ok(());
        };

        for view_to_remove in remove_views {
            let Some(&view_id) = self.view_placeholders.get(&view_to_remove) else {
                continue;
            };

            if View::remove(self.ctx, view_id).await.is_ok() {
                WsEvent::view_deleted(self.ctx, view_id)
                    .await?
                    .publish_on_commit(self.ctx)
                    .await?;

                self.ctx
                    .write_audit_log(
                        AuditLogKind::DeleteView { view_id },
                        view_to_remove.to_owned(),
                    )
                    .await?;

                self.view_placeholders.remove(&view_to_remove);
            }
        }

        Ok(())
    }

    async fn remove_from_views(&mut self) -> ManagementResult<()> {
        let Some(removes) = self.operations.remove.take() else {
            return Ok(());
        };

        let mut removed_components: HashMap<ViewId, ViewComponentsUpdateSingle> = HashMap::new();

        for (view_placeholder, component_placeholders) in removes {
            let Some(view_id) = self.view_placeholders.get(&view_placeholder).copied() else {
                continue;
            };

            for component_placeholder in component_placeholders {
                let component_id = self.get_managed_component_id(&component_placeholder)?;
                if let Some(geometry) =
                    Geometry::try_get_by_component_and_view(self.ctx, component_id, view_id).await?
                {
                    let removed_from_view = match Geometry::remove(self.ctx, geometry.id()).await {
                        Ok(_) => true,
                        Err(DiagramError::DeletingLastGeometryForComponent(_, _)) => false,
                        Err(err) => {
                            return Err(err)?;
                        }
                    };

                    if removed_from_view {
                        removed_components
                            .entry(view_id)
                            .or_default()
                            .removed
                            .insert(component_id);
                    }
                }
            }
        }

        if !removed_components.is_empty() {
            WsEvent::view_components_update(self.ctx, removed_components)
                .await?
                .publish_on_commit(self.ctx)
                .await?;
        }

        Ok(())
    }

    async fn deletes(&mut self, force_erase: bool) -> ManagementResult<()> {
        let Some(deletes) = (if force_erase {
            self.operations.erase.take()
        } else {
            self.operations.delete.take()
        }) else {
            return Ok(());
        };

        let mut component_ids_to_delete = vec![];
        for placeholder in &deletes {
            component_ids_to_delete.push(self.get_managed_component_id(placeholder)?);
        }

        let deletion_statuses =
            delete_components(self.ctx, &component_ids_to_delete, force_erase).await?;

        for placeholder in &deletes {
            let component_id = self.get_managed_component_id(placeholder)?;
            if let Some(
                ComponentDeletionStatus::Deleted | ComponentDeletionStatus::StillExistsOnHead,
            ) = deletion_statuses.get(&component_id)
            {
                self.managed_component_id_placeholders.remove(placeholder);
                self.managed_component_id_placeholders
                    .remove(&component_id.to_string());
            }
        }

        Ok(())
    }

    pub async fn operate(&mut self) -> ManagementResult<Option<Vec<ComponentId>>> {
        self.deletes(true).await?;
        self.deletes(false).await?;
        self.create_views().await?;
        let mut pending_operations = self.creates().await?;
        let mut component_graph = DependencyGraph::new();
        pending_operations.extend(self.updates().await?);
        let mut inferred_edges_by_component_id = HashMap::new();

        // Parents have to be set before component events are sent

        let mut new_pending_ops_from_parentage = vec![];
        for pending_parent in pending_operations
            .iter()
            .filter_map(|pending_op| match pending_op {
                PendingOperation::Parent(pending_parent) => Some(pending_parent),
                _ => None,
            })
        {
            let (parent_id, inferred_edges) = self
                .set_parent(pending_parent.child_component_id, &pending_parent.parent)
                .await?;
            if let Some(inferred_edges) = inferred_edges {
                inferred_edges_by_component_id.insert(
                    pending_parent.child_component_id,
                    inferred_edges.upserted_edges.clone(),
                );

                // Inferred edge events should also come after all component events
                if !inferred_edges.upserted_edges.is_empty() {
                    new_pending_ops_from_parentage.push(PendingOperation::InferredEdgeUpsertEvent(
                        inferred_edges.upserted_edges,
                    ));
                }
                if !inferred_edges.removed_edges.is_empty() {
                    new_pending_ops_from_parentage.push(PendingOperation::InferredEdgeRemoveEvent(
                        inferred_edges.removed_edges,
                    ));
                }
            }

            component_graph.id_depends_on(pending_parent.child_component_id, parent_id);
        }

        let created_component_ids = (!self.created_components.is_empty())
            .then_some(self.created_components.keys().copied().collect());

        self.send_component_ws_events(component_graph, inferred_edges_by_component_id)
            .await?;

        // Now, the rest of the pending ops can be executed, which need to have
        // their wsevents sent *after* the component ws events (otherwise some
        // will be discarded by the frontend, since it does not know about the
        // newly created components until the above events are sent)
        for pending_op in pending_operations
            .into_iter()
            .chain(new_pending_ops_from_parentage.into_iter())
        {
            // Signal when we process a pending operation.
            WsEvent::management_operations_in_progress(self.ctx, self.request_ulid)
                .await?
                .publish_immediately(self.ctx)
                .await?;

            match pending_op {
                PendingOperation::Connect(pending_connect) => {
                    self.create_connection(
                        pending_connect.component_id,
                        &pending_connect.connection,
                    )
                    .await?;
                }
                PendingOperation::RemoveConnection(remove) => {
                    self.remove_connection(remove.component_id, &remove.connection)
                        .await?;
                }
                PendingOperation::Manage(PendingManage {
                    managed_component_id,
                    managed_component_schema_id,
                }) => {
                    self.manage(managed_component_id, managed_component_schema_id)
                        .await?;
                }
                PendingOperation::InferredEdgeRemoveEvent(removed_edges) => {
                    WsEvent::remove_inferred_edges(self.ctx, removed_edges)
                        .await?
                        .publish_on_commit(self.ctx)
                        .await?;
                }
                PendingOperation::InferredEdgeUpsertEvent(upserted_edges) => {
                    WsEvent::upsert_inferred_edges(self.ctx, upserted_edges)
                        .await?
                        .publish_on_commit(self.ctx)
                        .await?;
                }
                PendingOperation::Parent(_) => {}
            }
        }

        self.remove_from_views().await?;
        self.remove_views().await?;
        self.actions().await?;

        Ok(created_component_ids)
    }
}

async fn operate_actions(
    ctx: &DalContext,
    component_id: ComponentId,
    operation: &ManagementActionOperation,
) -> ManagementResult<()> {
    if let Some(remove_actions) = &operation.remove {
        for to_remove in remove_actions
            .iter()
            .map(|action| ActionIdentifier::from(action.as_str()))
        {
            remove_action(ctx, component_id, to_remove).await?;
        }
    }
    if let Some(add_actions) = &operation.add {
        let sv_id = Component::schema_variant_id(ctx, component_id).await?;
        let available_actions = ActionPrototype::for_variant(ctx, sv_id).await?;
        for action in add_actions
            .iter()
            .map(|action| ActionIdentifier::from(action.as_str()))
        {
            add_action(ctx, component_id, action, &available_actions).await?;
        }
    }

    Ok(())
}

async fn remove_action(
    ctx: &DalContext,
    component_id: ComponentId,
    action: ActionIdentifier,
) -> ManagementResult<()> {
    let actions = Action::find_for_kind_and_component_id(ctx, component_id, action.kind).await?;
    match action.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Refresh | ActionKind::Update => {
            for action_id in actions {
                Action::remove_by_id(ctx, action_id).await?;
            }
        }
        ActionKind::Manual => {
            for action_id in actions {
                let prototype_id = Action::prototype_id(ctx, action_id).await?;
                let func_id = ActionPrototype::func_id(ctx, prototype_id).await?;
                let func = Func::get_by_id(ctx, func_id).await?;
                if Some(func.name) == action.manual_func_name {
                    Action::remove_by_id(ctx, action_id).await?;
                }
            }
        }
    }

    Ok(())
}

async fn add_action(
    ctx: &DalContext,
    component_id: ComponentId,
    action: ActionIdentifier,
    available_actions: &[ActionPrototype],
) -> ManagementResult<()> {
    let prototype_id = match action.kind {
        ActionKind::Create | ActionKind::Destroy | ActionKind::Refresh | ActionKind::Update => {
            if !Action::find_for_kind_and_component_id(ctx, component_id, action.kind)
                .await?
                .is_empty()
            {
                return Ok(());
            }

            let Some(action_prototype) = available_actions
                .iter()
                .find(|proto| proto.kind == action.kind)
            else {
                return Err(ManagementError::ComponentDoesNotHaveAction(
                    action.kind,
                    component_id,
                ));
            };

            action_prototype.id()
        }
        ActionKind::Manual => {
            let Some(manual_func_name) = action.manual_func_name else {
                return Err(ManagementError::ComponentDoesNotHaveAction(
                    ActionKind::Manual,
                    component_id,
                ));
            };

            let mut proto_id = None;
            for manual_proto in available_actions
                .iter()
                .filter(|proto| proto.kind == ActionKind::Manual)
            {
                let func =
                    Func::get_by_id(ctx, ActionPrototype::func_id(ctx, manual_proto.id()).await?)
                        .await?;
                if func.name == manual_func_name {
                    proto_id = Some(manual_proto.id());
                    break;
                }
            }

            let Some(proto_id) = proto_id else {
                return Err(ManagementError::ComponentDoesNotHaveManualAction(
                    manual_func_name,
                    component_id,
                ));
            };

            proto_id
        }
    };

    Action::new(ctx, prototype_id, Some(component_id)).await?;

    Ok(())
}

// Update operations should not be able to set these props or their children
const IGNORE_PATHS: [&[&str]; 5] = [
    &["root", "code"],
    &["root", "deleted_at"],
    &["root", "qualification"],
    &["root", "resource_value"],
    &["root", "secrets"],
];

const ROOT_SI_TYPE_PATH: &[&str] = &["root", "si", "type"];
const RESOURCE_PATH: &[&str] = &["root", "resource"];

async fn update_component(
    ctx: &DalContext,
    component_id: ComponentId,
    properties: serde_json::Value,
    extra_ignore_paths: &[&[&str]],
    controlling_avs: HashMap<AttributeValueId, ControllingFuncData>,
) -> ManagementResult<()> {
    let variant_id = Component::schema_variant_id(ctx, component_id).await?;

    // walk the properties serde_json::Value object without recursion
    let mut work_queue = VecDeque::new();
    work_queue.push_back((vec!["root".to_string()], properties));

    while let Some((path, current_val)) = work_queue.pop_front() {
        let path_as_refs: Vec<_> = path.iter().map(|part| part.as_str()).collect();
        if IGNORE_PATHS.contains(&path_as_refs.as_slice())
            || extra_ignore_paths.contains(&path_as_refs.as_slice())
        {
            continue;
        }

        let Some(prop_id) =
            Prop::find_prop_id_by_path_opt(ctx, variant_id, &PropPath::new(path.as_slice()))
                .await?
        else {
            continue;
        };

        let path_attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;

        if AttributeValue::is_set_by_dependent_function(ctx, path_attribute_value_id).await?
            && controlling_avs
                .get_key_value(&path_attribute_value_id)
                .map(|(_, value)| value.av_id)
                .unwrap_or(path_attribute_value_id)
                != path_attribute_value_id
        {
            continue;
        }

        // component type has to be special cased
        if path_as_refs.as_slice() == ROOT_SI_TYPE_PATH {
            let Ok(new_type) = serde_json::from_value::<ComponentType>(current_val.to_owned())
            else {
                // error here?
                continue;
            };
            Component::set_type_by_id(ctx, component_id, new_type).await?;

            continue;
        }
        // handle resource special as well
        if path_as_refs.as_slice() == RESOURCE_PATH {
            let resource_data = ResourceData::new(
                veritech_client::ResourceStatus::Ok,
                Some(current_val.to_owned()),
            );
            let component = Component::get_by_id(ctx, component_id).await?;
            component.set_resource(ctx, resource_data).await?;
            continue;
        }

        if let serde_json::Value::Null = current_val {
            AttributeValue::update(ctx, path_attribute_value_id, Some(current_val.to_owned()))
                .await?;
            continue;
        }

        let prop = Prop::get_by_id(ctx, prop_id).await?;

        match prop.kind {
            PropKind::String
            | PropKind::Boolean
            | PropKind::Integer
            | PropKind::Float
            | PropKind::Json => {
                // todo: type check!
                let view = AttributeValue::view(ctx, path_attribute_value_id).await?;
                if Some(&current_val) != view.as_ref() {
                    AttributeValue::update(ctx, path_attribute_value_id, Some(current_val)).await?;
                }
            }
            PropKind::Object => {
                let serde_json::Value::Object(obj) = current_val else {
                    continue;
                };

                for (key, value) in obj {
                    let mut new_path = path.clone();
                    new_path.push(key);
                    work_queue.push_back((new_path, value));
                }
            }
            PropKind::Map => {
                let serde_json::Value::Object(map) = current_val else {
                    continue;
                };

                let map_children =
                    AttributeValue::map_children(ctx, path_attribute_value_id).await?;

                // Remove any children that are not in the new map
                for (key, child_id) in &map_children {
                    if !map.contains_key(key) {
                        if AttributeValue::is_set_by_dependent_function(ctx, *child_id).await? {
                            continue;
                        }

                        AttributeValue::remove(ctx, *child_id).await?;
                    }
                }

                // We do not descend below a map. Instead we update the *entire*
                // child tree of each map key
                for (key, value) in map {
                    match map_children.get(&key) {
                        Some(child_id) => {
                            if AttributeValue::is_set_by_dependent_function(ctx, *child_id).await? {
                                continue;
                            }
                            let view = AttributeValue::view(ctx, *child_id).await?;
                            if Some(&value) != view.as_ref() {
                                AttributeValue::update(ctx, *child_id, Some(value)).await?;
                            }
                        }
                        None => {
                            AttributeValue::insert(
                                ctx,
                                path_attribute_value_id,
                                Some(value),
                                Some(key),
                            )
                            .await?;
                        }
                    }
                }
            }
            PropKind::Array => {
                if matches!(current_val, serde_json::Value::Array(_)) {
                    let view = AttributeValue::view(ctx, path_attribute_value_id).await?;

                    if Some(&current_val) != view.as_ref() {
                        // Just update the entire array whole cloth
                        AttributeValue::update(ctx, path_attribute_value_id, Some(current_val))
                            .await?;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn component_will_be_frame(
    ctx: &DalContext,
    component: &Component,
    new_properties: Option<&serde_json::Value>,
) -> ManagementResult<bool> {
    if type_being_set(new_properties).is_some_and(|c_type| c_type.is_frame()) {
        return Ok(true);
    }

    Ok(component.get_type(ctx).await?.is_frame())
}

fn type_being_set(properties: Option<&serde_json::Value>) -> Option<ComponentType> {
    let mut work_queue = VecDeque::from([("root", properties?)]);

    while let Some((path, current_val)) = work_queue.pop_front() {
        let match_key = match path {
            "root" => "si",
            "si" => "type",
            "type" => {
                let Ok(new_type) = serde_json::from_value::<ComponentType>(current_val.to_owned())
                else {
                    break;
                };
                return Some(new_type);
            }
            _ => break,
        };

        let serde_json::Value::Object(map) = current_val else {
            break;
        };

        let Some(next_value) = map.get(match_key) else {
            break;
        };

        work_queue.push_back((match_key, next_value));
    }

    None
}

fn process_geometry(
    geometry: &ManagementGeometry,
    default_x: Option<f64>,
    default_y: Option<f64>,
    origin_x: Option<f64>,
    origin_y: Option<f64>,
    will_be_frame: bool,
) -> ManagementGeometry {
    let mut geometry = geometry.to_owned();

    if will_be_frame && geometry.height.zip(geometry.width).is_none() {
        geometry.height = Some(DEFAULT_FRAME_HEIGHT);
        geometry.width = Some(DEFAULT_FRAME_WIDTH);
    }

    geometry.x.get_or_insert(default_x.unwrap_or(0.0));
    geometry.y.get_or_insert(default_y.unwrap_or(0.0));

    geometry.offset_by(origin_x, origin_y)
}

impl From<ActionError> for ManagementError {
    fn from(value: ActionError) -> Self {
        Box::new(value).into()
    }
}

impl From<ActionPrototypeError> for ManagementError {
    fn from(value: ActionPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<crate::attribute::attributes::AttributesError> for ManagementError {
    fn from(value: crate::attribute::attributes::AttributesError) -> Self {
        Box::new(value).into()
    }
}

impl From<DiagramError> for ManagementError {
    fn from(value: DiagramError) -> Self {
        Box::new(value).into()
    }
}

impl From<FrameError> for ManagementError {
    fn from(value: FrameError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for ManagementError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for ManagementError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<WsEventError> for ManagementError {
    fn from(value: WsEventError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for ManagementError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for ManagementError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for ManagementError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for ManagementError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for ManagementError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaError> for ManagementError {
    fn from(value: SchemaError) -> Self {
        Box::new(value).into()
    }
}
