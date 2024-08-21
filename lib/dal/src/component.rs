//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](crate::SchemaVariant) and a _model_ of a "real world resource".

use inferred_connection_graph::InferredConnectionGraph;
use itertools::Itertools;
use petgraph::Direction::Outgoing;
use serde::{Deserialize, Serialize};
use si_pkg::KeyOrIndex;
use socket::{ComponentInputSocket, ComponentOutputSocket};
use std::collections::{HashMap, VecDeque};
use std::hash::Hash;
use std::num::ParseFloatError;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;

use si_events::{ulid::Ulid, ContentHash};

use crate::action::prototype::{ActionKind, ActionPrototype, ActionPrototypeError};
use crate::action::{Action, ActionError, ActionState};
use crate::actor_view::ActorView;
use crate::attribute::prototype::argument::value_source::ValueSource;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::prototype::{AttributePrototypeError, AttributePrototypeSource};
use crate::attribute::value::{
    AttributeValueError, ChildAttributeValuePair, DependentValueGraph, ValueIsFor,
};
use crate::change_set::ChangeSetError;
use crate::code_view::CodeViewError;
use crate::diagram::{SummaryDiagramComponent, SummaryDiagramInferredEdge};
use crate::history_event::HistoryEventMetadata;
use crate::layer_db_types::{ComponentContent, ComponentContentV1};
use crate::prop::{PropError, PropPath};
use crate::qualification::QualificationError;
use crate::schema::variant::leaves::LeafKind;
use crate::schema::variant::root_prop::component_type::ComponentType;
use crate::schema::variant::SchemaVariantError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::content_address::ContentAddressDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeightKind, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::node_weight::attribute_prototype_argument_node_weight::ArgumentTargets;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{ComponentNodeWeight, NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{AttributePrototypeId, SocketArity};
use frame::{Frame, FrameError};
use resource::ResourceData;

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
pub mod inferred_connection_graph;
pub mod properties;
pub mod qualification;
pub mod resource;
pub mod socket;

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
    #[error("cannot clone attributes from a component with a different schema variant id")]
    CannotCloneFromDifferentVariants,
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
    #[error("frame error: {0}")]
    Frame(#[from] Box<FrameError>),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("input socket {0} not found for component id {1}")]
    InputSocketNotFoundForComponentId(InputSocketId, ComponentId),
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
    #[error("output socket {0} not found for component id {1}")]
    OutputSocketNotFoundForComponentId(OutputSocketId, ComponentId),
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
    #[error("too many explicit connection sources ({0:?}) for component ({1}) and input socket ({2}) with an arity of one")]
    TooManyExplicitConnectionSources(Vec<ComponentId>, ComponentId, InputSocketId),
    #[error(
        "too many inferred connections ({0:?}) for input socket match ({1:?}) with an arity of one"
    )]
    TooManyInferredConnections(Vec<ComponentOutputSocket>, ComponentInputSocket),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("unexpected explicit source ({0}) and inferred source ({1}) for input socket match ({2:?}) with an arity of one")]
    UnexpectedExplicitAndInferredSources(ComponentId, ComponentId, ComponentInputSocket),
    #[error("value source for known prop attribute value {0} is not a prop id")]
    ValueSourceForPropValueNotPropId(AttributeValueId),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace pk not found on context")]
    WorkspacePkNone,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("attribute value {0} has wrong type for operation: {0}")]
    WrongAttributeValueType(AttributeValueId, ValueIsFor),
    #[error("Attribute Prototype Argument used by too many Attribute Prototypes: {0}")]
    WrongNumberOfPrototypesForAttributePrototypeArgument(AttributePrototypeArgumentId),
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

impl From<si_events::ComponentId> for ComponentId {
    fn from(value: si_events::ComponentId) -> Self {
        Self(value.into_raw_id())
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
pub struct InferredConnection {
    pub to_component_id: ComponentId,
    pub to_input_socket_id: InputSocketId,
    pub from_component_id: ComponentId,
    pub from_output_socket_id: OutputSocketId,
    pub to_delete: bool,
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

// Used to transfer the size and position of a component
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComponentGeometry {
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
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
        Self::view_by_id(ctx, self.id).await
    }

    pub async fn view_by_id(
        ctx: &DalContext,
        id: ComponentId,
    ) -> ComponentResult<Option<serde_json::Value>> {
        let schema_variant_id = Self::schema_variant_id(ctx, id).await?;
        let root_prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(["root"])).await?;

        for value_id in Component::attribute_values_for_prop_id(ctx, id, root_prop_id).await? {
            let value_component_id = AttributeValue::component_id(ctx, value_id).await?;
            if value_component_id == id {
                let root_value = AttributeValue::get_by_id_or_error(ctx, value_id).await?;
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

        Self::new_with_content_address(ctx, name, schema_variant_id, hash).await
    }

    pub async fn new_with_content_address(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
        content_address: ContentHash,
    ) -> ComponentResult<Self> {
        let name: String = name.into();

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;

        let node_weight = NodeWeight::new_component(id, lineage_id, content_address);

        // Attach component to category and add use edge to schema variant
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

    /// Attempts to merge the values other_component into this component, if
    /// values exist for the prop in other. Only use this immediately after
    /// Component::new, so that we can make certain assumptions (for example, we
    /// can assume that the prototypes are correct, and that arrays and maps are
    /// empty)
    async fn merge_from_component_with_different_schema_variant(
        &self,
        ctx: &DalContext,
        old_component_id: ComponentId,
    ) -> ComponentResult<()> {
        let old_root_id = Component::root_attribute_value_id(ctx, old_component_id).await?;
        let self_schema_variant_id = Component::schema_variant_id(ctx, self.id).await?;
        let mut attribute_values = vec![];

        // Gather up a bunch of data about the current schema variant
        let mut new_input_sockets = HashMap::new();
        for input_socket_id in
            InputSocket::list_ids_for_schema_variant(ctx, self_schema_variant_id).await?
        {
            let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
            new_input_sockets.insert(input_socket.name().to_string(), input_socket.id());
        }

        let mut new_output_sockets = HashMap::new();
        for output_socket_id in
            OutputSocket::list_ids_for_schema_variant(ctx, self_schema_variant_id).await?
        {
            let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
            new_output_sockets.insert(output_socket.name().to_string(), output_socket.id());
        }

        let mut new_props = HashMap::new();
        for prop in SchemaVariant::all_props(ctx, self_schema_variant_id).await? {
            let path = prop.path(ctx).await?;
            new_props.insert(path.as_owned_parts(), prop.id());
        }

        // Walk the original components attribute value tree, finding matching
        // values in self and updating their value if necessary. Also find if a
        // component specific dynamic function was configured in the original
        // component. If so, attempt to copy it over.
        let mut value_q = VecDeque::from([(old_root_id, None, None)]);
        while let Some((old_av_id, old_key_or_index, new_parent_id)) = value_q.pop_front() {
            let old_av = AttributeValue::get_by_id_or_error(ctx, old_av_id).await?;
            let old_component_prototype_id =
                AttributeValue::component_prototype_id(ctx, old_av_id).await?;
            let old_prop_id = AttributeValue::is_for(ctx, old_av_id)
                .await?
                .prop_id()
                .ok_or(ComponentError::ValueSourceForPropValueNotPropId(old_av_id))?;

            let prop_path = Prop::path_by_id(ctx, old_prop_id).await?.as_owned_parts();

            // Is there a matching prop in self for this prop in other? If there
            // is no matching prop do nothing (this means the prop was removed
            // from self, so can't get values from other)
            if let Some(&new_prop_id) = new_props.get(&prop_path) {
                let new_prop = Prop::get_by_id_or_error(ctx, new_prop_id).await?;
                let old_prop = Prop::get_by_id_or_error(ctx, old_prop_id).await?;

                // Prop kinds c ould have changed for the same prop. We could
                // try and coerce values, but it's safer to just skip.  Even if
                // there is a component specific prototype for this prop's value
                // in other, we don't want to copy it over, since the kind has
                // changed.
                if new_prop.kind != old_prop.kind {
                    continue;
                }

                // Similarly, we should verify that the secret kind has not
                // changed if this is a secret prop. If it has changed, leave
                // the prop alone (effectively empyting the secret)
                if new_prop.secret_kind_widget_option() != old_prop.secret_kind_widget_option() {
                    continue;
                }

                let mut new_av_id = None;
                for maybe_new_av_id in
                    Component::attribute_values_for_prop_id(ctx, self.id, new_prop_id).await?
                {
                    let new_key_or_index =
                        AttributeValue::get_index_or_key_of_child_entry(ctx, maybe_new_av_id)
                            .await?;

                    if old_key_or_index == new_key_or_index {
                        new_av_id = Some(maybe_new_av_id);
                        break;
                    }
                }

                let key = old_key_or_index
                    .as_ref()
                    .and_then(|key_or_index| match key_or_index {
                        KeyOrIndex::Key(key) => Some(key.to_owned()),
                        _ => None,
                    });

                match new_av_id {
                    // The value exists in both old and new (thought it might be defaulted)
                    Some(new_av_id) => {
                        attribute_values.push(new_av_id);
                        match old_component_prototype_id {
                            // The old component has an explicit value set rather than using
                            // the default: set the value in the new component as well.
                            Some(old_component_prototype_id) => {
                                let old_prototype_func =
                                    AttributePrototype::func(ctx, old_component_prototype_id)
                                        .await?;
                                if old_prototype_func.is_dynamic() {
                                    // a custom function has been defined for
                                    // this specific component. We have to copy
                                    // this custom prototype over, but we can
                                    // only do so if the inputs to the function
                                    // exist in self after regeneration and have
                                    // the same types.

                                    self.merge_component_specific_dynamic_func_from_other(
                                        ctx,
                                        new_av_id,
                                        old_component_prototype_id,
                                        &new_input_sockets,
                                        &new_output_sockets,
                                        &new_props,
                                        key.clone(),
                                    )
                                    .await?;

                                    // We continue here since we don't want to descend below a dynamic func
                                    continue;
                                } else {
                                    // Ok, the original component has a
                                    // component specific prototype here, but
                                    // it's not a dynamic function. Just set the
                                    // value. This means either it's a simple
                                    // scalar that has had a value set manually,
                                    // *OR*, it's a value set by a dynamic
                                    // function that has been overriden by the
                                    // user, manually, either way, we want to
                                    // just set the value
                                    let old_value = old_av.value(ctx).await?;
                                    AttributeValue::set_value(ctx, new_av_id, old_value).await?;
                                }
                            }
                            // The old component was using the default value. The new component
                            // should do the same, so there's not much to do.
                            None => {
                                // The only exception is values that change the meaning or
                                // validity of other components and connections the user may
                                // have created. In these cases, we want to preserve the old
                                // value to prevent the user's work from being invalidated.
                                //
                                // For example, if root/si/type is changed from Frame to
                                // Component, and the user had already added child components,
                                // those child components would now be in an invalid place
                                // (because Components can't have children).
                                //
                                // For properties like this, we check whether the value has
                                // changed between the old and new schema variant, and if so,
                                // we explicitly set the value on the component to the old
                                // value it used to have, as if the user had explicitly set it
                                // themselves. (You could argue they basically implicitly set
                                // the value of root/si/type when they created the child
                                // components.)
                                if prop_path == ["root", "si", "type"] {
                                    let old_value = Prop::default_value(ctx, old_prop_id).await?;
                                    let new_value = Prop::default_value(ctx, new_prop_id).await?;
                                    if old_value != new_value {
                                        error!("root/si/type different");
                                        // Even if the value was set to a dynamic function, we want
                                        // to freeze it now.
                                        AttributeValue::set_value(ctx, new_av_id, old_value)
                                            .await?;
                                    }
                                }

                                // But we do need to see if this value is set dynamically. If
                                // it is, we don't want to descend, since the tree underneath
                                // it is completely controlled by the dynamic func.
                                let new_prototype_for_value =
                                    AttributeValue::prototype_id(ctx, new_av_id).await?;
                                let new_prototype_func =
                                    AttributePrototype::func(ctx, new_prototype_for_value).await?;
                                if new_prototype_func.is_dynamic() {
                                    continue;
                                }
                            }
                        }
                    }
                    // The new schema variant never had the value. If it's an array or map
                    // element, we need to insert it.
                    None => {
                        if old_key_or_index.is_some() {
                            if let Some(new_parent_id) = new_parent_id {
                                if let Some(old_component_prototype_id) = old_component_prototype_id
                                {
                                    let prototype_func = Func::get_by_id_or_error(
                                        ctx,
                                        AttributePrototype::func_id(
                                            ctx,
                                            old_component_prototype_id,
                                        )
                                        .await?,
                                    )
                                    .await?;
                                    // Insert this value
                                    let inserted_value = AttributeValue::new(
                                        ctx,
                                        new_prop_id,
                                        Some(self.id),
                                        Some(new_parent_id),
                                        key.clone(),
                                    )
                                    .await?;
                                    if prototype_func.is_dynamic() {
                                        self.merge_component_specific_dynamic_func_from_other(
                                            ctx,
                                            inserted_value.id,
                                            old_component_prototype_id,
                                            &new_input_sockets,
                                            &new_output_sockets,
                                            &new_props,
                                            key.clone(),
                                        )
                                        .await?;

                                        continue;
                                    } else {
                                        let old_value = old_av.value(ctx).await?;
                                        AttributeValue::set_value(
                                            ctx,
                                            inserted_value.id,
                                            old_value,
                                        )
                                        .await?;
                                        attribute_values.push(inserted_value.id);
                                    }
                                }
                            }
                        }
                    }
                }

                for old_child_av_id in
                    AttributeValue::get_child_av_ids_in_order(ctx, old_av_id).await?
                {
                    let old_key_or_index =
                        AttributeValue::get_index_or_key_of_child_entry(ctx, old_child_av_id)
                            .await?;
                    value_q.push_back((old_child_av_id, old_key_or_index, new_av_id));
                }
            }
        }

        let component_graph = DependentValueGraph::new(ctx, attribute_values).await?;
        let leaf_value_ids = component_graph.independent_values();
        ctx.add_dependent_values_and_enqueue(leaf_value_ids).await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn merge_component_specific_dynamic_func_from_other(
        &self,
        ctx: &DalContext,
        new_attribute_value_id: AttributeValueId,
        old_component_prototype_id: AttributePrototypeId,
        self_input_sockets: &HashMap<String, InputSocketId>,
        self_output_sockets: &HashMap<String, OutputSocketId>,
        self_props: &HashMap<Vec<String>, PropId>,
        key: Option<String>,
    ) -> ComponentResult<()> {
        let apa_ids =
            AttributePrototypeArgument::list_ids_for_prototype(ctx, old_component_prototype_id)
                .await?;

        let component_prototype_func =
            AttributePrototype::func(ctx, old_component_prototype_id).await?;
        if !component_prototype_func.is_dynamic() {
            return Ok(());
        }

        let mut new_value_sources = vec![];

        for apa_id in &apa_ids {
            let apa = AttributePrototypeArgument::get_by_id(ctx, *apa_id).await?;

            let func_arg = apa.func_argument(ctx).await?;

            if let Some(source) = apa.value_source(ctx).await? {
                match source {
                    ValueSource::InputSocket(input_socket_id) => {
                        // find matching input socket in self
                        let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
                        match self_input_sockets.get(input_socket.name()) {
                            Some(self_input_socket_id) => new_value_sources.push((
                                func_arg.id,
                                ValueSource::InputSocket(*self_input_socket_id),
                            )),
                            None => {
                                // XXX: This means that the dynamic function
                                // XXX: here has an input that no longer exists, so
                                // XXX: we can't copy the function over.
                                // XXX: what should we do here? Warn the user?
                                return Ok(());
                            }
                        }
                    }
                    ValueSource::OutputSocket(output_socket_id) => {
                        let output_socket = OutputSocket::get_by_id(ctx, output_socket_id).await?;
                        match self_output_sockets.get(output_socket.name()) {
                            Some(self_output_socket_id) => new_value_sources.push((
                                func_arg.id,
                                ValueSource::OutputSocket(*self_output_socket_id),
                            )),
                            None => {
                                return Ok(());
                            }
                        }
                    }
                    ValueSource::Prop(prop_id) => {
                        let path = Prop::path_by_id(ctx, prop_id).await?.as_owned_parts();
                        match self_props.get(&path) {
                            Some(self_prop_id) => new_value_sources
                                .push((func_arg.id, ValueSource::Prop(*self_prop_id))),
                            None => {
                                return Ok(());
                            }
                        }
                    }
                    ValueSource::Secret(_) | ValueSource::StaticArgumentValue(_) => {
                        // Should we determine if this secret is still compatible?
                        new_value_sources.push((func_arg.id, source));
                    }
                }
            }
        }

        // All inputs are valid, create the component specific override
        let new_prototype = AttributePrototype::new(ctx, component_prototype_func.id).await?;
        for (func_arg_id, value_source) in new_value_sources {
            let new_apa =
                AttributePrototypeArgument::new(ctx, new_prototype.id, func_arg_id).await?;
            AttributePrototypeArgument::set_value_source(ctx, new_apa.id(), value_source).await?;
        }

        AttributeValue::set_component_prototype_id(
            ctx,
            new_attribute_value_id,
            new_prototype.id,
            key,
        )
        .await?;

        Ok(())
    }

    /// Copy all the attribute values from old_component_id into this
    /// component. Components must be on the same schema variant. This will
    /// preserve any component specific prototypes defined on the component
    /// being copied from.
    pub async fn clone_attributes_from(
        &self,
        ctx: &DalContext,
        old_component_id: ComponentId,
    ) -> ComponentResult<()> {
        let old_sv_id = Component::schema_variant_id(ctx, old_component_id).await?;
        let new_sv_id = Component::schema_variant_id(ctx, self.id).await?;

        if old_sv_id != new_sv_id {
            return Err(ComponentError::CannotCloneFromDifferentVariants);
        }

        // Paste attribute value "values" from original component (or create them for maps/arrays)
        //
        // We could make this more efficient by skipping everything set by non builtins (si:setString, si:setObject, etc), since everything that is propagated will be re-propagated
        let old_root_id = Component::root_attribute_value_id(ctx, old_component_id).await?;
        let new_root_id = Component::root_attribute_value_id(ctx, self.id).await?;
        let mut work_queue = VecDeque::from([(old_root_id, new_root_id)]);
        // Paste attribute prototypes
        // - either updates component prototype to a copy of the original component
        // - or removes component prototype, restoring the schema one (needed because of manual update from the block above)        while
        while let Some((old_av_id, new_av_id)) = work_queue.pop_front() {
            // If the old component has a value (prototype), copy it over
            if let Some(old_prototype_id) =
                AttributeValue::component_prototype_id(ctx, old_av_id).await?
            {
                let old_func_id = AttributePrototype::func_id(ctx, old_prototype_id).await?;
                let new_prototype = AttributePrototype::new(ctx, old_func_id).await?;

                for old_apa_id in
                    AttributePrototypeArgument::list_ids_for_prototype(ctx, old_prototype_id)
                        .await?
                {
                    let old_func_arg_id =
                        AttributePrototypeArgument::func_argument_id_by_id(ctx, old_apa_id).await?;
                    let old_value_source =
                        AttributePrototypeArgument::value_source_by_id(ctx, old_apa_id)
                            .await?
                            .ok_or(ComponentError::MissingAttributePrototypeArgumentSource(
                                old_apa_id,
                            ))?;

                    let apa =
                        AttributePrototypeArgument::new(ctx, new_prototype.id(), old_func_arg_id)
                            .await?;
                    AttributePrototypeArgument::set_value_source(ctx, apa.id(), old_value_source)
                        .await?;
                }

                AttributeValue::set_component_prototype_id(ctx, new_av_id, new_prototype.id, None)
                    .await?;

                let sources = AttributePrototype::input_sources(ctx, new_prototype.id).await?;
                for source in sources {
                    match source {
                        AttributePrototypeSource::AttributeValue(_, _) => {
                            continue;
                        }
                        AttributePrototypeSource::Prop(prop_id, key) => {
                            Prop::add_edge_to_attribute_prototype(
                                ctx,
                                prop_id,
                                new_prototype.id,
                                EdgeWeightKind::Prototype(key),
                            )
                            .await?;
                        }
                        AttributePrototypeSource::InputSocket(socket_id, key) => {
                            InputSocket::add_edge_to_attribute_prototype(
                                ctx,
                                socket_id,
                                new_prototype.id,
                                EdgeWeightKind::Prototype(key),
                            )
                            .await?;
                        }
                        AttributePrototypeSource::OutputSocket(socket_id, key) => {
                            OutputSocket::add_edge_to_attribute_prototype(
                                ctx,
                                socket_id,
                                new_prototype.id,
                                EdgeWeightKind::Prototype(key),
                            )
                            .await?;
                        }
                    }
                }
            } else if let Some(existing_prototype_id) =
                AttributeValue::component_prototype_id(ctx, new_av_id).await?
            {
                AttributePrototype::remove(ctx, existing_prototype_id).await?;
            }

            // Get children, possibly creating new ones if we don't have them yet
            for child_pair in
                AttributeValue::get_child_av_id_pairs_in_order(ctx, old_av_id, new_av_id).await?
            {
                match child_pair {
                    ChildAttributeValuePair::Both(_, old_child_av_id, new_child_av_id) => {
                        work_queue.push_back((old_child_av_id, new_child_av_id));
                    }
                    // If the child is only in the copied component, we create a new one for
                    // ourselves
                    ChildAttributeValuePair::FirstOnly(key, old_child_av_id) => {
                        let new_child_av_id = AttributeValue::new(
                            ctx,
                            AttributeValue::is_for(ctx, old_child_av_id).await?,
                            Some(self.id),
                            Some(new_av_id),
                            key,
                        )
                        .await?
                        .id;
                        work_queue.push_back((old_child_av_id, new_child_av_id));
                    }
                    // TODO this case wasn't handled before, and shouldn't really be possible ...
                    ChildAttributeValuePair::SecondOnly(..) => {
                        continue;
                    }
                }
            }
        }

        self.clear_resource(ctx).await?;
        self.set_name(ctx, &Self::generate_copy_name(self.name(ctx).await?))
            .await?;

        Ok(())
    }

    pub async fn outgoing_connections_for_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<OutgoingConnection>> {
        let mut outgoing_edges = vec![];

        for from_output_socket in
            ComponentOutputSocket::list_for_component_id(ctx, component_id).await?
        {
            for apa_id in AttributePrototypeArgument::list_ids_for_output_socket_and_source(
                ctx,
                from_output_socket.output_socket_id,
                component_id,
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
                            from_output_socket_id: from_output_socket.output_socket_id,
                            created_info,
                            deleted_info: None,
                        });
                    }
                }
            }
        }

        Ok(outgoing_edges)
    }

    pub async fn outgoing_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<OutgoingConnection>> {
        Self::outgoing_connections_for_id(ctx, self.id).await
    }

    /// Calls [`Self::incoming_connections_by_id`] by passing in the id from [`self`](Component).
    pub async fn incoming_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<IncomingConnection>> {
        Self::incoming_connections_for_id(ctx, self.id).await
    }

    /// Finds all incoming connections for explicit edges (i.e. those coming from
    /// [`Components`](ComponentType::Component) and not from frames.
    pub async fn incoming_connections_for_id(
        ctx: &DalContext,
        id: ComponentId,
    ) -> ComponentResult<Vec<IncomingConnection>> {
        let mut incoming_edges = vec![];

        for component_input_socket in ComponentInputSocket::list_for_component_id(ctx, id).await? {
            let prototype_id =
                AttributeValue::prototype_id(ctx, component_input_socket.attribute_value_id)
                    .await?;
            for apa_id in AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                ctx,
                prototype_id,
                id,
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
                            to_input_socket_id: component_input_socket.input_socket_id,
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

    async fn try_get_node_weight_and_content(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<(ComponentNodeWeight, ComponentContentV1)>> {
        if let Some((component_node_weight, content_hash)) =
            Self::try_get_node_weight_and_content_hash(ctx, component_id).await?
        {
            let content: ComponentContent = ctx
                .layer_db()
                .cas()
                .try_read_as(&content_hash)
                .await?
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    component_id.into(),
                ))?;

            let ComponentContent::V1(inner) = content;

            return Ok(Some((component_node_weight, inner)));
        }

        Ok(None)
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ComponentNodeWeight, ComponentContentV1)> {
        Self::try_get_node_weight_and_content(ctx, component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))
    }

    async fn try_get_node_weight_and_content_hash(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<(ComponentNodeWeight, ContentHash)>> {
        let id: Ulid = component_id.into();
        if let Some(node_index) = ctx.workspace_snapshot()?.get_node_index_by_id_opt(id).await {
            let node_weight = ctx
                .workspace_snapshot()?
                .get_node_weight(node_index)
                .await?;

            let hash = node_weight.content_hash();
            let component_node_weight = node_weight.get_component_node_weight()?;
            return Ok(Some((component_node_weight, hash)));
        }

        Ok(None)
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

    pub async fn list_to_be_deleted(ctx: &DalContext) -> ComponentResult<Vec<ComponentId>> {
        let mut to_be_deleted = vec![];
        let components = Self::list(ctx).await?;
        for component in components {
            if component.to_delete {
                to_be_deleted.push(component.id());
            }
        }
        Ok(to_be_deleted)
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
        Ok(SchemaVariant::get_by_id_or_error(ctx, schema_variant_id).await?)
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

    pub async fn try_get_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<Self>> {
        if let Some((node_weight, content)) =
            Self::try_get_node_weight_and_content(ctx, component_id).await?
        {
            return Ok(Some(Self::assemble(&node_weight, content)));
        }

        Ok(None)
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
                .update_content(id.into(), hash)
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

        let av = AttributeValue::get_by_id_or_error(ctx, value_id).await?;

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

        let name_av = AttributeValue::get_by_id_or_error(ctx, name_value_id).await?;

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

        let color_av = AttributeValue::get_by_id_or_error(ctx, color_value_id).await?;

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
        let type_value = AttributeValue::get_by_id_or_error(ctx, type_value_id)
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
    pub async fn output_socket_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        ComponentOutputSocket::attribute_values_for_component_id(ctx, self.id()).await
    }
    pub async fn input_socket_attribute_values(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        ComponentInputSocket::attribute_values_for_component_id(ctx, self.id()).await
    }

    /// Find the attribute values for *this* component and a given prop path
    pub async fn attribute_values_for_prop(
        &self,
        ctx: &DalContext,
        prop_path: &[&str],
    ) -> ComponentResult<Vec<AttributeValueId>> {
        Self::attribute_values_for_prop_by_id(ctx, self.id(), prop_path).await
    }

    /// Find the attribute values for a component id and prop path
    pub async fn attribute_values_for_prop_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
        prop_path: &[&str],
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;

        let prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(prop_path)).await?;

        Self::attribute_values_for_prop_id(ctx, component_id, prop_id).await
    }

    /// Find the attribute values for a component id and prop id
    pub async fn attribute_values_for_prop_id(
        ctx: &DalContext,
        component_id: ComponentId,
        prop_id: PropId,
    ) -> ComponentResult<Vec<AttributeValueId>> {
        let mut result = vec![];
        for attribute_value_id in
            Prop::all_attribute_values_everywhere_for_prop_id(ctx, prop_id).await?
        {
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

    async fn attribute_values_for_all_sockets(
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

    #[instrument(level = "debug", skip(ctx))]
    async fn create_new_connection(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
        destination_prototype_id: AttributePrototypeId,
    ) -> ComponentResult<AttributePrototypeArgumentId> {
        debug!(
            "Creating new Component connection: {:?}, {:?}, {:?}, {:?}",
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id
        );
        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;

        let attribute_prototype_argument = AttributePrototypeArgument::new_inter_component(
            ctx,
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_prototype_id,
        )
        .await?;

        drop(cycle_check_guard);
        debug!("Cycle Check Guard dropped");

        Ok(attribute_prototype_argument.id())
    }

    pub async fn remove_edge_from_frame(
        ctx: &DalContext,
        parent_id: ComponentId,
        child_id: ComponentId,
    ) -> ComponentResult<()> {
        ctx.workspace_snapshot()?
            .remove_edge_for_ulids(
                parent_id,
                child_id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?;

        Ok(())
    }

    #[instrument(level = "info", skip(ctx))]
    pub async fn connect(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<AttributePrototypeArgumentId>> {
        let total_start = std::time::Instant::now();
        // Make sure both source & destination Components exist in the "current" change set.
        // Eventually, this should probably be reported as an error actionable by the frontend, but
        // for now, this is a no-op so we don't end up creating a broken graph.
        let (_source_component, destination_component) = match (
            Component::try_get_by_id(ctx, source_component_id).await?,
            Component::try_get_by_id(ctx, destination_component_id).await?,
        ) {
            (Some(source), Some(destination)) => (source, destination),
            (source, destination) => {
                warn!(
                    "Not creating edge; either source or destination component does not exist in current change set: {:?}, {:?}",
                    source,
                    destination,
                );
                return Ok(None);
            }
        };
        // Already have this connection? Nothing to do.
        for connection in destination_component.incoming_connections(ctx).await? {
            if connection.from_component_id == source_component_id
                && connection.from_output_socket_id == source_output_socket_id
                && connection.to_component_id == destination_component_id
                && connection.to_input_socket_id == destination_input_socket_id
            {
                warn!(
                    "Not creating edge; already have this connection in change set: {:?}",
                    connection
                );
                return Ok(None);
            }
        }

        // filter the value ids by destination_component_id
        let destination_attribute_value_id =
            InputSocket::component_attribute_value_for_input_socket_id(
                ctx,
                destination_input_socket_id,
                destination_component_id,
            )
            .await?;

        let destination_prototype_id =
            AttributeValue::prototype_id(ctx, destination_attribute_value_id).await?;

        Self::connect_arity_cleanup(
            ctx,
            destination_component_id,
            destination_input_socket_id,
            destination_prototype_id,
        )
        .await?;

        let attribute_prototype_argument_id = match Self::restore_connection_from_base_change_set(
            ctx,
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
        )
        .await?
        {
            Some(apa_id) => apa_id,
            None => {
                Self::create_new_connection(
                    ctx,
                    source_component_id,
                    source_output_socket_id,
                    destination_component_id,
                    destination_input_socket_id,
                    destination_prototype_id,
                )
                .await?
            }
        };

        ctx.add_dependent_values_and_enqueue(vec![destination_attribute_value_id])
            .await?;

        debug!("Component::connect took {:?}", total_start.elapsed());

        Ok(Some(attribute_prototype_argument_id))
    }

    /// Check for socket arity on the input socket; if the input socket has arity of
    /// one, and there's an existing edge, need to remove it before we can add a new one.
    #[instrument(level = "debug", skip(ctx))]
    async fn connect_arity_cleanup(
        ctx: &DalContext,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
        destination_prototype_id: AttributePrototypeId,
    ) -> ComponentResult<()> {
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

        Ok(())
    }

    #[instrument(level = "debug", skip(ctx))]
    async fn restore_connection_from_base_change_set(
        ctx: &DalContext,
        source_component_id: ComponentId,
        source_output_socket_id: OutputSocketId,
        destination_component_id: ComponentId,
        destination_input_socket_id: InputSocketId,
    ) -> ComponentResult<Option<AttributePrototypeArgumentId>> {
        debug!(
            "Restoring connection from base change set: {:?}, {:?}, {:?}, {:?}",
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id
        );
        let base_change_set_ctx = ctx.clone_with_base().await?;
        let base_change_set_ctx = &base_change_set_ctx;
        let base_change_set_component = if let Some(component) =
            Component::try_get_by_id(base_change_set_ctx, destination_component_id).await?
        {
            component
        } else {
            return Ok(None);
        };
        let base_change_set_connection = match base_change_set_component
            .incoming_connections(base_change_set_ctx)
            .await?
            .iter()
            .find(|ic| {
                ic.from_component_id == source_component_id
                    && ic.from_output_socket_id == source_output_socket_id
                    && ic.to_component_id == destination_component_id
                    && ic.to_input_socket_id == destination_input_socket_id
            })
            .cloned()
        {
            Some(connection) => connection,
            None => return Ok(None),
        };
        debug!(
            "Restoring connection from base change set: {:?}, {:?}, {:?}, {:?}, {:?}",
            source_component_id,
            source_output_socket_id,
            destination_component_id,
            destination_input_socket_id,
            base_change_set_connection,
        );

        let base_attribute_prototype_argument_node_index = base_change_set_ctx
            .workspace_snapshot()?
            .get_node_index_by_id(base_change_set_connection.attribute_prototype_argument_id)
            .await?;
        let base_attribute_prototype_argument_node_weight = base_change_set_ctx
            .workspace_snapshot()?
            .get_node_weight(base_attribute_prototype_argument_node_index)
            .await?;
        let base_func_arg_id = AttributePrototypeArgument::func_argument_id_by_id(
            base_change_set_ctx,
            base_change_set_connection.attribute_prototype_argument_id,
        )
        .await?;

        // We want to recreate the `AttributePrototype -> AttributePrototypeArgument` edge as it
        // exists in the base change set.
        let mut base_prototype_argument_incoming_edges = base_change_set_ctx
            .workspace_snapshot()?
            .edges_directed(
                base_change_set_connection.attribute_prototype_argument_id,
                petgraph::Direction::Incoming,
            )
            .await?;
        base_prototype_argument_incoming_edges.retain(
            |(edge_weight, _source_index, _destination_index)| {
                EdgeWeightKindDiscriminants::PrototypeArgument == edge_weight.kind().into()
            },
        );
        let (base_prototype_to_argument_edge_weight, prototype_id) =
            if base_prototype_argument_incoming_edges.len() == 1 {
                match base_prototype_argument_incoming_edges.first() {
                    Some((edge_weight, source_node_index, _destination_node_index)) => {
                        let prototype_weight = base_change_set_ctx
                            .workspace_snapshot()?
                            .get_node_weight(*source_node_index)
                            .await?;
                        (edge_weight, prototype_weight.id())
                    }
                    None => {
                        // We just made sure that there was exactly one element in the Vec.
                        unreachable!("Unable to get first element of a one element Vec");
                    }
                }
            } else {
                return Err(
                    ComponentError::WrongNumberOfPrototypesForAttributePrototypeArgument(
                        base_change_set_connection.attribute_prototype_argument_id,
                    ),
                );
            };

        // We want to recreate the `AttributePrototypeArgument -> OutputSocket` edge
        // (EdgeWeightKind::PrototypeArgumentValue).
        let base_prototype_arg_to_output_socket_edges = base_change_set_ctx
            .workspace_snapshot()?
            .get_edges_between_nodes(
                base_change_set_connection
                    .attribute_prototype_argument_id
                    .into(),
                source_output_socket_id.into(),
            )
            .await?;
        let base_prototype_arg_to_output_socket_edge_weight =
            match base_prototype_arg_to_output_socket_edges.first() {
                Some(edge_weight) => edge_weight,
                None => {
                    return Err(AttributePrototypeArgumentError::MissingSource(
                        base_change_set_connection.attribute_prototype_argument_id,
                    )
                    .into());
                }
            };

        // We want to recreate the `AttributePrototypeArgument -> FuncArg` edge
        // (EdgeWeightKind::Use).
        let base_prototype_arg_to_func_arg_edges = base_change_set_ctx
            .workspace_snapshot()?
            .get_edges_between_nodes(
                base_change_set_connection
                    .attribute_prototype_argument_id
                    .into(),
                base_func_arg_id.into(),
            )
            .await?;
        let base_prototype_arg_to_func_arg_edge_weight =
            match base_prototype_arg_to_func_arg_edges.first() {
                Some(edge_weight) => edge_weight,
                None => {
                    return Err(AttributePrototypeArgumentError::MissingFuncArgument(
                        base_change_set_connection.attribute_prototype_argument_id,
                    )
                    .into())
                }
            };

        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;

        // Recreate the AttributePrototypeArgument & associated edges.
        // We only need to import the AttributePrototypeArgument node, as all of the other relevant
        // nodes should already exist.
        ctx.workspace_snapshot()?
            .add_node(base_attribute_prototype_argument_node_weight.clone())
            .await?;
        ctx.workspace_snapshot()?
            .add_edge(
                prototype_id,
                base_prototype_to_argument_edge_weight.clone(),
                base_change_set_connection.attribute_prototype_argument_id,
            )
            .await?;
        ctx.workspace_snapshot()?
            .add_edge(
                base_change_set_connection.attribute_prototype_argument_id,
                base_prototype_arg_to_func_arg_edge_weight.clone(),
                base_func_arg_id,
            )
            .await?;
        ctx.workspace_snapshot()?
            .add_edge(
                base_change_set_connection.attribute_prototype_argument_id,
                base_prototype_arg_to_output_socket_edge_weight.clone(),
                source_output_socket_id,
            )
            .await?;

        drop(cycle_check_guard);
        debug!("Cycle Check Guard dropped");

        Ok(Some(
            base_attribute_prototype_argument_node_weight.id().into(),
        ))
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
                AttributeValue::get_child_av_ids_in_order(ctx, av_id)
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
        Ok(
            match (destination_component_is_delete, source_component_is_delete) {
                (None, _) | (_, None) => false,
                (Some(destination_component_is_delete), Some(source_component_is_delete)) => {
                    destination_component_is_delete || !source_component_is_delete
                }
            },
        )
    }

    /// Simply gets the to_delete status for a component via the Node Weight
    async fn is_set_to_delete(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<bool>> {
        match ctx
            .workspace_snapshot()?
            .get_node_index_by_id_opt(component_id)
            .await
        {
            Some(component_idx) => {
                let component_node_weight = ctx
                    .workspace_snapshot()?
                    .get_node_weight(component_idx)
                    .await?
                    .get_component_node_weight()?;
                Ok(Some(component_node_weight.to_delete()))
            }
            None => Ok(None),
        }
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
            let mut new_component_node_weight = component_node_weight.clone();
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
                .update_content(component.id.into(), hash)
                .await?;
        }

        let component_node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight_by_id(original_component.id)
            .await?
            .get_component_node_weight()?;

        Ok(Component::assemble(&component_node_weight, updated))
    }

    #[instrument(level = "info", skip(ctx))]
    pub async fn remove(ctx: &DalContext, id: ComponentId) -> ComponentResult<()> {
        let component = Self::get_by_id(ctx, id).await?;

        if component.parent(ctx).await?.is_some() {
            Frame::orphan_child(ctx, id)
                .await
                .map_err(|e| ComponentError::Frame(Box::new(e)))?;
        }

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
        for component_output_socket in ComponentOutputSocket::list_for_component_id(ctx, id).await?
        {
            let output_socket =
                OutputSocket::get_by_id(ctx, component_output_socket.output_socket_id).await?;
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

        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;

        WsEvent::component_deleted(ctx, id)
            .await?
            .publish_on_commit(ctx)
            .await?;

        Ok(())
    }

    /// A [`Component`] is allowed to be removed from the graph if it meets the following
    /// requirements:
    ///
    /// 1. It doesn't have a populated resource.
    /// 2. It is not feeding data to a [`Component`] that has a populated resource.
    #[instrument(level = "debug", skip_all)]
    pub async fn allowed_to_be_removed(&self, ctx: &DalContext) -> ComponentResult<bool> {
        if self.resource(ctx).await?.is_some() {
            debug!(
                "component {:?} cannot be removed because it has a resource",
                self.id
            );
            return Ok(false);
        }

        // Check all outgoing connections.
        let outgoing_connections = self.outgoing_connections(ctx).await?;
        for outgoing_connection in outgoing_connections {
            let connected_to_component =
                Self::get_by_id(ctx, outgoing_connection.to_component_id).await?;
            if connected_to_component.resource(ctx).await?.is_some() {
                debug!(
                    "component {:?} cannot be removed because {:?} has resource",
                    self.id,
                    connected_to_component.id()
                );
                return Ok(false);
            }
        }

        // Check all inferred outgoing connections, which accounts for up and down configuration
        // frames alike due to the direction of the connection.
        let inferred_outgoing_connections = self.inferred_outgoing_connections(ctx).await?;
        for inferred_outgoing in inferred_outgoing_connections {
            let connected_to_component =
                Self::get_by_id(ctx, inferred_outgoing.to_component_id).await?;
            if connected_to_component.resource(ctx).await?.is_some() {
                debug!(
                    "component {:?} cannot be removed because {:?} has resource",
                    self.id,
                    connected_to_component.id()
                );
                return Ok(false);
            }
        }

        debug!("component {:?} can be removed", self.id,);
        Ok(true)
    }

    pub async fn delete(self, ctx: &DalContext) -> ComponentResult<Option<Self>> {
        if self.allowed_to_be_removed(ctx).await? {
            Self::remove(ctx, self.id).await?;
            Ok(None)
        } else {
            Ok(Some(self.set_to_delete(ctx, true).await?))
        }
    }

    pub async fn set_to_delete(self, ctx: &DalContext, to_delete: bool) -> ComponentResult<Self> {
        let component_id = self.id;
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;
        let original_to_delete = self.to_delete;

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

        let input_av_ids: Vec<AttributeValueId> =
            modified.input_socket_attribute_values(ctx).await?;

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

        // Deal with deletion actions, but only if we're transitioning from not being to_delete
        // into being to_delete.
        if to_delete && !original_to_delete {
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
        } else if !to_delete {
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

        let output_sockets: Vec<ComponentOutputSocket> =
            ComponentOutputSocket::list_for_component_id(ctx, self.id()).await?;
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
            let inferred_inputs = ComponentOutputSocket::find_inferred_connections(
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

    pub async fn copy_paste(
        &self,
        ctx: &DalContext,
        component_geometry: ComponentGeometry,
    ) -> ComponentResult<Self> {
        let schema_variant = self.schema_variant(ctx).await?;

        let mut pasted_comp = Component::new(
            ctx,
            Self::generate_copy_name(self.name(ctx).await?),
            schema_variant.id(),
        )
        .await?;

        pasted_comp
            .set_geometry(
                ctx,
                component_geometry.x,
                component_geometry.y,
                component_geometry.width,
                component_geometry.height,
            )
            .await?;

        pasted_comp.clone_attributes_from(ctx, self.id()).await?;
        Ok(pasted_comp)
    }

    /// Finds all inferred incoming connections for the [`Component`]
    /// A connection is inferred if it's input socket is being driven
    /// by another component's output socket as a result of lineage
    /// via FrameContains Edges.
    #[instrument(level = "debug", skip(ctx))]
    pub async fn inferred_incoming_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<InferredConnection>> {
        let to_component_id = self.id();
        let mut connections = vec![];

        let incoming_connections = match ctx
            .workspace_snapshot()?
            .get_cached_inferred_connection_graph()
            .await
            .as_ref()
        {
            Some(cached_graph) => {
                cached_graph.get_inferred_incoming_connections_to_component(to_component_id)
            }
            None => InferredConnectionGraph::assemble(ctx, to_component_id)
                .await?
                .get_inferred_incoming_connections_to_component(to_component_id),
        };

        for incoming_connection in incoming_connections {
            // add the check for to_delete on either to or from component
            // Both "deleted" and not deleted Components can feed data into
            // "deleted" Components. **ONLY** not deleted Components can feed
            // data into not deleted Components.
            let destination_component = Self::get_by_id(ctx, to_component_id).await?;
            let source_component =
                Self::get_by_id(ctx, incoming_connection.output_socket.component_id).await?;
            let to_delete = !Self::should_data_flow_between_components(
                ctx,
                destination_component.id,
                source_component.id,
            )
            .await?;

            connections.push(InferredConnection {
                to_component_id,
                to_input_socket_id: incoming_connection.input_socket.input_socket_id,
                from_component_id: incoming_connection.output_socket.component_id,
                from_output_socket_id: incoming_connection.output_socket.output_socket_id,
                to_delete,
            });
        }

        Ok(connections)
    }

    /// Finds all inferred outgoing connections for the [`Component`]. A connection is inferred if
    /// its output sockets are driving another [`Component's`](Component) [`InputSocket`] as a
    /// result of lineage via an [`EdgeWeightKind::FrameContains`] edge.
    #[instrument(level = "info", skip(ctx))]
    pub async fn inferred_outgoing_connections(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<Vec<InferredConnection>> {
        let from_component_id = self.id();
        let mut connections = vec![];

        let outgoing_connections = match ctx
            .workspace_snapshot()?
            .get_cached_inferred_connection_graph()
            .await
            .as_ref()
        {
            Some(cached_graph) => {
                cached_graph.get_inferred_outgoing_connections_for_component(from_component_id)
            }
            None => InferredConnectionGraph::assemble(ctx, from_component_id)
                .await?
                .get_inferred_outgoing_connections_for_component(from_component_id),
        };

        for outgoing_connection in outgoing_connections {
            // add the check for to_delete on either to or from component
            // Both "deleted" and not deleted Components can feed data into
            // "deleted" Components. **ONLY** not deleted Components can feed
            // data into not deleted Components.
            let destination_component = outgoing_connection.input_socket.component_id;
            let source_component = self.id();

            let to_delete = !Self::should_data_flow_between_components(
                ctx,
                destination_component,
                source_component,
            )
            .await?;
            connections.push(InferredConnection {
                to_component_id: outgoing_connection.input_socket.component_id,
                to_input_socket_id: outgoing_connection.input_socket.input_socket_id,
                from_component_id: outgoing_connection.output_socket.component_id,
                from_output_socket_id: outgoing_connection.output_socket.output_socket_id,
                to_delete,
            });
        }
        Ok(connections)
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

                                let destination_attribute_value_id =
                                    InputSocket::component_attribute_value_for_input_socket_id(
                                        ctx,
                                        destination_input_socket_id,
                                        destination_component_id,
                                    )
                                    .await?;

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

        // ================================================================================
        // Cache original component data
        // ================================================================================
        let snap = ctx.workspace_snapshot()?;

        let original_component_node_weight =
            snap.get_node_weight_by_id(original_component.id).await?;

        let original_component_name = self.name(ctx).await?;
        let original_component_id = self.id();
        let original_component_lineage_id = original_component_node_weight.lineage_id();

        let original_incoming_connections = original_component.incoming_connections(ctx).await?;
        let original_outgoing_connections = original_component.outgoing_connections(ctx).await?;

        // ================================================================================
        // Create new component and run changes that depend on the old one still existing
        // ================================================================================
        let new_component_with_temp_id = Component::new_with_content_address(
            ctx,
            original_component_name.clone(),
            schema_variant_id,
            original_component_node_weight.content_hash(),
        )
        .await?;

        let new_schema_variant_id = new_component_with_temp_id.schema_variant(ctx).await?.id();
        if new_schema_variant_id != schema_variant_id {
            return Err(ComponentError::ComponentIncorrectSchemaVariant(
                new_component_with_temp_id.id(),
            ));
        }

        new_component_with_temp_id
            .merge_from_component_with_different_schema_variant(ctx, original_component.id())
            .await?;

        if schema_variant_id
            != Component::get_by_id(ctx, new_component_with_temp_id.id())
                .await?
                .schema_variant(ctx)
                .await?
                .id()
        {
            return Err(ComponentError::ComponentIncorrectSchemaVariant(
                new_component_with_temp_id.id(),
            ));
        }

        // Restore parent connection on new component
        if let Some(parent) = original_component.parent(ctx).await? {
            Frame::upsert_parent(ctx, new_component_with_temp_id.id(), parent)
                .await
                .map_err(Box::new)?;
        }

        // Restore child connections on new component
        for child in Component::get_children_for_id(ctx, original_component_id).await? {
            Frame::upsert_parent(ctx, child, new_component_with_temp_id.id())
                .await
                .map_err(Box::new)?;
        }

        // Remove old component connections
        for incoming in &original_incoming_connections {
            Component::remove_connection(
                ctx,
                incoming.from_component_id,
                incoming.from_output_socket_id,
                incoming.to_component_id,
                incoming.to_input_socket_id,
            )
            .await?;

            WsEvent::connection_deleted(
                ctx,
                incoming.from_component_id,
                incoming.to_component_id,
                incoming.from_output_socket_id,
                incoming.to_input_socket_id,
            )
            .await?
            .publish_on_commit(ctx)
            .await?;
        }

        for outgoing in &original_outgoing_connections {
            Component::remove_connection(
                ctx,
                outgoing.from_component_id,
                outgoing.from_output_socket_id,
                outgoing.to_component_id,
                outgoing.to_input_socket_id,
            )
            .await?;

            WsEvent::connection_deleted(
                ctx,
                outgoing.from_component_id,
                outgoing.to_component_id,
                outgoing.from_output_socket_id,
                outgoing.to_input_socket_id,
            )
            .await?
            .publish_on_commit(ctx)
            .await?;
        }

        // Let's requeue any Actions for the component
        Self::requeue_actions_for_upgraded_component(
            ctx,
            original_component.id(),
            new_component_with_temp_id.id(),
            new_schema_variant_id,
        )
        .await?;

        // ========================================
        // Delete original component
        // ========================================

        // Remove the original resource so that we don't queue a delete action
        original_component.clear_resource(ctx).await?;
        Self::remove(ctx, original_component.id).await?;
        snap.cleanup().await?;

        // ========================================
        // Finish up the new component
        // ========================================

        // Now we replace the new component id with the id of the original one
        snap.update_node_id(
            new_component_with_temp_id.id,
            original_component_id,
            original_component_lineage_id,
        )
        .await?;

        // Re fetch the component with the old id
        let finalized_new_component = Self::get_by_id(ctx, original_component_id).await?;

        // Restore connections on new component
        for incoming in &original_incoming_connections {
            let socket = InputSocket::get_by_id(ctx, incoming.to_input_socket_id).await?;
            if let Some(socket) =
                InputSocket::find_with_name(ctx, socket.name(), schema_variant_id).await?
            {
                Component::connect(
                    ctx,
                    incoming.from_component_id,
                    incoming.from_output_socket_id,
                    finalized_new_component.id(),
                    socket.id(),
                )
                .await?;
            } else {
                debug!(
                    "Unable to reconnect to socket_id: {0} for component_id: {1}",
                    socket.id(),
                    finalized_new_component.id()
                );
            }
        }

        for outgoing in &original_outgoing_connections {
            let socket = OutputSocket::get_by_id(ctx, outgoing.from_output_socket_id).await?;
            if let Some(socket) =
                OutputSocket::find_with_name(ctx, socket.name(), schema_variant_id).await?
            {
                Component::connect(
                    ctx,
                    finalized_new_component.id(),
                    socket.id(),
                    outgoing.to_component_id,
                    outgoing.to_input_socket_id,
                )
                .await?;
            } else {
                debug!(
                    "Unable to reconnect to socket_id: {0} for component_id: {1}",
                    socket.id(),
                    finalized_new_component.id()
                );
            }
        }

        Ok(finalized_new_component)
    }

    async fn requeue_actions_for_upgraded_component(
        ctx: &DalContext,
        old_component_id: ComponentId,
        new_component_id: ComponentId,
        new_schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<()> {
        // Remove any actions created for the new component as a side effect of the upgrade
        // Then loop through the existing queued actions for the old component and re-add them piecemeal.
        Action::remove_all_for_component_id(ctx, new_component_id)
            .await
            .map_err(|err| ComponentError::Action(Box::new(err)))?;

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
                    let func_id = ActionPrototype::func_id(ctx, action_prototype_id)
                        .await
                        .map_err(|err| ComponentError::ActionPrototype(Box::new(err)))?;
                    let queued_func = Func::get_by_id_or_error(ctx, func_id).await?;

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
                            Action::new(
                                ctx,
                                available_action_prototype.id(),
                                Some(new_component_id),
                            )
                            .await
                            .map_err(|err| ComponentError::Action(Box::new(err)))?;
                        }
                    }
                }
                ActionState::Running | ActionState::Dispatched => continue,
            }
        }
        Ok(())
    }

    fn generate_copy_name(name: String) -> String {
        if name.ends_with("- Copy") {
            name
        } else {
            format!("{} - Copy", name)
        }
    }

    /// This method finds the [`AttributeValueId`](crate::AttributeValue) corresponding to either  "/root/code" or
    /// "/root/qualification" for the given [`ComponentId`](Component) and ['LeafKind'](LeafKind).
    pub async fn find_map_attribute_value_for_leaf_kind(
        ctx: &DalContext,
        component_id: ComponentId,
        leaf_kind: LeafKind,
    ) -> ComponentResult<AttributeValueId> {
        let attribute_value_id = match leaf_kind {
            LeafKind::CodeGeneration => {
                Component::find_code_map_attribute_value_id(ctx, component_id).await?
            }
            LeafKind::Qualification => {
                Component::find_qualification_map_attribute_value_id(ctx, component_id).await?
            }
        };
        Ok(attribute_value_id)
    }

    pub async fn restore_from_base_change_set(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<()> {
        let base_change_set_ctx = ctx.clone_with_base().await?;

        ctx.workspace_snapshot()?
            .import_component_subgraph(&*base_change_set_ctx.workspace_snapshot()?, component_id)
            .await?;

        let component = Component::get_by_id(ctx, component_id).await?;

        ctx.add_dependent_values_and_enqueue(component.input_socket_attribute_values(ctx).await?)
            .await?;

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
