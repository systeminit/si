//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use itertools::Itertools;
use petgraph::Direction::Outgoing;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map, HashMap, HashSet, VecDeque};
use std::hash::Hash;
use std::num::ParseFloatError;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
use veritech_client::ResourceStatus;

use si_events::{ulid::Ulid, ContentHash};

use self::frame::{Frame, FrameError};
use self::resource::ResourceData;
use crate::action::prototype::{ActionKind, ActionPrototype, ActionPrototypeError};
use crate::action::{Action, ActionError, ActionState};
use crate::actor_view::ActorView;
use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::{AttributePrototypeError, AttributePrototypeSource};
use crate::attribute::value::{AttributeValueError, DependentValueGraph, ValueIsFor};
use crate::change_set::ChangeSetError;
use crate::code_view::CodeViewError;
use crate::diagram::{SummaryDiagramComponent, SummaryDiagramInferredEdge};
use crate::history_event::HistoryEventMetadata;
use crate::layer_db_types::{ComponentContent, ComponentContentV1};
use crate::prop::{PropError, PropPath};
use crate::qualification::QualificationError;
use crate::schema::variant::root_prop::component_type::ComponentType;
use crate::schema::variant::SchemaVariantError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::attribute_prototype_argument_node_weight::ArgumentTargets;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{ComponentNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::SocketArity;

use crate::{
    implement_add_edge_to, pk, AttributePrototype, AttributeValue, AttributeValueId, ChangeSetId,
    DalContext, Func, FuncError, FuncId, HelperError, InputSocket, InputSocketId, OutputSocket,
    OutputSocketId, Prop, PropId, PropKind, Schema, SchemaVariant, SchemaVariantId,
    StandardModelError, Timestamp, TransactionsError, UserPk, WorkspaceError, WorkspacePk, WsEvent,
    WsEventError, WsEventResult, WsPayload,
};

pub mod code;
pub mod debug;
pub mod diff;
pub mod frame;
pub mod properties;
pub mod qualification;
pub mod resource;

pub const DEFAULT_COMPONENT_X_POSITION: &str = "0";
pub const DEFAULT_COMPONENT_Y_POSITION: &str = "0";
pub const DEFAULT_COMPONENT_WIDTH: &str = "500";
pub const DEFAULT_COMPONENT_HEIGHT: &str = "500";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("action error: {0}")]
    Action(Box<ActionError>),
    #[error("action prototype error: {0}")]
    ActionPrototype(Box<ActionPrototypeError>),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("code view error: {0}")]
    CodeView(#[from] CodeViewError),
    #[error("component {0} has an unexpected schema variant id")]
    ComponentIncorrectSchemaVariant(ComponentId),
    #[error("component {0} has no attribute value for the root/si/color prop")]
    ComponentMissingColorValue(ComponentId),
    #[error("component {0} has no attribute value for the root/domain prop")]
    ComponentMissingDomainValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/name prop")]
    ComponentMissingNameValue(ComponentId),
    #[error("component {0} has no attribute value for the root/resource prop")]
    ComponentMissingResourceValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/type prop")]
    ComponentMissingTypeValue(ComponentId),
    #[error("component {0} has no materialized view for the root/si/type prop")]
    ComponentMissingTypeValueMaterializedView(ComponentId),
    #[error("connection destination component {0} has no attribute value for input socket {1}")]
    DestinationComponentMissingAttributeValueForInputSocket(ComponentId, InputSocketId),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("frame error: {0}")]
    Frame(#[from] Box<FrameError>),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("input socket {0} has more than one attribute value")]
    InputSocketTooManyAttributeValues(InputSocketId),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing attribute prototype argument source: {0}")]
    MissingAttributePrototypeArgumentSource(AttributePrototypeArgumentId),
    #[error("component {0} missing attribute value for code")]
    MissingCodeValue(ComponentId),
    #[error("missing controlling func data for parent attribute value id: {0}")]
    MissingControllingFuncDataForParentAttributeValue(AttributeValueId),
    #[error("missing path for attribute value: {0}")]
    MissingPathForAttributeValue(AttributeValueId),
    #[error("component {0} missing attribute value for qualifications")]
    MissingQualificationsValue(ComponentId),
    #[error("component {0} missing attribute value for root")]
    MissingRootProp(ComponentId),
    #[error("more than one schema variant found for component: {0}")]
    MoreThanOneSchemaVariantFound(ComponentId),
    #[error("found multiple parents for component: {0}")]
    MultipleParentsForComponent(ComponentId),
    #[error("found multiple root attribute values ({0} and {1}, at minimum) for component: {2}")]
    MultipleRootAttributeValuesFound(AttributeValueId, AttributeValueId, ComponentId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("component not found: {0}")]
    NotFound(ComponentId),
    #[error("object prop {0} has no ordering node")]
    ObjectPropHasNoOrderingNode(PropId),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("output socket has not found for attribute value id {0}")]
    OutputSocketNotFoundForAttributeValueId(AttributeValueId),
    #[error("output socket {0} has more than one attribute value")]
    OutputSocketTooManyAttributeValues(OutputSocketId),
    #[error("parse float error: {0}")]
    ParseFloat(#[from] ParseFloatError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("found prop id ({0}) that is not a prop")]
    PropIdNotAProp(PropId),
    #[error("qualification error: {0}")]
    Qualification(#[from] QualificationError),
    #[error("ordering node not found for qualifications map {0} and component {1}")]
    QualificationNoOrderingNode(AttributeValueId, ComponentId),
    #[error("resource attribute value not found for component: {0}")]
    ResourceAttributeValueNotFound(ComponentId),
    #[error("root attribute value not found for component: {0}")]
    RootAttributeValueNotFound(ComponentId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found for component: {0}")]
    SchemaVariantNotFound(ComponentId),
    #[error("serde_json error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace pk not found on context")]
    WorkspacePkNone,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("attribute value {0} has wrong type for operation: {0}")]
    WrongAttributeValueType(AttributeValueId, ValueIsFor),
    #[error("WsEvent error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ComponentResult<T> = Result<T, ComponentError>;

pk!(ComponentId);

impl From<ComponentId> for si_events::ComponentId {
    fn from(value: ComponentId) -> Self {
        value.into_inner().into()
    }
}

#[derive(Clone, Debug)]
pub struct IncomingConnection {
    pub attribute_prototype_argument_id: AttributePrototypeArgumentId,
    pub to_component_id: ComponentId,
    pub to_input_socket_id: InputSocketId,
    pub from_component_id: ComponentId,
    pub from_output_socket_id: OutputSocketId,
    pub created_info: HistoryEventMetadata,
    pub deleted_info: Option<HistoryEventMetadata>,
}

#[derive(Clone, Debug)]
pub struct OutgoingConnection {
    pub attribute_prototype_argument_id: AttributePrototypeArgumentId,
    pub to_component_id: ComponentId,
    pub to_input_socket_id: InputSocketId,
    pub from_component_id: ComponentId,
    pub from_output_socket_id: OutputSocketId,
    pub created_info: HistoryEventMetadata,
    pub deleted_info: Option<HistoryEventMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InferredIncomingConnection {
    pub to_component_id: ComponentId,
    pub to_input_socket_id: InputSocketId,
    pub from_component_id: ComponentId,
    pub from_output_socket_id: OutputSocketId,
    pub to_delete: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct InputSocketMatch {
    pub component_id: ComponentId,
    pub input_socket_id: InputSocketId,
    pub attribute_value_id: AttributeValueId,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct OutputSocketMatch {
    pub component_id: ComponentId,
    pub output_socket_id: OutputSocketId,
    pub attribute_value_id: AttributeValueId,
}

/// A [`Component`] is an instantiation of a [`SchemaVariant`](crate::SchemaVariant).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Component {
    id: ComponentId,
    #[serde(flatten)]
    timestamp: Timestamp,
    to_delete: bool,
    x: String,
    y: String,
    width: Option<String>,
    height: Option<String>,
}

impl From<Component> for ComponentContentV1 {
    fn from(value: Component) -> Self {
        Self {
            timestamp: value.timestamp,
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Copy, Clone)]
pub struct ControllingFuncData {
    pub func_id: FuncId,
    pub av_id: AttributeValueId,
    pub is_dynamic_func: bool,
}

impl Component {
    pub fn assemble(node_weight: &ComponentNodeWeight, content: ComponentContentV1) -> Self {
        Self {
            id: node_weight.id().into(),
            timestamp: content.timestamp,
            to_delete: node_weight.to_delete(),
            x: content.x,
            y: content.y,
            width: content.width,
            height: content.height,
        }
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn x(&self) -> &str {
        &self.x
    }

    pub fn y(&self) -> &str {
        &self.y
    }

    pub fn width(&self) -> Option<&str> {
        self.width.as_deref()
    }

    pub fn height(&self) -> Option<&str> {
        self.height.as_deref()
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub async fn view(&self, ctx: &DalContext) -> ComponentResult<Option<serde_json::Value>> {
        let schema_variant_id = Self::schema_variant_id(ctx, self.id()).await?;
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(["root"])).await?;

        let root_value_ids = Prop::attribute_values_for_prop_id(ctx, root_prop_id).await?;
        for value_id in root_value_ids {
            let value_component_id = AttributeValue::component_id(ctx, value_id).await?;
            if value_component_id == self.id() {
                let root_value = AttributeValue::get_by_id(ctx, value_id).await?;
                return Ok(root_value.view(ctx).await?);
            }
        }

        // Should this be an error?
        Ok(None)
    }

    implement_add_edge_to!(
        source_id: ComponentId,
        destination_id: SchemaVariantId,
        add_fn: add_edge_to_schema_variant,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ComponentResult,
    );
    implement_add_edge_to!(
        source_id: ComponentId,
        destination_id: ComponentId,
        add_fn: add_edge_to_frame,
        discriminant: EdgeWeightKindDiscriminants::FrameContains,
        result: ComponentResult,
    );
    implement_add_edge_to!(
        source_id: ComponentId,
        destination_id: AttributeValueId,
        add_fn: add_edge_to_root_attribute_value,
        discriminant: EdgeWeightKindDiscriminants::Root,
        result: ComponentResult,
    );
    implement_add_edge_to!(
        source_id: ComponentId,
        destination_id: AttributeValueId,
        add_fn: add_edge_to_socket_attribute_value,
        discriminant: EdgeWeightKindDiscriminants::SocketValue,
        result: ComponentResult,
    );
    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: ComponentId,
        add_fn: add_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: ComponentResult,
    );

    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<Self> {
        let name: String = name.into();

        let content = ComponentContentV1 {
            timestamp: Timestamp::now(),
            x: DEFAULT_COMPONENT_X_POSITION.to_string(),
            y: DEFAULT_COMPONENT_Y_POSITION.to_string(),
            width: None,
            height: None,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ComponentContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_component(change_set, id, hash)?;

        // Attach component to category and add use edge to schema variant
        let workspace_snapshot = ctx.workspace_snapshot()?;
        workspace_snapshot.add_node(node_weight).await?;

        // Root --> Component Category --> Component (this)
        let component_category_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Component)
            .await?;
        Self::add_category_edge(
            ctx,
            component_category_id,
            id.into(),
            EdgeWeightKind::new_use(),
        )
        .await?;

        let mut attribute_values = vec![];

        // Create attribute values for all socket corresponding to input and output sockets.
        for input_socket_id in
            InputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, input_socket_id, Some(id.into()), None, None).await?;

            attribute_values.push(attribute_value.id());
        }
        for output_socket_id in
            OutputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, output_socket_id, Some(id.into()), None, None).await?;

            attribute_values.push(attribute_value.id());
        }

        // Walk all the props for the schema variant and create attribute values for all of them
        let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;
        let mut work_queue = VecDeque::from([(root_prop_id, None::<AttributeValueId>, None)]);

        while let Some((prop_id, maybe_parent_attribute_value_id, key)) = work_queue.pop_front() {
            // If we came in with a key, we're the child of a map. We should not descend deeper
            // into it because the value should be governed by its prototype function and will
            // create child values when that function is executed
            let should_descend = key.is_none();

            // Ensure that we are processing a prop before creating attribute values. Cache the
            // prop kind for later.
            let prop_kind = workspace_snapshot
                .get_node_weight_by_id(prop_id)
                .await?
                .get_prop_node_weight()?
                .kind();

            // Create an attribute value for the prop.
            let attribute_value = AttributeValue::new(
                ctx,
                prop_id,
                Some(id.into()),
                maybe_parent_attribute_value_id,
                key,
            )
            .await?;

            attribute_values.push(attribute_value.id());
            ctx.enqueue_compute_validations(attribute_value.id())
                .await?;

            if should_descend {
                match prop_kind {
                    PropKind::Object => {
                        let ordering_node_weight = workspace_snapshot
                            .ordering_node_for_container(prop_id)
                            .await?
                            .ok_or(ComponentError::ObjectPropHasNoOrderingNode(prop_id))?;

                        for &child_prop_id in ordering_node_weight.order() {
                            work_queue.push_back((
                                child_prop_id.into(),
                                Some(attribute_value.id()),
                                None,
                            ));
                        }
                    }
                    PropKind::Map => {
                        let element_prop_id =
                            Prop::direct_single_child_prop_id(ctx, prop_id).await?;

                        for (key, _) in Prop::prototypes_by_key(ctx, element_prop_id).await? {
                            if key.is_some() {
                                work_queue.push_back((
                                    element_prop_id,
                                    Some(attribute_value.id()),
                                    key,
                                ))
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let (node_weight, content) = Self::get_node_weight_and_content(ctx, id.into()).await?;
        let component = Self::assemble(&node_weight, content);

        // Component (this) --> Schema Variant
        Component::add_edge_to_schema_variant(
            ctx,
            component.id,
            schema_variant_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        component.set_name(ctx, &name).await?;

        let component_graph = DependentValueGraph::new(ctx, attribute_values).await?;
        let leaf_value_ids = component_graph.independent_values();
        ctx.add_dependent_values_and_enqueue(leaf_value_ids).await?;

        // Find all create action prototypes for the variant and create actions for them.
        for prototype_id in SchemaVariant::find_action_prototypes_by_kind(
            ctx,
            schema_variant_id,
            ActionKind::Create,
        )
        .await?
        {
            Action::new(ctx, prototype_id, Some(component.id))
                .await
                .map_err(|err| ComponentError::Action(Box::new(err)))?;
        }

        WsEvent::component_created(ctx, component.id())
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(component)
    }

    pub async fn clone_attributes_from(
        &self,
        ctx: &DalContext,
        copied_component_id: ComponentId,
        reset_resource: bool,
        reset_name: bool,
    ) -> ComponentResult<()> {
        let copied_root_id = Component::root_attribute_value_id(ctx, copied_component_id).await?;
        let pasted_root_id = Component::root_attribute_value_id(ctx, self.id).await?;

        // Paste attribute value "values" from original component (or create them for maps/arrays)
        //
        // We could make this more efficient by skipping everything set by non builtins (si:setString, si:setObject, etc), since everything that is propagated will be re-propagated
        let mut work_queue: VecDeque<(AttributeValueId, AttributeValueId)> =
            vec![(copied_root_id, pasted_root_id)].into_iter().collect();
        while let Some((copied_av_id, pasted_av_id)) = work_queue.pop_front() {
            if let Some(prop_id) = AttributeValue::prop_id_for_id(ctx, copied_av_id).await? {
                let prop = Prop::get_by_id_or_error(ctx, prop_id).await?;
                if prop.kind != PropKind::Object
                    && prop.kind != PropKind::Map
                    && prop.kind != PropKind::Array
                {
                    let copied_av = AttributeValue::get_by_id(ctx, copied_av_id).await?;
                    let value = copied_av.value(ctx).await?;
                    AttributeValue::update(ctx, pasted_av_id, value).await?;
                }
            }

            // Enqueue children
            let copied_children = AttributeValue::list_all_children(ctx, copied_av_id).await?;
            let pasted_children = AttributeValue::list_all_children(ctx, pasted_av_id).await?;
            let mut pasted_children_paths = HashMap::new();

            for pasted_child_av_id in &pasted_children {
                let pasted_path = AttributeValue::get_path_for_id(ctx, *pasted_child_av_id)
                    .await?
                    .ok_or(ComponentError::MissingPathForAttributeValue(
                        *pasted_child_av_id,
                    ))?;
                pasted_children_paths.insert(pasted_path, *pasted_child_av_id);
            }

            for copied_child_av_id in copied_children {
                let copied_path = AttributeValue::get_path_for_id(ctx, copied_child_av_id)
                    .await?
                    .ok_or(ComponentError::MissingPathForAttributeValue(
                        copied_child_av_id,
                    ))?;

                let pasted_child_av_id = if let Some(pasted_child_av_id) =
                    pasted_children_paths.get(&copied_path).copied()
                {
                    pasted_child_av_id
                } else {
                    AttributeValue::new(
                        ctx,
                        AttributeValue::is_for(ctx, copied_child_av_id).await?,
                        Some(self.id),
                        Some(pasted_av_id),
                        AttributeValue::key_for_id(ctx, copied_child_av_id).await?,
                    )
                    .await?
                    .id
                };
                work_queue.push_back((copied_child_av_id, pasted_child_av_id));
            }
        }

        if reset_resource {
            self.set_resource(ctx, ResourceData::new(ResourceStatus::Ok, None))
                .await?;
        }
        if reset_name {
            self.set_name(ctx, &format!("{} - Copy", self.name(ctx).await?))
                .await?;
        }

        let copied_root_id = Component::root_attribute_value_id(ctx, copied_component_id).await?;
        let pasted_root_id = Component::root_attribute_value_id(ctx, self.id).await?;
        let mut work_queue: VecDeque<(AttributeValueId, AttributeValueId)> =
            vec![(copied_root_id, pasted_root_id)].into_iter().collect();

        // Paste attribute prototypes
        // - either updates component prototype to a copy of the original component
        // - or removes component prototype, restoring the schema one (needed because of manual update from the block above)
        while let Some((copied_av_id, pasted_av_id)) = work_queue.pop_front() {
            if let Some(copied_prototype_id) =
                AttributeValue::component_prototype_id(ctx, copied_av_id).await?
            {
                let func_id = AttributePrototype::func_id(ctx, copied_prototype_id).await?;
                let prototype = AttributePrototype::new(ctx, func_id).await?;

                for copied_apa_id in
                    AttributePrototypeArgument::list_ids_for_prototype(ctx, copied_prototype_id)
                        .await?
                {
                    let func_arg_id =
                        AttributePrototypeArgument::func_argument_id_by_id(ctx, copied_apa_id)
                            .await?;
                    let value_source =
                        AttributePrototypeArgument::value_source_by_id(ctx, copied_apa_id)
                            .await?
                            .ok_or(ComponentError::MissingAttributePrototypeArgumentSource(
                                copied_apa_id,
                            ))?;

                    let apa =
                        AttributePrototypeArgument::new(ctx, prototype.id(), func_arg_id).await?;
                    match value_source {
                        ValueSource::InputSocket(socket_id) => {
                            apa.set_value_from_input_socket_id(ctx, socket_id).await?;
                        }
                        ValueSource::OutputSocket(socket_id) => {
                            apa.set_value_from_output_socket_id(ctx, socket_id).await?;
                        }
                        ValueSource::Prop(prop_id) => {
                            apa.set_value_from_prop_id(ctx, prop_id).await?;
                        }
                        ValueSource::Secret(secret_id) => {
                            apa.set_value_from_secret_id(ctx, secret_id).await?;
                        }
                        ValueSource::StaticArgumentValue(id) => {
                            apa.set_value_from_static_value_id(ctx, id).await?;
                        }
                    }
                }

                AttributeValue::set_component_prototype_id(ctx, pasted_av_id, prototype.id).await?;

                let sources = AttributePrototype::input_sources(ctx, prototype.id).await?;
                for source in sources {
                    match source {
                        AttributePrototypeSource::AttributeValue(_, _) => {
                            continue;
                        }
                        AttributePrototypeSource::Prop(prop_id, key) => {
                            Prop::add_edge_to_attribute_prototype(
                                ctx,
                                prop_id,
                                prototype.id,
                                EdgeWeightKind::Prototype(key),
                            )
                            .await?;
                        }
                        AttributePrototypeSource::InputSocket(socket_id, key) => {
                            InputSocket::add_edge_to_attribute_prototype(
                                ctx,
                                socket_id,
                                prototype.id,
                                EdgeWeightKind::Prototype(key),
                            )
                            .await?;
                        }
                        AttributePrototypeSource::OutputSocket(socket_id, key) => {
                            OutputSocket::add_edge_to_attribute_prototype(
                                ctx,
                                socket_id,
                                prototype.id,
                                EdgeWeightKind::Prototype(key),
                            )
                            .await?;
                        }
                    }
                }
            } else if let Some(existing_prototype_id) =
                AttributeValue::component_prototype_id(ctx, pasted_av_id).await?
            {
                AttributePrototype::remove(ctx, existing_prototype_id).await?;
            }

            // Enqueue children
            let copied_children = AttributeValue::list_all_children(ctx, copied_av_id).await?;
            let pasted_children = AttributeValue::list_all_children(ctx, pasted_av_id).await?;
            let mut pasted_children_paths = HashMap::new();

            for pasted_child_av_id in &pasted_children {
                let pasted_path = AttributeValue::get_path_for_id(ctx, *pasted_child_av_id)
                    .await?
                    .ok_or(ComponentError::MissingPathForAttributeValue(
                        *pasted_child_av_id,
                    ))?;
                pasted_children_paths.insert(pasted_path, *pasted_child_av_id);
            }

            for copied_child_av_id in copied_children {
                let copied_path = AttributeValue::get_path_for_id(ctx, copied_child_av_id)
                    .await?
                    .ok_or(ComponentError::MissingPathForAttributeValue(
                        copied_child_av_id,
                    ))?;

                let pasted_child_av_id = if let Some(pasted_child_av_id) =
                    pasted_children_paths.get(&copied_path).copied()
                {
                    pasted_child_av_id
                } else {
                    AttributeValue::new(
                        ctx,
                        AttributeValue::is_for(ctx, copied_child_av_id).await?,
                        Some(self.id),
                        Some(pasted_av_id),
                        AttributeValue::key_for_id(ctx, copied_child_av_id).await?,
                    )
                    .await?
                    .id
                };
                work_queue.push_back((copied_child_av_id, pasted_child_av_id));
            }
        }

        Ok(())
    }

    pub async fn outgoing_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<OutgoingConnection>> {
        let mut outgoing_edges = vec![];

        for (from_output_socket_id, _) in self.output_socket_attribute_values(ctx).await? {
            for apa_id in AttributePrototypeArgument::list_ids_for_output_socket_and_source(
                ctx,
                from_output_socket_id,
                self.id,
            )
            .await?
            {
                let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;

                let created_info = {
                    let history_actor = ctx.history_actor();
                    let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
                    HistoryEventMetadata {
                        actor,
                        timestamp: apa.timestamp().created_at,
                    }
                };

                if let Some(ArgumentTargets {
                    source_component_id,
                    destination_component_id,
                }) = apa.targets()
                {
                    let prototype_id = apa.prototype_id(ctx).await?;
                    let input_sources =
                        AttributePrototype::input_sources(ctx, prototype_id).await?;
                    if input_sources.len() > 1 {
                        debug!("More than 1 source for an attribute prototype");
                    }

                    if let Some(AttributePrototypeSource::InputSocket(input_socket, _)) =
                        input_sources.first()
                    {
                        outgoing_edges.push(OutgoingConnection {
                            attribute_prototype_argument_id: apa_id,
                            to_component_id: destination_component_id,
                            from_component_id: source_component_id,
                            to_input_socket_id: *input_socket,
                            from_output_socket_id,
                            created_info,
                            deleted_info: None,
                        });
                    }
                }
            }
        }

        Ok(outgoing_edges)
    }

    pub async fn incoming_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<IncomingConnection>> {
        let mut incoming_edges = vec![];

        for (to_input_socket_id, to_value_id) in self.input_socket_attribute_values(ctx).await? {
            let prototype_id =
                AttributeValue::prototype_id(ctx, to_value_id.attribute_value_id).await?;
            for apa_id in AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                ctx,
                prototype_id,
                self.id,
            )
            .await?
            {
                let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;

                let created_info = {
                    let history_actor = ctx.history_actor();
                    let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
                    HistoryEventMetadata {
                        actor,
                        timestamp: apa.timestamp().created_at,
                    }
                };

                if let Some(ArgumentTargets {
                    source_component_id,
                    destination_component_id,
                }) = apa.targets()
                {
                    if let Some(ValueSource::OutputSocket(from_output_socket_id)) =
                        apa.value_source(ctx).await?
                    {
                        incoming_edges.push(IncomingConnection {
                            attribute_prototype_argument_id: apa_id,
                            to_component_id: destination_component_id,
                            from_component_id: source_component_id,
                            to_input_socket_id,
                            from_output_socket_id,
                            created_info,
                            deleted_info: None,
                        });
                    }
                }
            }
        }

        Ok(incoming_edges)
    }
    pub async fn get_children_for_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        let mut children: Vec<ComponentId> = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;
        for children_target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?
        {
            children.push(
                workspace_snapshot
                    .get_node_weight(children_target)
                    .await?
                    .id()
                    .into(),
            );
        }

        Ok(children)
    }
    #[instrument(level = "debug" skip(ctx))]
    pub async fn get_parent_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<ComponentId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut raw_sources = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?;

        let maybe_parent = if let Some(raw_parent) = raw_sources.pop() {
            if !raw_sources.is_empty() {
                return Err(ComponentError::MultipleParentsForComponent(component_id));
            }
            Some(
                workspace_snapshot
                    .get_node_weight(raw_parent)
                    .await?
                    .id()
                    .into(),
            )
        } else {
            None
        };
        Ok(maybe_parent)
    }
    pub async fn parent(&self, ctx: &DalContext) -> ComponentResult<Option<ComponentId>> {
        Self::get_parent_by_id(ctx, self.id).await
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ComponentNodeWeight, ComponentContentV1)> {
        let (component_node_weight, hash) =
            Self::get_node_weight_and_content_hash(ctx, component_id).await?;

        let content: ComponentContent = ctx.layer_db().cas().try_read_as(&hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(component_id.into()),
        )?;

        let ComponentContent::V1(inner) = content;

        Ok((component_node_weight, inner))
    }

    async fn get_node_weight_and_content_hash(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ComponentNodeWeight, ContentHash)> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id: Ulid = component_id.into();
        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;

        let hash = node_weight.content_hash();
        let component_node_weight = node_weight.get_component_node_weight()?;
        Ok((component_node_weight, hash))
    }

    pub async fn list(ctx: &DalContext) -> ComponentResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut components = vec![];
        let component_category_node_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Component)
            .await?;

        let component_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut hashes = vec![];
        for index in component_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_component_node_weight()?;
            hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let contents: HashMap<ContentHash, ComponentContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match contents.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let ComponentContent::V1(inner) = content;

                    components.push(Self::assemble(&node_weight, inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(components)
    }

    pub async fn schema_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Schema> {
        let schema_variant = Self::schema_variant_for_component_id(ctx, component_id).await?;

        Ok(schema_variant.schema(ctx).await?)
    }

    pub async fn schema(&self, ctx: &DalContext) -> ComponentResult<Schema> {
        Self::schema_for_component_id(ctx, self.id).await
    }

    pub async fn schema_variant_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariant> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;
        Ok(SchemaVariant::get_by_id(ctx, schema_variant_id).await?)
    }

    pub async fn schema_variant(&self, ctx: &DalContext) -> ComponentResult<SchemaVariant> {
        Self::schema_variant_for_component_id(ctx, self.id).await
    }

    pub async fn schema_variant_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaVariantId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let maybe_schema_variant_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(component_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        let mut schema_variant_id: Option<SchemaVariantId> = None;
        for maybe_schema_variant_index in maybe_schema_variant_indices {
            if let NodeWeight::Content(content) = workspace_snapshot
                .get_node_weight(maybe_schema_variant_index)
                .await?
            {
                let content_hash_discriminants: ContentAddressDiscriminants =
                    content.content_address().into();
                if let ContentAddressDiscriminants::SchemaVariant = content_hash_discriminants {
                    schema_variant_id = match schema_variant_id {
                        None => Some(content.id().into()),
                        Some(_already_found_schema_variant_id) => {
                            return Err(ComponentError::MoreThanOneSchemaVariantFound(
                                component_id,
                            ));
                        }
                    };
                }
            }
        }
        let schema_variant_id =
            schema_variant_id.ok_or(ComponentError::SchemaVariantNotFound(component_id))?;
        Ok(schema_variant_id)
    }

    pub async fn get_by_id(ctx: &DalContext, component_id: ComponentId) -> ComponentResult<Self> {
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, component_id).await?;
        Ok(Self::assemble(&node_weight, content))
    }

    pub async fn set_geometry(
        &mut self,
        ctx: &DalContext,
        x: impl Into<String>,
        y: impl Into<String>,
        width: Option<impl Into<String>>,
        height: Option<impl Into<String>>,
    ) -> ComponentResult<Self> {
        let id: ComponentId = self.id;

        let before = ComponentContentV1::from(self.clone());
        self.x = x.into();
        self.y = y.into();
        self.width = width.map(|w| w.into());
        self.height = height.map(|h| h.into());
        let updated = ComponentContentV1::from(self.clone());

        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(ComponentContent::V1(updated).into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;

            ctx.workspace_snapshot()?
                .update_content(ctx.change_set()?, id.into(), hash)
                .await?;
        }
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, id).await?;

        Ok(Self::assemble(&node_weight, content))
    }

    // Set the name of the component. Should only be used during component creation
    async fn set_name(&self, ctx: &DalContext, name: &str) -> ComponentResult<()> {
        let path = ["root", "si", "name"];
        let sv_id = Self::schema_variant_id(ctx, self.id).await?;
        let name_prop_id = Prop::find_prop_id_by_path(ctx, sv_id, &PropPath::new(path)).await?;
        // If the name prop is controlled by an identity or other function,
        // don't override the prototype here
        if Prop::is_set_by_dependent_function(ctx, name_prop_id).await? {
            return Ok(());
        }

        let av_for_name = self
            .attribute_values_for_prop(ctx, &path)
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingNameValue(self.id()))?;

        AttributeValue::update(ctx, av_for_name, Some(serde_json::to_value(name)?)).await?;

        Ok(())
    }

    pub async fn set_resource(
        &self,
        ctx: &DalContext,
        resource: ResourceData,
    ) -> ComponentResult<()> {
        let av_for_resource = self
            .attribute_values_for_prop(ctx, &["root", "resource"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingResourceValue(self.id()))?;

        AttributeValue::update(ctx, av_for_resource, Some(serde_json::to_value(resource)?)).await?;

        Ok(())
    }

    pub async fn clear_resource(&self, ctx: &DalContext) -> ComponentResult<()> {
        let av_for_resource = self
            .attribute_values_for_prop(ctx, &["root", "resource"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingResourceValue(self.id()))?;

        AttributeValue::update(ctx, av_for_resource, Some(serde_json::json!({}))).await?;

        Ok(())
    }

    pub async fn resource(&self, ctx: &DalContext) -> ComponentResult<Option<ResourceData>> {
        let value_id = self
            .attribute_values_for_prop(ctx, &["root", "resource"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingResourceValue(self.id()))?;

        let av = AttributeValue::get_by_id(ctx, value_id).await?;

        match av.view(ctx).await? {
            Some(serde_value) => {
                if serde_value.is_object()
                    && serde_value
                        .as_object()
                        .expect("we just checked if its an object")
                        .is_empty()
                {
                    Ok(None)
                } else {
                    Ok(Some(serde_json::from_value(serde_value)?))
                }
            }
            None => Ok(None),
        }
    }

    pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
        let name_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "name"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingNameValue(self.id()))?;

        let name_av = AttributeValue::get_by_id(ctx, name_value_id).await?;

        Ok(match name_av.view(ctx).await? {
            Some(serde_value) => serde_json::from_value(serde_value)?,
            None => "".into(),
        })
    }

    pub async fn color(&self, ctx: &DalContext) -> ComponentResult<Option<String>> {
        let color_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "color"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingColorValue(self.id()))?;

        let color_av = AttributeValue::get_by_id(ctx, color_value_id).await?;

        Ok(match color_av.view(ctx).await? {
            Some(serde_value) => Some(serde_json::from_value(serde_value)?),
            None => None,
        })
    }
    #[instrument(level="debug" skip(ctx))]
    pub async fn get_type_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<ComponentType> {
        let type_value_id =
            Self::attribute_values_for_prop_by_id(ctx, component_id, &["root", "si", "type"])
                .await?
                .into_iter()
                .next()
                .ok_or(ComponentError::ComponentMissingTypeValue(component_id))?;
        let type_value = AttributeValue::get_by_id(ctx, type_value_id)
            .await?
            .view(ctx)
            .await?
            .ok_or(ComponentError::ComponentMissingTypeValueMaterializedView(
                component_id,
            ))?;

        Ok(serde_json::from_value(type_value)?)
    }

    pub async fn get_type(&self, ctx: &DalContext) -> ComponentResult<ComponentType> {
        Self::get_type_by_id(ctx, self.id()).await
    }

    pub async fn set_type(&self, ctx: &DalContext, new_type: ComponentType) -> ComponentResult<()> {
        let type_value_id = self
            .attribute_values_for_prop(ctx, &["root", "si", "type"])
            .await?
            .into_iter()
            .next()
            .ok_or(ComponentError::ComponentMissingTypeValue(self.id()))?;

        let value = serde_json::to_value(new_type)?;

        AttributeValue::update(ctx, type_value_id, Some(value)).await?;

        Ok(())
    }

    pub async fn root_attribute_value_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<AttributeValueId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut maybe_root_attribute_value_id = None;
        for target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(component_id, EdgeWeightKindDiscriminants::Root)
            .await?
        {
            let target_node_weight = workspace_snapshot.get_node_weight(target).await?;
            if let NodeWeight::AttributeValue(_) = target_node_weight {
                maybe_root_attribute_value_id = match maybe_root_attribute_value_id {
                    Some(already_found_root_attribute_value_id) => {
                        return Err(ComponentError::MultipleRootAttributeValuesFound(
                            target_node_weight.id().into(),
                            already_found_root_attribute_value_id,
                            component_id,
                        ));
                    }
                    None => Some(target_node_weight.id().into()),
                };
            }
        }
        maybe_root_attribute_value_id
            .ok_or(ComponentError::RootAttributeValueNotFound(component_id))
    }

    pub async fn output_socket_attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<OutputSocketId, OutputSocketMatch>> {
        let mut result = HashMap::new();

        let socket_values = Self::values_for_all_sockets(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(output_socket_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .output_socket_id()
            {
                match result.entry(output_socket_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(OutputSocketMatch {
                            component_id,
                            attribute_value_id: socket_value_id,
                            output_socket_id,
                        });
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::OutputSocketTooManyAttributeValues(
                            output_socket_id,
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn output_socket_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<OutputSocketId, OutputSocketMatch>> {
        Self::output_socket_attribute_values_for_component_id(ctx, self.id()).await
    }

    pub async fn attribute_values_for_prop(
        &self,
        ctx: &DalContext,
        prop_path: &[&str],
    ) -> ComponentResult<Vec<AttributeValueId>> {
        Self::attribute_values_for_prop_by_id(ctx, self.id(), prop_path).await
    }

    pub async fn attribute_values_for_prop_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
        prop_path: &[&str],
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut result = vec![];

        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;

        let prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(prop_path)).await?;

        for attribute_value_id in Prop::attribute_values_for_prop_id(ctx, prop_id).await? {
            let value_component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
            if value_component_id == component_id {
                result.push(attribute_value_id)
            }
        }
        Ok(result)
    }

    pub async fn domain_prop_attribute_value(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<AttributeValueId> {
        self.attribute_values_for_prop(ctx, &["root", "domain"])
            .await?
            .first()
            .cloned()
            .ok_or(ComponentError::ComponentMissingDomainValue(self.id))
    }

    async fn values_for_all_sockets(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut socket_values: Vec<AttributeValueId> = vec![];
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for socket_target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::SocketValue,
            )
            .await?
        {
            socket_values.push(
                workspace_snapshot
                    .get_node_weight(socket_target)
                    .await?
                    .get_attribute_value_node_weight()?
                    .id()
                    .into(),
            );
        }

        Ok(socket_values)
    }
    #[instrument(level="debug" skip_all)]
    pub async fn input_socket_match(
        ctx: &DalContext,
        component_id: ComponentId,
        input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<InputSocketMatch>> {
        let all_input_sockets =
            Self::input_socket_attribute_values_for_component_id(ctx, component_id).await?;
        Ok(all_input_sockets.get(&input_socket_id).cloned())
    }
    pub async fn output_socket_match(
        ctx: &DalContext,
        component_id: ComponentId,
        output_socket_id: OutputSocketId,
    ) -> ComponentResult<Option<OutputSocketMatch>> {
        let all_output_sockets =
            Self::output_socket_attribute_values_for_component_id(ctx, component_id).await?;
        Ok(all_output_sockets.get(&output_socket_id).copied())
    }

    pub async fn input_socket_attribute_values_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<InputSocketId, InputSocketMatch>> {
        let mut result = HashMap::new();

        let socket_values = Self::values_for_all_sockets(ctx, component_id).await?;

        for socket_value_id in socket_values {
            if let Some(input_socket_id) = AttributeValue::is_for(ctx, socket_value_id)
                .await?
                .input_socket_id()
            {
                match result.entry(input_socket_id) {
                    hash_map::Entry::Vacant(entry) => {
                        entry.insert(InputSocketMatch {
                            component_id,
                            attribute_value_id: socket_value_id,
                            input_socket_id,
                        });
                    }
                    hash_map::Entry::Occupied(_) => {
                        return Err(ComponentError::InputSocketTooManyAttributeValues(
                            input_socket_id,
                        ));
                    }
                }
            }
        }

        Ok(result)
    }

    pub async fn input_socket_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<InputSocketId, InputSocketMatch>> {
        Self::input_socket_attribute_values_for_component_id(ctx, self.id()).await
    }
    #[instrument(level = "info", skip(ctx))]
    async fn connect_inner(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<(AttributeValueId, AttributePrototypeArgumentId)>> {
        let total_start = std::time::Instant::now();

        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;

        let destination_component = Component::get_by_id(ctx, destination_component_id).await?;
        for connection in destination_component.incoming_connections(ctx).await? {
            if connection.from_component_id == source_component_id
                && connection.from_output_socket_id == source_output_socket_id
                && connection.to_component_id == destination_component_id
                && connection.to_input_socket_id == destination_input_socket_id
            {
                return Ok(None);
            }
        }

        let destination_attribute_value_ids =
            InputSocket::attribute_values_for_input_socket_id(ctx, destination_input_socket_id)
                .await?;

        // filter the value ids by destination_component_id
        let mut destination_attribute_value_id: Option<AttributeValueId> = None;
        for value_id in destination_attribute_value_ids {
            let component_id = AttributeValue::component_id(ctx, value_id).await?;
            if component_id == destination_component_id {
                destination_attribute_value_id = Some(value_id);
                break;
            }
        }

        let destination_attribute_value_id = destination_attribute_value_id.ok_or(
            ComponentError::DestinationComponentMissingAttributeValueForInputSocket(
                destination_component_id,
                destination_input_socket_id,
            ),
        )?;

        let destination_prototype_id =
            AttributeValue::prototype_id(ctx, destination_attribute_value_id).await?;

        // check for socket arity on the input socket
        // if the input socket has arity of one, and there's an existing edge, need to remove it before adding the new one
        let input_socket = InputSocket::get_by_id(ctx, destination_input_socket_id).await?;
        if input_socket.arity() == SocketArity::One {
            let existing_attribute_prototype_args =
                AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                    ctx,
                    destination_prototype_id,
                    destination_component_id,
                )
                .await?;
            if !existing_attribute_prototype_args.is_empty() {
                for attribute_prototype_argument_id in existing_attribute_prototype_args {
                    let attribute_prototype_argument =
                        AttributePrototypeArgument::get_by_id(ctx, attribute_prototype_argument_id)
                            .await?;
                    if let Some(targets) = attribute_prototype_argument.targets() {
                        if targets.destination_component_id == destination_component_id {
                            debug!(
                                "Removing existing prototype as we are trying to connect a new one"
                            );
                            AttributePrototypeArgument::remove(
                                ctx,
                                attribute_prototype_argument_id,
                            )
                            .await?;
                        }
                    }
                }
            }
        }

        let attribute_prototype_argument = AttributePrototypeArgument::new_inter_component(
            ctx,
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_prototype_id,
        )
        .await?;

        drop(cycle_check_guard);
        info!(
            "Cycle Check Guard dropped, add edge took {:?}",
            total_start.elapsed()
        );

        Ok(Some((
            destination_attribute_value_id,
            attribute_prototype_argument.id(),
        )))
    }
    pub async fn remove_edge_from_frame(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> ComponentResult<()> {
        ctx.workspace_snapshot()?
            .remove_edge_for_ulids(
                ctx.change_set()?,
                parent_id,
                child_id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?;

        Ok(())
    }

    pub async fn connect(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<AttributePrototypeArgumentId>> {
        let maybe = Self::connect_inner(
            ctx,
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        )
        .await?;

        if let Some((destination_attribute_value_id, attribute_prototype_argument_id)) = maybe {
            ctx.add_dependent_values_and_enqueue(vec![destination_attribute_value_id])
                .await?;

            Ok(Some(attribute_prototype_argument_id))
        } else {
            Ok(None)
        }
    }

    // Returns map of node id -> parent node ids
    pub async fn build_graph(
        ctx: &DalContext,
    ) -> ComponentResult<HashMap<ComponentId, HashSet<ComponentId>>> {
        let total_start = std::time::Instant::now();

        let components = Self::list(ctx).await?;

        let mut components_map: HashMap<ComponentId, HashSet<ComponentId>> = HashMap::new();

        for component in components {
            components_map.insert(component.id, HashSet::new());

            for incoming_connection in component.incoming_connections(ctx).await? {
                components_map
                    .entry(component.id)
                    .or_default()
                    .insert(incoming_connection.from_component_id);
            }
            for inferred_incoming_connections in
                component.inferred_incoming_connections(ctx).await?
            {
                components_map
                    .entry(component.id)
                    .or_default()
                    .insert(inferred_incoming_connections.from_component_id);
            }
        }

        debug!("build graph took {:?}", total_start.elapsed());
        Ok(components_map)
    }

    pub async fn list_av_controlling_func_ids_for_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<AttributeValueId, ControllingFuncData>> {
        let root_av_id: AttributeValueId =
            Component::root_attribute_value_id(ctx, component_id).await?;

        let mut av_queue = VecDeque::from([(root_av_id, None)]);
        let mut result: HashMap<AttributeValueId, ControllingFuncData> = HashMap::new();

        while let Some((av_id, maybe_parent_av_id)) = av_queue.pop_front() {
            let prototype_id = AttributeValue::prototype_id(ctx, av_id).await?;
            let func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
            let func = Func::get_by_id_or_error(ctx, func_id).await?;

            let this_tuple = ControllingFuncData {
                func_id,
                av_id,
                is_dynamic_func: func.is_dynamic(),
            };

            // if av has a parent and parent is controlled by dynamic func, that's the controller
            // else av controls itself
            let controlling_tuple = if let Some(parent_av_id) = maybe_parent_av_id {
                let parent_controlling_data = *result.get(&parent_av_id).ok_or(
                    ComponentError::MissingControllingFuncDataForParentAttributeValue(parent_av_id),
                )?;

                if parent_controlling_data.is_dynamic_func {
                    parent_controlling_data
                } else {
                    this_tuple
                }
            } else {
                this_tuple
            };

            // {
            //     let prop_id = AttributeValue::prop_id_for_id(ctx, av_id).await?;
            //     let this_prop = Prop::get_by_id(ctx, prop_id).await?;
            //
            //     let controlling_prop = {
            //         let prop_id =
            //             AttributeValue::prop_id_for_id(ctx, controlling_tuple.av_id).await?;
            //         Prop::get_by_id(ctx, prop_id).await?
            //     };
            //     let controlling_func = Func::get_by_id(ctx, controlling_tuple.func_id).await?;
            //
            //     let controlled_by_ancestor = controlling_tuple.av_id != this_tuple.av_id;
            //     println!("===========================");
            //
            //     println!(
            //         "Prop {} is controlled by {}, through func {}({}dynamic){}",
            //         this_prop.name,
            //         if controlled_by_ancestor {
            //             controlling_prop.name
            //         } else {
            //             "itself".to_string()
            //         },
            //         controlling_func.name,
            //         if controlling_tuple.is_dynamic_func {
            //             ""
            //         } else {
            //             "non-"
            //         },
            //         if controlled_by_ancestor {
            //             format!(
            //                 " - controlled. original func {}({}dynamic)",
            //                 func.name,
            //                 if this_tuple.is_dynamic_func {
            //                     ""
            //                 } else {
            //                     "non-"
            //                 }
            //             )
            //         } else {
            //             "".to_string()
            //         }
            //     );
            // }

            result.insert(av_id, controlling_tuple);

            av_queue.extend(
                AttributeValue::get_child_av_ids_for_ordered_parent(ctx, av_id)
                    .await?
                    .into_iter()
                    .map(|child_av_id| (child_av_id, Some(av_id)))
                    .collect::<VecDeque<_>>(),
            );
        }

        Ok(result)
    }
    /// Checks the destination and source component to determine if data flow between them
    /// Both "deleted" and not deleted Components can feed data into
    /// "deleted" Components. **ONLY** not deleted Components can feed
    /// data into not deleted Components.
    pub async fn should_data_flow_between_components(
        ctx: &DalContext,
        destination_component_id: ComponentId,
        source_component_id: ComponentId,
    ) -> ComponentResult<bool> {
        let destination_component_is_delete =
            Self::is_set_to_delete(ctx, destination_component_id).await?;
        let source_component_is_delete = Self::is_set_to_delete(ctx, source_component_id).await?;
        let should_data_flow = destination_component_is_delete || !source_component_is_delete;
        Ok(should_data_flow)
    }
    /// Simply gets the to_delete status for a component via the Node Weight
    async fn is_set_to_delete(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<bool> {
        let component_idx = ctx
            .workspace_snapshot()?
            .get_node_index_by_id(component_id)
            .await?;
        let component_node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(component_idx)
            .await?
            .get_component_node_weight()?;
        Ok(component_node_weight.to_delete())
    }
    async fn modify<L>(self, ctx: &DalContext, lambda: L) -> ComponentResult<Self>
    where
        L: FnOnce(&mut Self) -> ComponentResult<()>,
    {
        let original_component = self.clone();
        let mut component = self;

        let before = ComponentContentV1::from(component.clone());
        lambda(&mut component)?;

        // The `to_delete` lives on the node itself, not in the content, so we need to be a little
        // more manual when updating that field.
        if component.to_delete != original_component.to_delete {
            let component_idx = ctx
                .workspace_snapshot()?
                .get_node_index_by_id(original_component.id)
                .await?;
            let component_node_weight = ctx
                .workspace_snapshot()?
                .get_node_weight(component_idx)
                .await?
                .get_component_node_weight()?;
            let mut new_component_node_weight =
                component_node_weight.new_with_incremented_vector_clock(ctx.change_set()?)?;
            new_component_node_weight.set_to_delete(component.to_delete);
            ctx.workspace_snapshot()?
                .add_node(NodeWeight::Component(new_component_node_weight))
                .await?;
            ctx.workspace_snapshot()?
                .replace_references(component_idx)
                .await?;
        }

        let updated = ComponentContentV1::from(component.clone());
        if updated != before {
            let (hash, _) = ctx
                .layer_db()
                .cas()
                .write(
                    Arc::new(ComponentContent::V1(updated.clone()).into()),
                    None,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
            ctx.workspace_snapshot()?
                .update_content(ctx.change_set()?, component.id.into(), hash)
                .await?;
        }

        let component_node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(original_component.id)
            .await?
            .get_component_node_weight()?;

        Ok(Component::assemble(&component_node_weight, updated))
    }

    pub async fn remove(ctx: &DalContext, id: ComponentId) -> ComponentResult<()> {
        let change_set = ctx.change_set()?;

        let component = Self::get_by_id(ctx, id).await?;

        let _ = Frame::orphan_child(ctx, id)
            .await
            .map_err(|e| ComponentError::Frame(Box::new(e)));
        for incoming_connection in component.incoming_connections(ctx).await? {
            Component::remove_connection(
                ctx,
                incoming_connection.from_component_id,
                incoming_connection.from_output_socket_id,
                incoming_connection.to_component_id,
                incoming_connection.to_input_socket_id,
            )
            .await?;
        }
        for (output_socket_id, _) in
            Component::output_socket_attribute_values_for_component_id(ctx, id).await?
        {
            let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
            let apa_ids = output_socket.prototype_arguments_using(ctx).await?;
            for apa_id in apa_ids {
                let prototype_argument = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
                if let Some(targets) = prototype_argument.targets() {
                    if targets.source_component_id == id {
                        AttributePrototypeArgument::remove(ctx, apa_id).await?;
                    }
                }
            }
        }

        // Remove all actions for this component from queue
        Action::remove_all_for_component_id(ctx, id)
            .await
            .map_err(|err| ComponentError::Action(Box::new(err)))?;
        WsEvent::action_list_updated(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;

        ctx.workspace_snapshot()?
            .remove_node_by_id(change_set, id)
            .await?;

        WsEvent::component_deleted(ctx, id)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }

    pub async fn delete(self, ctx: &DalContext) -> ComponentResult<Option<Self>> {
        if self.resource(ctx).await?.is_none() {
            Self::remove(ctx, self.id).await?;
            Ok(None)
        } else {
            Ok(Some(self.set_to_delete(ctx, true).await?))
        }
    }

    pub async fn set_to_delete(self, ctx: &DalContext, to_delete: bool) -> ComponentResult<Self> {
        let component_id = self.id;
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;

        let modified = self
            .modify(ctx, |component| {
                component.to_delete = to_delete;
                Ok(())
            })
            .await?;

        // If we're clearing the `to_delete` flag, we need to make sure our inputs are updated
        // appropriately, as we may have an input connected to a still `to_delete` component, and
        // we should not be using it for input as long as it's still marked `to_delete`.
        //
        // If we're setting the `to_delete` flag, then we may need to pick up inputs from other
        // `to_delete` Components that we were ignoring before.
        //
        // This will update more than is strictly necessary, but it will ensure that everything is
        // correct.

        let input_av_ids: Vec<AttributeValueId> = modified
            .input_socket_attribute_values(ctx)
            .await?
            .values()
            .map(|f| &f.attribute_value_id)
            .cloned()
            .collect();

        ctx.add_dependent_values_and_enqueue(input_av_ids).await?;

        // We always want to make sure that everything "downstream" of us reacts appropriately
        // regardless of whether we're setting, or clearing the `to_delete` flag.
        //
        // We can't use self.output_socket_attribute_values here, and just enqueue a dependent
        // values update for those IDs, as the DVU explicitly *does not* update a not-to_delete AV,
        // using a source from a to_delete AV, and we want the not-to_delete AVs to be updated to
        // reflect that they're not getting data from this to_delete Component any more.

        let downstream_av_ids = modified.downstream_attribute_value_ids(ctx).await?;

        ctx.add_dependent_values_and_enqueue(downstream_av_ids)
            .await?;

        // Deal with deletion actions
        if to_delete {
            // Enqueue delete actions for component
            for prototype_id in SchemaVariant::find_action_prototypes_by_kind(
                ctx,
                schema_variant_id,
                ActionKind::Destroy,
            )
            .await?
            {
                Action::new(ctx, prototype_id, Some(component_id))
                    .await
                    .map_err(|err| ComponentError::Action(Box::new(err)))?;
            }
        } else {
            // Remove delete actions for component
            Action::remove_all_for_component_id(ctx, component_id)
                .await
                .map_err(|err| ComponentError::Action(Box::new(err)))?;
            WsEvent::action_list_updated(ctx)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        Ok(modified)
    }

    /// `AttributeValueId`s of all input sockets connected to any output socket of this component.
    async fn downstream_attribute_value_ids(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut results = Vec::new();

        let output_sockets: Vec<OutputSocketMatch> = self
            .output_socket_attribute_values(ctx)
            .await?
            .values()
            .copied()
            .collect();
        for output_socket_match in output_sockets {
            let output_socket =
                OutputSocket::get_by_id(ctx, output_socket_match.output_socket_id).await?;
            for argument_using_id in output_socket.prototype_arguments_using(ctx).await? {
                let argument_using =
                    AttributePrototypeArgument::get_by_id(ctx, argument_using_id).await?;
                if let Some(targets) = argument_using.targets() {
                    if targets.source_component_id == self.id() {
                        let prototype_id = argument_using.prototype_id(ctx).await?;
                        for maybe_downstream_av_id in
                            AttributePrototype::attribute_value_ids(ctx, prototype_id).await?
                        {
                            if AttributeValue::component_id(ctx, maybe_downstream_av_id).await?
                                == targets.destination_component_id
                            {
                                results.push(maybe_downstream_av_id);
                            }
                        }
                    }
                }
            }
            // also need to make sure inferred sockets are re-ran if there are any
            let inferred_inputs = Self::find_inferred_values_using_this_output_socket(
                ctx,
                output_socket_match.attribute_value_id,
            )
            .await?
            .into_iter()
            .map(|input| input.attribute_value_id)
            .collect_vec();
            results.extend(inferred_inputs)
        }

        Ok(results)
    }

    pub async fn copy_paste(&self, ctx: &DalContext, offset: (f64, f64)) -> ComponentResult<Self> {
        let schema_variant = self.schema_variant(ctx).await?;

        let mut pasted_comp = Component::new(
            ctx,
            format!("{} - Copy", self.name(ctx).await?),
            schema_variant.id(),
        )
        .await?;

        let x: f64 = self.x().parse()?;
        let y: f64 = self.y().parse()?;
        pasted_comp
            .set_geometry(
                ctx,
                (x + offset.0).to_string(),
                (y + offset.1).to_string(),
                self.width(),
                self.height(),
            )
            .await?;

        pasted_comp
            .clone_attributes_from(ctx, self.id(), true, true)
            .await?;
        Ok(pasted_comp)
    }
    /// For a given [`ComponentId`], map each input socket to the inferred output sockets
    /// it is connected to. Inferred socket connections are determined by following
    /// the ancestry line of FrameContains edges and matching the relevant input to output sockets.
    #[instrument(level = "debug", skip_all)]
    pub async fn build_map_for_component_id_inferred_incoming_connections(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<InputSocketMatch, Vec<OutputSocketMatch>>> {
        let mut results = HashMap::new();
        let input_sockets =
            Self::input_socket_attribute_values_for_component_id(ctx, component_id).await?;
        for (_, input_socket_match) in input_sockets {
            let output_matches =
                Self::find_available_inferred_connections_to_input_socket(ctx, input_socket_match)
                    .await?;
            if !output_matches.is_empty() {
                results.entry(input_socket_match).or_insert(output_matches);
            }
        }
        debug!(
            "Map of inferred input to output connections for component {:?}: {:?}",
            component_id, results
        );
        Ok(results)
    }
    /// For a given [`ComponentId`], map each output socket to the inferred input sockets
    /// it is connected to. Inferred socket connections are determined by following the
    /// lineage of Frame Contains edges and matching relevant output to input sockets.
    #[instrument(level = "debug", skip_all)]
    pub async fn build_map_for_component_id_inferred_outgoing_connections(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<HashMap<OutputSocketMatch, Vec<InputSocketMatch>>> {
        let mut results = HashMap::new();
        let output_sockets =
            Self::output_socket_attribute_values_for_component_id(ctx, component_id).await?;
        for (_, output_socket_match) in output_sockets {
            let input_matches = Self::find_inferred_values_using_this_output_socket(
                ctx,
                output_socket_match.attribute_value_id,
            )
            .await?;
            if !input_matches.is_empty() {
                results.entry(output_socket_match).or_insert(input_matches);
            }
        }
        debug!(
            "Map of inferred input to output connections for component {:?}: {:?}",
            component_id, results
        );
        Ok(results)
    }
    /// For a given [`InputSocketMatch`], find the inferred [`OutputSocketMatch`]es that are driving it
    /// if it exists. This walks up or down the component lineage tree depending on the [`ComponentType`]
    /// and finds the closest matching [`OutputSocket`]
    ///
    /// When walking down the lineage tree, we allow multiple output sockets to drive an input socket
    /// if the input socket has arity many and the matches are all siblings
    ///
    /// Note: this does not check for whether data should actually flow between components
    #[instrument(level = "debug", skip(ctx))]
    pub async fn find_available_inferred_connections_to_input_socket(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
    ) -> ComponentResult<Vec<OutputSocketMatch>> {
        let mut destination_sockets = vec![];
        if InputSocket::is_manually_configured(ctx, input_socket_match).await? {
            //if the input socket is being manually driven (the user has drawn an edge)
            // there will be no inferred connections to it
            return Ok(destination_sockets);
        }

        destination_sockets =
            match Component::get_type_by_id(ctx, input_socket_match.component_id).await? {
                ComponentType::Component | ComponentType::ConfigurationFrameDown => {
                    //For a component, or a down frame, check my parents and other ancestors
                    // find the first output socket match that is a down frame and use it!

                    if let Some(output_match) = Self::find_first_output_socket_match_in_ancestors(
                        ctx,
                        input_socket_match,
                        vec![ComponentType::ConfigurationFrameDown],
                    )
                    .await?
                    {
                        vec![output_match]
                    } else {
                        vec![]
                    }
                }
                ComponentType::ConfigurationFrameUp => {
                    // An up frame's input sockets are sourced from its children's output sockets
                    // For now, we won't let down frames send outputs to parents and children
                    // This might need to change, but we can change it when we've got a use case.
                    let mut matches = Self::find_available_output_socket_match_in_descendants(
                        ctx,
                        input_socket_match,
                        vec![
                            ComponentType::ConfigurationFrameUp,
                            ComponentType::Component,
                        ],
                    )
                    .await?;
                    // if there is more than one match, sort by component Ulid so they're
                    // consistently ordered
                    matches.sort_by_key(|output_socket| output_socket.component_id);
                    matches
                }
                ComponentType::AggregationFrame => vec![],
            };
        debug!(
            "Source socket for input socket {:?} is: {:?}",
            input_socket_match, destination_sockets
        );

        Ok(destination_sockets)
    }
    /// Walk down the component lineage to find all matching input sockets that a given output
    /// socket is driving
    ///
    /// Note: This does not check if data should actually flow between the components
    #[instrument(level = "debug", skip(ctx))]
    async fn find_all_potential_inferred_input_socket_matches_in_descendants(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
        component_id: ComponentId,
        component_types: Vec<ComponentType>,
    ) -> ComponentResult<Vec<InputSocketMatch>> {
        let mut found_sockets: Vec<InputSocketMatch> = vec![];
        let mut work_queue = VecDeque::from([component_id]);
        while let Some(component_id) = work_queue.pop_front() {
            if component_types.contains(&Component::get_type_by_id(ctx, component_id).await?) {
                //for each child, gather the input socket map for it
                // find the input sockets that consider this output socket an input
                // aggregate them as there might be many (for example a region frame passing values to many children)

                let matchy_matchy =
                    Component::build_map_for_component_id_inferred_incoming_connections(
                        ctx,
                        component_id,
                    )
                    .await?;
                for key in matchy_matchy.keys() {
                    if let Some((input_socket_match, output_socket_matches)) =
                        matchy_matchy.get_key_value(key)
                    {
                        for output_socket_match in output_socket_matches {
                            if output_socket_match.output_socket_id == output_socket_id {
                                found_sockets.push(*input_socket_match);
                            }
                        }
                    }
                }
                for child in Self::get_children_for_id(ctx, component_id).await? {
                    work_queue.push_back(child);
                }
            }
        }

        Ok(found_sockets)
    }

    /// For a given [`InputSocketMatch`], see if there are any [`OutputSocketMatch`]es for the provided
    /// [`ComponentId`]
    ///
    ///  Note: this does not check to see whether data should actually flow
    #[instrument(level = "debug" skip(ctx))]
    async fn find_potential_inferred_output_socket_matches_in_component(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
        source_component_id: ComponentId,
    ) -> ComponentResult<Vec<OutputSocketMatch>> {
        // check for matching output socket names for this input socket
        let parent_sv_id = Self::schema_variant_id(ctx, source_component_id).await?;
        let output_socket_ids =
            OutputSocket::list_ids_for_schema_variant(ctx, parent_sv_id).await?;
        let mut maybe_matches = vec![];

        for output_socket_id in output_socket_ids {
            if OutputSocket::fits_input_by_id(
                ctx,
                input_socket_match.input_socket_id,
                output_socket_id,
            )
            .await?
            {
                if let Some(output_socket_match) =
                    Self::output_socket_match(ctx, source_component_id, output_socket_id).await?
                {
                    maybe_matches.push(OutputSocketMatch {
                        component_id: source_component_id,
                        output_socket_id,
                        attribute_value_id: output_socket_match.attribute_value_id,
                    });
                }
            }
        }

        Ok(maybe_matches)
    }
    /// Find all [`InputSocketMatch`]es in the ancestry tree for a [`Component`] with the provided [`ComponentId`]
    /// This searches for matches in the component's parents and up the entire lineage tree
    ///
    /// Note: this does not check if data should actually flow between the components with matches
    #[instrument(level = "debug" skip(ctx))]
    async fn find_all_input_socket_matches_in_ascendants(
        ctx: &DalContext,
        output_socket_id: OutputSocketId,
        component_id: ComponentId,
        component_types: Vec<ComponentType>,
    ) -> ComponentResult<Vec<InputSocketMatch>> {
        let maybe_parent_id = Self::get_parent_by_id(ctx, component_id).await?;

        let mut found_sockets: Vec<InputSocketMatch> = vec![];
        let Some(parent_id) = maybe_parent_id else {
            return Ok(found_sockets);
        };
        let mut work_queue = VecDeque::from([parent_id]);
        while let Some(working_component_id) = work_queue.pop_front() {
            if component_types
                .contains(&Component::get_type_by_id(ctx, working_component_id).await?)
            {
                //for each parent, gather the input socket map for it
                // find the input sockets that consider this output socket an input
                // aggregate them as there might be many

                let matchy_matchy =
                    Component::build_map_for_component_id_inferred_incoming_connections(
                        ctx,
                        working_component_id,
                    )
                    .await?;
                for key in matchy_matchy.keys() {
                    if let Some((input_socket_match, output_socket_matches)) =
                        matchy_matchy.get_key_value(key)
                    {
                        for output_socket_match in output_socket_matches {
                            if output_socket_match.output_socket_id == output_socket_id {
                                debug!(
                                    "Found matching input socket {:?} for component id {}",
                                    input_socket_match, working_component_id
                                );
                                found_sockets.push(*input_socket_match);
                            }
                        }
                    }
                }
                if let Some(parent) = Self::get_parent_by_id(ctx, working_component_id).await? {
                    work_queue.push_back(parent);
                }
            }
        }

        Ok(found_sockets)
    }

    /// Finds all inferred incoming connections for the [`Component`]
    /// A connection is inferred if it's input socket is being driven
    /// by another component's output socket as a result of lineage
    /// via FrameContains Edges.
    #[instrument(level = "debug", skip(ctx))]
    pub async fn inferred_incoming_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<InferredIncomingConnection>> {
        let to_component_id = self.id();
        let mut connections = vec![];
        let input_sockets =
            Self::input_socket_attribute_values_for_component_id(ctx, to_component_id).await?;
        for (to_input_socket_id, input_socket_match) in input_sockets.into_iter() {
            for output_socket_match in
                Self::find_available_inferred_connections_to_input_socket(ctx, input_socket_match)
                    .await?
            {
                // add the check for to_delete on either to or from component
                // Both "deleted" and not deleted Components can feed data into
                // "deleted" Components. **ONLY** not deleted Components can feed
                // data into not deleted Components.
                let destination_component = Self::get_by_id(ctx, to_component_id).await?;
                let source_component =
                    Self::get_by_id(ctx, output_socket_match.component_id).await?;
                let to_delete = !Self::should_data_flow_between_components(
                    ctx,
                    destination_component.id,
                    source_component.id,
                )
                .await?;

                let implicit_edge = InferredIncomingConnection {
                    to_component_id,
                    to_input_socket_id,
                    from_component_id: output_socket_match.component_id,
                    from_output_socket_id: output_socket_match.output_socket_id,
                    to_delete,
                };
                debug!("Found inferred edge: {:?}", implicit_edge);
                connections.push(implicit_edge);
            }
        }
        Ok(connections)
    }
    /// Finds all inferred outgoing connections for the [`Component`]
    /// A connection is inferred if it's output sockets is driving
    /// another component's [`InputSocket`] as a result of lineage via
    /// the FrameContains edge.
    #[instrument(level = "debug", skip(ctx))]
    pub async fn inferred_outgoing_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<InferredIncomingConnection>> {
        let from_component_id = self.id();
        let mut connections = vec![];
        let output_sockets =
            Self::output_socket_attribute_values_for_component_id(ctx, from_component_id).await?;
        for (from_output_socket_id, output_socket_match) in output_sockets.into_iter() {
            for input_socket_match in Self::find_inferred_values_using_this_output_socket(
                ctx,
                output_socket_match.attribute_value_id,
            )
            .await?
            {
                // add the check for to_delete on either to or from component
                // Both "deleted" and not deleted Components can feed data into
                // "deleted" Components. **ONLY** not deleted Components can feed
                // data into not deleted Components.
                let destination_component = input_socket_match.component_id;
                let source_component = self.id();

                let to_delete = !Self::should_data_flow_between_components(
                    ctx,
                    destination_component,
                    source_component,
                )
                .await?;

                let implicit_edge = InferredIncomingConnection {
                    to_component_id: input_socket_match.component_id,
                    to_input_socket_id: input_socket_match.input_socket_id,
                    from_component_id,
                    from_output_socket_id,
                    to_delete,
                };
                debug!("Found inferred edge: {:?}", implicit_edge);
                connections.push(implicit_edge);
            }
        }
        Ok(connections)
    }

    /// For the provided [`InputSocketMatch`], find any matching [`OutputSocketMatch`] that should
    /// drive this [`InputSocket`] by searching down the descendants of the [`Component`],
    /// checking children first and walking down until we find any matches.
    ///
    /// If the provided [`InputSocketMatch`] has an arity of one, we look for only one
    /// eligible [`OutputSocket`]. If we find multiple, we won't return any, forcing the
    /// user to explicity draw the edge.
    ///
    /// If it has an arity of many, we will look for multiple matches, but they must
    /// be at the same 'level' to be considered valid.
    ///
    /// Note: this does not check if data should actually flow between the components with matches,
    /// it only checks if there are available sockets that might be driven
    #[instrument(level = "debug", skip(ctx))]
    async fn find_available_output_socket_match_in_descendants(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
        component_types: Vec<ComponentType>,
    ) -> ComponentResult<Vec<OutputSocketMatch>> {
        let mut output_socket_matches: Vec<OutputSocketMatch> = vec![];
        let component_id = input_socket_match.component_id;
        let children = Component::get_children_for_id(ctx, component_id).await?;
        let socket_arrity = InputSocket::get_by_id(ctx, input_socket_match.input_socket_id)
            .await?
            .arity();
        //load up the children and look for matches
        let mut work_queue: VecDeque<Vec<ComponentId>> = VecDeque::new();
        work_queue.push_front(children);
        if socket_arrity == SocketArity::One {
            while let Some(children) = work_queue.pop_front() {
                if children.is_empty() {
                    break;
                }
                let (maybe_match, next_children) = Self::find_single_match_in_children(
                    ctx,
                    input_socket_match,
                    &component_types,
                    children,
                )
                .await?;
                // if there wasn't a match here, load up the next children
                // if there was, return
                match maybe_match {
                    Some(output_match) => {
                        output_socket_matches.push(output_match);
                        break;
                    }
                    None => work_queue.push_back(next_children),
                }
            }
        } else {
            while let Some(children) = work_queue.pop_front() {
                if children.is_empty() {
                    break;
                }
                let (maybe_matches, next_children) = Self::find_all_matches_in_children(
                    ctx,
                    input_socket_match,
                    &component_types,
                    children,
                )
                .await?;
                // if there are matches found, push them and stop looking
                // otherwise, load up the next children if there are any
                if maybe_matches.is_empty() && !next_children.is_empty() {
                    work_queue.push_back(next_children);
                } else {
                    output_socket_matches.extend(maybe_matches);
                    break;
                }
            }
        }
        Ok(output_socket_matches)
    }

    #[instrument(level = "debug", skip(ctx))]
    async fn find_single_match_in_children(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
        component_types: &[ComponentType],
        children: Vec<ComponentId>,
    ) -> ComponentResult<(Option<OutputSocketMatch>, Vec<ComponentId>)> {
        let mut maybe_output_match = None;
        let mut next_children = vec![];
        // when the input socket is an arity of one, we need to find one single matching output socket
        for child_component in children {
            match maybe_output_match.is_some() {
                true => {
                    // we already have a match, but let's see if there are more
                    // if there are, stop looking and return none, letting the user decide which
                    // single child to connect to
                    if component_types.contains(&Self::get_type_by_id(ctx, child_component).await?)
                    {
                        let maybe_matches =
                            Self::find_potential_inferred_output_socket_matches_in_component(
                                ctx,
                                input_socket_match,
                                child_component,
                            )
                            .await?;

                        if !maybe_matches.is_empty() {
                            // this component has too many matches,
                            return Ok((None, vec![]));
                        }
                    }
                }
                false => {
                    // no match yet, keep looking!
                    if component_types
                        .contains(&Component::get_type_by_id(ctx, child_component).await?)
                    {
                        let maybe_matches =
                            Self::find_potential_inferred_output_socket_matches_in_component(
                                ctx,
                                input_socket_match,
                                child_component,
                            )
                            .await?;
                        if !maybe_matches.is_empty() && maybe_matches.len() == 1 {
                            // found exactly 1! it just might be the one!
                            maybe_output_match = maybe_matches.first().cloned();
                        }
                    }
                }
            }

            let child_components = Component::get_children_for_id(ctx, child_component).await?;
            next_children.extend(child_components);
        }
        Ok((maybe_output_match, next_children))
    }

    #[instrument(level = "debug", skip(ctx))]
    async fn find_all_matches_in_children(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
        component_types: &[ComponentType],
        children: Vec<ComponentId>,
    ) -> ComponentResult<(Vec<OutputSocketMatch>, Vec<ComponentId>)> {
        let mut maybe_output_matches = vec![];
        let mut next_children = vec![];
        for child_component in children {
            match !maybe_output_matches.is_empty() {
                true => {
                    // we already have a match but we need to check siblings
                    // as there might be more than one match!
                    if component_types.contains(&Self::get_type_by_id(ctx, child_component).await?)
                    {
                        let maybe_matches =
                            Self::find_potential_inferred_output_socket_matches_in_component(
                                ctx,
                                input_socket_match,
                                child_component,
                            )
                            .await?;
                        // if there's only one match in this component, use it! otherwise keep looking in
                        // the other children
                        if maybe_matches.len() == 1 {
                            // found a single match in descendants!
                            if let Some(output_match) = maybe_matches.first().cloned() {
                                maybe_output_matches.push(output_match);
                            }
                        }
                    }
                }
                false => {
                    // no match yet, let's find if this child has any matches!
                    if component_types.contains(&Self::get_type_by_id(ctx, child_component).await?)
                    {
                        let maybe_matches =
                            Self::find_potential_inferred_output_socket_matches_in_component(
                                ctx,
                                input_socket_match,
                                child_component,
                            )
                            .await?;

                        if maybe_matches.len() == 1 {
                            // found one match in this descendant!
                            if let Some(output_match) = maybe_matches.first().cloned() {
                                maybe_output_matches.push(output_match);
                            }
                        }
                    }
                }
            }
            let child_components = Component::get_children_for_id(ctx, child_component).await?;
            next_children.extend(child_components);
        }
        Ok((maybe_output_matches, next_children))
    }

    /// For the provided [`InputSocketMatch`], find the first [`OutputSocketMatch`] in the ancestry tree
    /// that should drive this [`InputSocket`] (first searching parents and onwards up the ancestry tree)
    #[instrument(level = "debug", skip(ctx))]
    pub async fn find_first_output_socket_match_in_ancestors(
        ctx: &DalContext,
        input_socket_match: InputSocketMatch,
        component_types: Vec<ComponentType>,
    ) -> ComponentResult<Option<OutputSocketMatch>> {
        if let Some(parent_id) =
            Component::get_parent_by_id(ctx, input_socket_match.component_id).await?
        {
            let mut work_queue = VecDeque::from([parent_id]);
            while let Some(component_id) = work_queue.pop_front() {
                // see if this component is the right type

                if component_types.contains(&Component::get_type_by_id(ctx, component_id).await?) {
                    // get all output sockets for this component
                    let maybe_matches =
                        Self::find_potential_inferred_output_socket_matches_in_component(
                            ctx,
                            input_socket_match,
                            component_id,
                        )
                        .await?;
                    {
                        if maybe_matches.len() > 1 {
                            // this ancestor has more than one match
                            // stop looking and return None to force
                            // the user to manually draw an edge to this socket
                            debug!("More than one match found: {:?}", maybe_matches);
                            return Ok(None);
                        }
                        if maybe_matches.len() == 1 {
                            // this ancestor has 1 match!
                            // return and stop looking
                            return Ok(maybe_matches.first().cloned());
                        }
                    }
                }
                // didn't find it, so let's queue up the next parent if it exists
                if let Some(maybe_parent_id) =
                    Component::get_parent_by_id(ctx, component_id).await?
                {
                    work_queue.push_back(maybe_parent_id);
                }
            }
        }

        Ok(None)
    }

    /// Find all inferred [`InputSocketMatch`]es that are being driven by the provided
    /// [`AttributeValueId`] that represents an [`OutputSocket`] for a specific [`Component`]
    ///
    /// Output sockets can drive Input Sockets through inference based on the following logic:
    ///
    /// Components and Up Frames can drive Input Sockets of their parents if the parent is an
    /// Up Frame.
    ///
    /// Down Frames can drive Input Sockets of their children if the child is a Down Frame
    /// or a Component.
    #[instrument(level = "debug", skip(ctx))]
    pub async fn find_inferred_values_using_this_output_socket(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ComponentResult<Vec<InputSocketMatch>> {
        // let's make sure this av is actually for an output socket
        let value_is_for = AttributeValue::is_for(ctx, attribute_value_id).await?;
        let output_socket_id = match value_is_for {
            ValueIsFor::Prop(_) | ValueIsFor::InputSocket(_) => {
                return Err(ComponentError::WrongAttributeValueType(
                    attribute_value_id,
                    value_is_for,
                ))
            }
            ValueIsFor::OutputSocket(sock) => sock,
        };
        let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
        let maybe_target_sockets = match Component::get_type_by_id(ctx, component_id).await? {
            ComponentType::Component | ComponentType::ConfigurationFrameUp => {
                // if the type is a component, find all ascendants
                // who have a matching input socket AND are an up frame
                Component::find_all_input_socket_matches_in_ascendants(
                    ctx,
                    output_socket_id,
                    component_id,
                    vec![ComponentType::ConfigurationFrameUp],
                )
                .await?
            }
            ComponentType::ConfigurationFrameDown => {
                // if the type is a down frame, find all descendants
                // who have a matching input socket AND are a Down Frame or Component
                Component::find_all_potential_inferred_input_socket_matches_in_descendants(
                    ctx,
                    output_socket_id,
                    component_id,
                    vec![
                        ComponentType::ConfigurationFrameDown,
                        ComponentType::Component,
                    ],
                )
                .await?
            }

            // we are not supporting aggregation frames right now
            _ => vec![],
        };

        Ok(maybe_target_sockets)
    }

    #[instrument(level = "info", skip(ctx))]
    pub async fn remove_connection(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<()> {
        let input_socket_prototype_id =
            AttributePrototype::find_for_input_socket(ctx, destination_input_socket_id)
                .await?
                .ok_or_else(|| InputSocketError::MissingPrototype(destination_input_socket_id))?;

        let attribute_prototype_arguments = ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                input_socket_prototype_id,
                Outgoing,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?;

        for (_, _, attribute_prototype_arg_idx) in attribute_prototype_arguments {
            let node_weight = ctx
                .workspace_snapshot()?
                .get_node_weight(attribute_prototype_arg_idx)
                .await?;
            let attribute_prototype_argument_node_weight =
                node_weight.get_attribute_prototype_argument_node_weight()?;
            if let Some(targets) = attribute_prototype_argument_node_weight.targets() {
                if targets.source_component_id == source_component_id
                    && targets.destination_component_id == destination_component_id
                {
                    let data_sources = ctx
                        .workspace_snapshot()?
                        .edges_directed_for_edge_weight_kind(
                            attribute_prototype_argument_node_weight.id(),
                            Outgoing,
                            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                        )
                        .await?;

                    for (_, _, data_source_idx) in data_sources {
                        let node_weight = ctx
                            .workspace_snapshot()?
                            .get_node_weight(data_source_idx)
                            .await?;
                        if let Ok(output_socket_node_weight) = node_weight
                            .get_content_node_weight_of_kind(
                                ContentAddressDiscriminants::OutputSocket,
                            )
                        {
                            if output_socket_node_weight.id() == source_output_socket_id.into() {
                                AttributePrototypeArgument::remove(
                                    ctx,
                                    attribute_prototype_argument_node_weight.id().into(),
                                )
                                .await?;

                                let destination_attribute_value_ids =
                                    InputSocket::attribute_values_for_input_socket_id(
                                        ctx,
                                        destination_input_socket_id,
                                    )
                                    .await?;
                                // filter the value ids by destination_component_id
                                let mut destination_attribute_value_id: Option<AttributeValueId> =
                                    None;
                                for value_id in destination_attribute_value_ids {
                                    let component_id =
                                        AttributeValue::component_id(ctx, value_id).await?;
                                    if component_id == destination_component_id {
                                        destination_attribute_value_id = Some(value_id);
                                        break;
                                    }
                                }

                                let destination_attribute_value_id = destination_attribute_value_id.ok_or(
                                ComponentError::DestinationComponentMissingAttributeValueForInputSocket(
                                         destination_component_id,
                                         destination_input_socket_id,
                                       ),
                                   )?;

                                ctx.add_dependent_values_and_enqueue(vec![
                                    destination_attribute_value_id,
                                ])
                                .await?;
                                return Ok(());
                            }
                        }
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            }
        }

        Ok(())
    }

    #[instrument(level = "debug", skip(ctx))]
    pub async fn upgrade_to_new_variant(
        &self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<Component> {
        let original_component = Self::get_by_id(ctx, self.id).await?;

        let original_component_name = self.name(ctx).await?;
        let mut new_component =
            Component::new(ctx, original_component_name.clone(), schema_variant_id).await?;

        let new_comp_schema_variant_id = new_component.schema_variant(ctx).await?.id();
        if new_comp_schema_variant_id != schema_variant_id {
            return Err(ComponentError::ComponentIncorrectSchemaVariant(
                new_component.id(),
            ));
        }

        new_component
            .clone_attributes_from(ctx, self.id(), false, false)
            .await?;
        new_component
            .set_geometry(ctx, self.x(), self.y(), self.width(), self.height())
            .await?;

        if schema_variant_id
            != Component::get_by_id(ctx, new_component.id())
                .await?
                .schema_variant(ctx)
                .await?
                .id()
        {
            return Err(ComponentError::ComponentIncorrectSchemaVariant(
                new_component.id(),
            ));
        }

        //Re-attach to any parent it has
        if let Some(parent) = original_component.parent(ctx).await? {
            Frame::upsert_parent(ctx, new_component.id(), parent)
                .await
                .map_err(Box::new)?;
        }

        // Re-attach any children to the new component
        for child in Component::get_children_for_id(ctx, original_component.id).await? {
            Frame::upsert_parent(ctx, child, new_component.id())
                .await
                .map_err(Box::new)?;
        }

        // Let's change the incoming connections to the component!
        for incoming in original_component.incoming_connections(ctx).await? {
            Component::remove_connection(
                ctx,
                incoming.from_component_id,
                incoming.from_output_socket_id,
                incoming.to_component_id,
                incoming.to_input_socket_id,
            )
            .await?;

            let socket = InputSocket::get_by_id(ctx, incoming.to_input_socket_id).await?;
            if let Some(socket) =
                InputSocket::find_with_name(ctx, socket.name(), schema_variant_id).await?
            {
                Component::connect(
                    ctx,
                    incoming.from_component_id,
                    incoming.from_output_socket_id,
                    new_component.id(),
                    socket.id(),
                )
                .await?;
            } else {
                debug!(
                    "Unable to reconnect to socket_id: {0} for component_id: {1}",
                    socket.id(),
                    new_component.id()
                );
            }
        }

        for outgoing in original_component.outgoing_connections(ctx).await? {
            Component::remove_connection(
                ctx,
                outgoing.from_component_id,
                outgoing.from_output_socket_id,
                outgoing.to_component_id,
                outgoing.to_input_socket_id,
            )
            .await?;

            let socket = OutputSocket::get_by_id(ctx, outgoing.from_output_socket_id).await?;
            if let Some(socket) =
                OutputSocket::find_with_name(ctx, socket.name(), schema_variant_id).await?
            {
                Component::connect(
                    ctx,
                    new_component.id(),
                    socket.id(),
                    outgoing.to_component_id,
                    outgoing.to_input_socket_id,
                )
                .await?;
            } else {
                debug!(
                    "Unable to reconnect to socket_id: {0} for component_id: {1}",
                    socket.id(),
                    new_component.id()
                );
            }
        }

        // Let's requeue any Actions for the component
        Self::requeue_actions_for_upgraded_component(
            ctx,
            original_component.id(),
            new_component.id(),
            new_comp_schema_variant_id,
        )
        .await?;

        // Let's remove the original resource so that we don't queue a delete action
        original_component.clear_resource(ctx).await?;
        original_component.delete(ctx).await?;

        Ok(new_component)
    }

    async fn requeue_actions_for_upgraded_component(
        ctx: &DalContext,
        old_component_id: ComponentId,
        new_component_id: ComponentId,
        new_schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<()> {
        let mut queue_create = false;

        let queued_for_old_component = Action::find_for_component_id(ctx, old_component_id)
            .await
            .map_err(|err| ComponentError::Action(Box::new(err)))?;
        let available_for_new_component = ActionPrototype::for_variant(ctx, new_schema_variant_id)
            .await
            .map_err(|err| ComponentError::ActionPrototype(Box::new(err)))?;
        for existing_queued in queued_for_old_component {
            let action = Action::get_by_id(ctx, existing_queued)
                .await
                .map_err(|err| ComponentError::Action(Box::new(err)))?;
            let action_prototype_id = Action::prototype_id(ctx, existing_queued)
                .await
                .map_err(|err| ComponentError::Action(Box::new(err)))?;
            // what do we do about the various states?
            // maybe you shouldn't upgrade a component if an action
            // is dispatched or running for the current?
            match action.state() {
                ActionState::Failed | ActionState::OnHold | ActionState::Queued => {
                    let action_prototype = ActionPrototype::get_by_id(ctx, action_prototype_id)
                        .await
                        .map_err(|err| ComponentError::ActionPrototype(Box::new(err)))?;
                    let func_id = ActionPrototype::func_id(ctx, action_prototype_id)
                        .await
                        .map_err(|err| ComponentError::ActionPrototype(Box::new(err)))?;
                    let queued_func = Func::get_by_id_or_error(ctx, func_id).await?;

                    if action_prototype.kind == ActionKind::Create {
                        queue_create = true;
                    }
                    for available_action_prototype in available_for_new_component.clone() {
                        let available_func_id =
                            ActionPrototype::func_id(ctx, available_action_prototype.id())
                                .await
                                .map_err(|err| ComponentError::ActionPrototype(Box::new(err)))?;
                        let available_func =
                            Func::get_by_id_or_error(ctx, available_func_id).await?;

                        if available_func.name == queued_func.name
                            && available_func.kind == queued_func.kind
                        {
                            Action::new(ctx, action_prototype_id, Some(new_component_id))
                                .await
                                .map_err(|err| ComponentError::Action(Box::new(err)))?;
                        }
                    }
                }
                ActionState::Running | ActionState::Dispatched => continue,
            }
        }
        if !queue_create {
            let create_actions =
                Action::find_for_kind_and_component_id(ctx, new_component_id, ActionKind::Create)
                    .await
                    .map_err(|err| ComponentError::Action(Box::new(err)))?;
            for action_id in create_actions {
                Action::remove_by_id(ctx, action_id)
                    .await
                    .map_err(|err| ComponentError::Action(Box::new(err)))?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCreatedPayload {
    success: bool,
    component_id: ComponentId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentUpdatedPayload {
    pub component: SummaryDiagramComponent,
    pub change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentUpgradedPayload {
    component: SummaryDiagramComponent,
    change_set_id: ChangeSetId,
    original_component_id: ComponentId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDeletedPayload {
    component_id: ComponentId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionCreatedPayload {
    from_component_id: ComponentId,
    to_component_id: ComponentId,
    from_socket_id: OutputSocketId,
    to_socket_id: InputSocketId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDeletedPayload {
    from_component_id: ComponentId,
    to_component_id: ComponentId,
    from_socket_id: OutputSocketId,
    to_socket_id: InputSocketId,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPosition {
    x: i32,
    y: i32,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSize {
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSetPosition {
    component_id: ComponentId,
    position: ComponentPosition,
    size: Option<ComponentSize>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSetPositionPayload {
    change_set_id: ChangeSetId,
    positions: Vec<ComponentSetPosition>,
    user_pk: Option<UserPk>,
}

impl ComponentSetPositionPayload {
    pub fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferredEdgeRemovePayload {
    change_set_id: ChangeSetId,
    edges: Vec<SummaryDiagramInferredEdge>,
}

impl InferredEdgeRemovePayload {
    pub fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InferredEdgeUpsertPayload {
    change_set_id: ChangeSetId,
    edges: Vec<SummaryDiagramInferredEdge>,
}

impl InferredEdgeUpsertPayload {
    pub fn change_set_id(&self) -> ChangeSetId {
        self.change_set_id
    }
}

impl WsEvent {
    pub async fn remove_inferred_edges(
        ctx: &DalContext,
        edges: Vec<SummaryDiagramInferredEdge>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::InferredEdgeRemove(InferredEdgeRemovePayload {
                change_set_id: ctx.change_set_id(),
                edges,
            }),
        )
        .await
    }

    pub async fn upsert_inferred_edges(
        ctx: &DalContext,
        edges: Vec<SummaryDiagramInferredEdge>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::InferredEdgeUpsert(InferredEdgeUpsertPayload {
                change_set_id: ctx.change_set_id(),
                edges,
            }),
        )
        .await
    }

    pub async fn reflect_component_position(
        workspace_pk: WorkspacePk,
        change_set_id: ChangeSetId,
        payload: ComponentSetPositionPayload,
    ) -> WsEventResult<Self> {
        WsEvent::new_raw(
            workspace_pk,
            Some(change_set_id),
            WsPayload::SetComponentPosition(payload),
        )
        .await
    }

    pub async fn set_component_position(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        components: Vec<Component>,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        let mut positions: Vec<ComponentSetPosition> = vec![];
        for component in components {
            let position = ComponentPosition {
                x: component.x.parse()?,
                y: component.y.parse()?,
            };
            let size = ComponentSize {
                width: component.width.as_ref().map(|w| w.parse()).transpose()?,
                height: component.height.as_ref().map(|w| w.parse()).transpose()?,
            };
            positions.push(ComponentSetPosition {
                component_id: component.id(),
                position,
                size: Some(size),
            });
        }
        WsEvent::new(
            ctx,
            WsPayload::SetComponentPosition(ComponentSetPositionPayload {
                change_set_id,
                positions,
                user_pk,
            }),
        )
        .await
    }

    pub async fn component_created(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentCreated(ComponentCreatedPayload {
                success: true,
                change_set_id: ctx.change_set_id(),
                component_id,
            }),
        )
        .await
    }

    pub async fn connection_created(
        ctx: &DalContext,
        from_component_id: ComponentId,
        to_component_id: ComponentId,
        from_socket_id: OutputSocketId,
        to_socket_id: InputSocketId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ConnectionCreated(ConnectionCreatedPayload {
                from_component_id,
                to_component_id,
                from_socket_id,
                change_set_id: ctx.change_set_id(),
                to_socket_id,
            }),
        )
        .await
    }

    pub async fn connection_deleted(
        ctx: &DalContext,
        from_component_id: ComponentId,
        to_component_id: ComponentId,
        from_socket_id: OutputSocketId,
        to_socket_id: InputSocketId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ConnectionDeleted(ConnectionDeletedPayload {
                from_component_id,
                to_component_id,
                from_socket_id,
                change_set_id: ctx.change_set_id(),
                to_socket_id,
            }),
        )
        .await
    }

    pub async fn component_updated(
        ctx: &DalContext,
        payload: SummaryDiagramComponent,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentUpdated(ComponentUpdatedPayload {
                component: payload,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn component_upgraded(
        ctx: &DalContext,
        payload: SummaryDiagramComponent,
        original_component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentUpgraded(ComponentUpgradedPayload {
                component: payload,
                change_set_id: ctx.change_set_id(),
                original_component_id,
            }),
        )
        .await
    }

    pub async fn component_deleted(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentDeleted(ComponentDeletedPayload {
                component_id,
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }
}
