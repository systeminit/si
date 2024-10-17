use std::collections::{hash_map, HashMap, VecDeque};

use prototype::ManagementPrototypeExecution;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use veritech_client::{ManagementFuncStatus, ManagementResultSuccess};

use crate::diagram::view::View;
use crate::diagram::DiagramError;
use crate::{
    action::{
        prototype::{ActionKind, ActionPrototype, ActionPrototypeError},
        Action, ActionError,
    },
    attribute::{
        prototype::argument::{AttributePrototypeArgument, AttributePrototypeArgumentError},
        value::AttributeValueError,
    },
    change_status::ChangeStatus::Added,
    component::IncomingConnection,
    diagram::{geometry::RawGeometry, SummaryDiagramEdge},
    history_event::HistoryEventMetadata,
    prop::{PropError, PropPath},
    socket::{input::InputSocketError, output::OutputSocketError},
    ActorView, AttributeValue, Component, ComponentError, ComponentId, DalContext, Func, FuncError,
    InputSocket, InputSocketId, OutputSocket, OutputSocketId, Prop, PropKind, Schema, SchemaError,
    SchemaId, SchemaVariantId, StandardModelError, WsEvent, WsEventError,
};

pub mod prototype;

#[derive(Debug, Error)]
pub enum ManagementError {
    #[error("action error: {0}")]
    Action(#[from] ActionError),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("cannot create component with 'self' as a placeholder")]
    CannotCreateComponentWithSelfPlaceholder,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("cannot add an action of kind {0} because component {1} does not have an action of that kind")]
    ComponentDoesNotHaveAction(ActionKind, ComponentId),
    #[error(
        "cannot add a manual action named {0} because component {1} does not have a manual action with that name"
    )]
    ComponentDoesNotHaveManualAction(String, ComponentId),
    #[error("Component with management placeholder {0} could not be found")]
    ComponentWithPlaceholderNotFound(String),
    #[error("Diagram Error {0}")]
    Diagram(#[from] DiagramError),
    #[error("Duplicate component placeholder {0}")]
    DuplicateComponentPlaceholder(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("Cannot connect component {0} to component {1} because component {1} does not have an input socket with name {2}")]
    InputSocketDoesNotExist(ComponentId, ComponentId, String),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("Cannot connect component {0} to component {1} because component {0} does not have an output socket with name {2}")]
    OutputSocketDoesNotExist(ComponentId, ComponentId, String),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("Cannot create component for Schema {0}, this schema does not exist or is not managed by this component")]
    SchemaDoesNotExist(String),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ManagementResult<T> = Result<T, ManagementError>;

/// Geometry type for deserialization lang-js, so even if we should only care about integers,
/// until we implement custom deserialization we can't merge it with [RawGeometry](RawGeometry)
#[derive(Clone, Debug, Copy, Serialize, Deserialize, PartialEq)]
pub struct NumericGeometry {
    pub x: f64,
    pub y: f64,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

impl NumericGeometry {
    pub fn offset_by(&self, mut x_off: f64, mut y_off: f64) -> Self {
        if !x_off.is_normal() {
            x_off = 0.0;
        }
        if !y_off.is_normal() {
            y_off = 0.0;
        }

        let x = if self.x.is_normal() {
            self.x + x_off
        } else {
            x_off
        };

        let y = if self.y.is_normal() {
            self.y + y_off
        } else {
            y_off
        };

        Self {
            x,
            y,
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

impl From<NumericGeometry> for RawGeometry {
    fn from(value: NumericGeometry) -> Self {
        Self {
            x: value.x as isize,
            y: value.y as isize,
            width: value.width.map(|w| w as isize),
            height: value.height.map(|h| h as isize),
        }
    }
}

impl From<RawGeometry> for NumericGeometry {
    fn from(value: RawGeometry) -> Self {
        Self {
            x: value.x as f64,
            y: value.y as f64,
            width: value.width.map(|w| w as f64),
            height: value.height.map(|h| h as f64),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionIdentifier {
    pub component: String,
    pub socket: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementConnection {
    from: String,
    to: ConnectionIdentifier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateConnections {
    add: Option<Vec<ManagementConnection>>,
    remove: Option<Vec<ManagementConnection>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementUpdateOperation {
    properties: Option<serde_json::Value>,
    geometry: Option<RawGeometry>,
    connect: Option<ManagementUpdateConnections>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementCreateOperation {
    kind: Option<String>,
    properties: Option<serde_json::Value>,
    geometry: Option<NumericGeometry>,
    connect: Option<Vec<ManagementConnection>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionIdentifier {
    pub kind: ActionKind,
    pub manual_func_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagementActionOperation {
    add: Option<Vec<String>>,
    remove: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementOperations {
    create: Option<HashMap<String, ManagementCreateOperation>>,
    update: Option<HashMap<String, ManagementUpdateOperation>>,
    actions: Option<HashMap<String, ManagementActionOperation>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    manager_schema_id: SchemaId,
    last_component_geometry: NumericGeometry,
    operations: ManagementOperations,
    schema_map: HashMap<String, SchemaId>,
    component_id_placeholders: HashMap<String, ComponentId>,
    component_schema_map: ComponentSchemaMap,
    socket_map: VariantSocketMap,
}

impl<'a> ManagementOperator<'a> {
    pub async fn new(
        ctx: &'a DalContext,
        manager_component_id: ComponentId,
        operations: ManagementOperations,
        management_execution: ManagementPrototypeExecution,
    ) -> ManagementResult<Self> {
        let mut component_id_placeholders = management_execution.placeholders;
        component_id_placeholders.insert(SELF_ID.to_string(), manager_component_id);

        let mut component_schema_map = ComponentSchemaMap::new();
        let manager_schema_id = component_schema_map
            .schema_for_component_id(ctx, manager_component_id)
            .await?;
        component_schema_map
            .variant_for_component_id(ctx, manager_component_id)
            .await?;

        Ok(Self {
            ctx,
            manager_component_id,
            manager_schema_id,
            last_component_geometry: management_execution.manager_component_geometry.into(),
            operations,
            schema_map: management_execution.managed_schema_map,
            component_id_placeholders,
            component_schema_map,
            socket_map: VariantSocketMap::new(),
        })
    }

    fn manager_schema_id(&self) -> SchemaId {
        self.manager_schema_id
    }

    async fn create_component(
        &self,
        placeholder: &str,
        operation: &ManagementCreateOperation,
    ) -> ManagementResult<(ComponentId, NumericGeometry)> {
        let schema_id = match &operation.kind {
            Some(kind) => self
                .schema_map
                .get(kind)
                .copied()
                .ok_or(ManagementError::SchemaDoesNotExist(kind.clone()))?,
            None => self.manager_schema_id(),
        };

        let variant_id = Schema::get_or_install_default_variant(self.ctx, schema_id).await?;

        let view_id = View::get_id_for_default(self.ctx).await?;
        let mut component = Component::new(self.ctx, placeholder, variant_id, view_id).await?;
        let geometry = if let Some(numeric_geometry) = &operation.geometry {
            component
                .set_raw_geometry(self.ctx, (*numeric_geometry).into(), view_id)
                .await?;

            *numeric_geometry
        } else {
            // We don't want to just stack components on top of each other if no
            // geometry is provided, so we're gonna do a bit of you-just-won
            // solitaire staggering
            let auto_geometry = self.last_component_geometry.offset_by(50.0, 50.0);
            component
                .set_raw_geometry(self.ctx, auto_geometry.into(), view_id)
                .await?;

            auto_geometry
        };

        WsEvent::component_created(
            self.ctx,
            component
                .into_frontend_type(
                    self.ctx,
                    Some(&component.geometry(self.ctx, view_id).await?),
                    Added,
                    &mut HashMap::new(),
                )
                .await?,
        )
        .await?
        .publish_on_commit(self.ctx)
        .await?;

        Ok((component.id(), geometry))
    }

    async fn prepare_for_connection(
        &mut self,
        source_component_id: ComponentId,
        connection: &ManagementConnection,
    ) -> ManagementResult<(ComponentId, OutputSocketId, ComponentId, InputSocketId)> {
        let from_variant_id = self
            .component_schema_map
            .variant_for_component_id(self.ctx, source_component_id)
            .await?;

        // if the map was already constructed this does nothing
        self.socket_map
            .add_sockets_for_variant(self.ctx, from_variant_id)
            .await?;

        let destination_component_id = self
            .component_id_placeholders
            .get(&connection.to.component)
            .copied()
            .ok_or(ManagementError::ComponentWithPlaceholderNotFound(
                connection.to.component.clone(),
            ))?;

        let to_variant_id = self
            .component_schema_map
            .variant_for_component_id(self.ctx, destination_component_id)
            .await?;

        self.socket_map
            .add_sockets_for_variant(self.ctx, to_variant_id)
            .await?;

        let source_output_socket_id = self
            .socket_map
            .output_socket_id(from_variant_id, &connection.from)
            .ok_or(ManagementError::OutputSocketDoesNotExist(
                source_component_id,
                destination_component_id,
                connection.from.to_owned(),
            ))?;

        let destination_input_socket_id = self
            .socket_map
            .input_socket_id(to_variant_id, &connection.to.socket)
            .ok_or(ManagementError::OutputSocketDoesNotExist(
                source_component_id,
                destination_component_id,
                connection.to.socket.to_owned(),
            ))?;

        Ok((
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        ))
    }

    async fn create_connection(
        &mut self,
        source_component_id: ComponentId,
        connection: &ManagementConnection,
    ) -> ManagementResult<()> {
        let (
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        ) = self
            .prepare_for_connection(source_component_id, connection)
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
            let incoming_connection = IncomingConnection {
                attribute_prototype_argument_id: connection_apa_id,
                to_component_id: destination_component_id,
                to_input_socket_id: destination_input_socket_id,
                from_component_id: source_component_id,
                from_output_socket_id: source_output_socket_id,
                created_info,
                deleted_info: None,
            };
            let edge = SummaryDiagramEdge::assemble_just_added(incoming_connection)?;

            WsEvent::connection_upserted(self.ctx, edge)
                .await?
                .publish_on_commit(self.ctx)
                .await?;
        }

        Ok(())
    }

    async fn remove_connection(
        &mut self,
        source_component_id: ComponentId,
        connection: &ManagementConnection,
    ) -> ManagementResult<()> {
        let (
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        ) = self
            .prepare_for_connection(source_component_id, connection)
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
        .await?;

        Ok(())
    }

    async fn creates(&mut self) -> ManagementResult<()> {
        // We take here to avoid holding on to an immutable ref to self throughout the loop
        let creates = self.operations.create.take();

        if let Some(creates) = &creates {
            for (placeholder, operation) in creates {
                if placeholder == SELF_ID {
                    return Err(ManagementError::CannotCreateComponentWithSelfPlaceholder);
                }

                if self.component_id_placeholders.contains_key(placeholder) {
                    return Err(ManagementError::DuplicateComponentPlaceholder(
                        placeholder.to_owned(),
                    ));
                }

                let (component_id, geometry) =
                    self.create_component(placeholder, operation).await?;

                self.last_component_geometry = geometry;

                self.component_id_placeholders
                    .insert(placeholder.to_owned(), component_id);

                if let Some(properties) = &operation.properties {
                    update_component(
                        self.ctx,
                        component_id,
                        properties,
                        &[&["root", "si", "name"]],
                    )
                    .await?;
                }

                if let Some(connections) = &operation.connect {
                    for create in connections {
                        self.create_connection(component_id, create).await?;
                    }
                }

                Component::add_manages_edge_to_component(
                    self.ctx,
                    self.manager_component_id,
                    component_id,
                    crate::EdgeWeightKind::Manages,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn get_real_component_id(&self, placeholder: &String) -> ManagementResult<ComponentId> {
        self.component_id_placeholders
            .get(placeholder)
            .copied()
            .ok_or(ManagementError::ComponentWithPlaceholderNotFound(
                placeholder.to_owned(),
            ))
    }

    async fn updates(&mut self) -> ManagementResult<()> {
        let updates = self.operations.update.take();
        let Some(updates) = &updates else {
            return Ok(());
        };

        for (placeholder, operation) in updates {
            let component_id = self.get_real_component_id(placeholder).await?;

            if let Some(properties) = &operation.properties {
                update_component(self.ctx, component_id, properties, &[]).await?;
            }

            if let Some(update_conns) = &operation.connect {
                if let Some(remove_conns) = &update_conns.remove {
                    for to_remove in remove_conns {
                        self.remove_connection(component_id, to_remove).await?;
                    }
                }

                if let Some(add_conns) = &update_conns.add {
                    for to_add in add_conns {
                        self.create_connection(component_id, to_add).await?;
                    }
                }
            }

            if let Some(raw_geometry) = &operation.geometry {
                let view_id = View::get_id_for_default(self.ctx).await?;
                let mut component = Component::get_by_id(self.ctx, component_id).await?;
                component
                    .set_raw_geometry(self.ctx, raw_geometry.to_owned(), view_id)
                    .await?;
            }
        }

        Ok(())
    }

    async fn actions(&self) -> ManagementResult<()> {
        if let Some(actions) = &self.operations.actions {
            for (placeholder, operations) in actions {
                let component_id = self.get_real_component_id(placeholder).await?;

                operate_actions(self.ctx, component_id, operations).await?;
            }

            WsEvent::action_list_updated(self.ctx)
                .await?
                .publish_on_commit(self.ctx)
                .await?;
        }

        Ok(())
    }

    pub async fn operate(&mut self) -> ManagementResult<()> {
        self.creates().await?;
        self.updates().await?;
        self.actions().await?;

        Ok(())
    }
}

fn identify_action(action_str: &str) -> ActionIdentifier {
    match action_str {
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

async fn operate_actions(
    ctx: &DalContext,
    component_id: ComponentId,
    operation: &ManagementActionOperation,
) -> ManagementResult<()> {
    if let Some(remove_actions) = &operation.remove {
        for to_remove in remove_actions
            .iter()
            .map(|action| identify_action(action.as_str()))
        {
            remove_action(ctx, component_id, to_remove).await?;
        }
    }
    if let Some(add_actions) = &operation.add {
        let sv_id = Component::schema_variant_id(ctx, component_id).await?;
        let available_actions = ActionPrototype::for_variant(ctx, sv_id).await?;
        for action in add_actions
            .iter()
            .map(|action| identify_action(action.as_str()))
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
                let func = Func::get_by_id_or_error(ctx, func_id).await?;
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
                let func = Func::get_by_id_or_error(
                    ctx,
                    ActionPrototype::func_id(ctx, manual_proto.id()).await?,
                )
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
const IGNORE_PATHS: [&[&str]; 6] = [
    &["root", "code"],
    &["root", "deleted_at"],
    &["root", "qualification"],
    &["root", "resource"],
    &["root", "resource_value"],
    &["root", "secrets"],
];

async fn update_component(
    ctx: &DalContext,
    component_id: ComponentId,
    properties: &serde_json::Value,
    extra_ignore_paths: &[&[&str]],
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

        if AttributeValue::is_set_by_dependent_function(ctx, path_attribute_value_id).await? {
            continue;
        }
        if let serde_json::Value::Null = current_val {
            AttributeValue::update(ctx, path_attribute_value_id, Some(current_val.to_owned()))
                .await?;
            continue;
        }

        let prop = Prop::get_by_id(ctx, prop_id).await?;

        match prop.kind {
            PropKind::String | PropKind::Boolean | PropKind::Integer | PropKind::Json => {
                // todo: type check!
                let view = AttributeValue::get_by_id(ctx, path_attribute_value_id)
                    .await?
                    .view(ctx)
                    .await?;
                if Some(current_val) != view.as_ref() {
                    AttributeValue::update(
                        ctx,
                        path_attribute_value_id,
                        Some(current_val.to_owned()),
                    )
                    .await?;
                }
            }
            PropKind::Object => {
                let serde_json::Value::Object(obj) = current_val else {
                    continue;
                };

                for (key, value) in obj {
                    let mut new_path = path.clone();
                    new_path.push(key.to_owned());
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

                        AttributeValue::remove_by_id(ctx, *child_id).await?;
                    }
                }

                // We do not descend below a map. Instead we update the *entire*
                // child tree of each map key
                for (key, value) in map {
                    match map_children.get(key) {
                        Some(child_id) => {
                            if AttributeValue::is_set_by_dependent_function(ctx, *child_id).await? {
                                continue;
                            }
                            let view = AttributeValue::get_by_id(ctx, *child_id)
                                .await?
                                .view(ctx)
                                .await?;
                            if Some(value) != view.as_ref() {
                                AttributeValue::update(ctx, *child_id, Some(value.to_owned()))
                                    .await?;
                            }
                        }
                        None => {
                            AttributeValue::insert(
                                ctx,
                                path_attribute_value_id,
                                Some(value.to_owned()),
                                Some(key.to_owned()),
                            )
                            .await?;
                        }
                    }
                }
            }
            PropKind::Array => {
                if matches!(current_val, serde_json::Value::Array(_)) {
                    let view = AttributeValue::get_by_id(ctx, path_attribute_value_id)
                        .await?
                        .view(ctx)
                        .await?;

                    if Some(current_val) != view.as_ref() {
                        // Just update the entire array whole cloth
                        AttributeValue::update(
                            ctx,
                            path_attribute_value_id,
                            Some(current_val.to_owned()),
                        )
                        .await?;
                    }
                }
            }
        }
    }

    let component = Component::get_by_id(ctx, component_id).await?;
    WsEvent::component_updated(
        ctx,
        component
            .into_frontend_type(
                ctx,
                None,
                component.change_status(ctx).await?,
                &mut HashMap::new(),
            )
            .await?,
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    Ok(())
}
