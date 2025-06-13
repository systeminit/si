//! This module contains [`Component`], which is an instance of a
//! [`SchemaVariant`](SchemaVariant) and a _model_ of a "real world resource".

use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
        hash_map,
    },
    num::{
        ParseFloatError,
        ParseIntError,
    },
    str::FromStr,
    sync::Arc,
};

use frame::{
    Frame,
    FrameError,
};
use itertools::Itertools;
use petgraph::Direction::Outgoing;
use resource::ResourceData;
use serde::{
    Deserialize,
    Serialize,
};
use si_db::{
    ActorView,
    HistoryEventMetadata,
};
use si_events::{
    ContentHash,
    Timestamp,
    ulid::Ulid,
};
use si_frontend_types::{
    DiagramComponentView,
    DiagramSocket,
    DiagramSocketDirection,
    DiagramSocketNodeSide,
    GeometryAndView,
    RawGeometry,
};
use si_id::SchemaId;
use si_pkg::KeyOrIndex;
use si_split_graph::SplitGraphError;
use socket::{
    ComponentInputSocket,
    ComponentOutputSocket,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;

use self::inferred_connection_graph::InferredConnectionGraphError;
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    AttributeValue,
    AttributeValueId,
    ChangeSetId,
    DalContext,
    EdgeWeight,
    Func,
    FuncError,
    FuncId,
    HelperError,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    Prop,
    PropId,
    PropKind,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    SocketArity,
    TransactionsError,
    WorkspaceError,
    WorkspacePk,
    WsEvent,
    WsEventError,
    WsEventResult,
    WsPayload,
    action::{
        Action,
        ActionError,
        ActionState,
        prototype::{
            ActionKind,
            ActionPrototype,
            ActionPrototypeError,
        },
    },
    attribute::{
        path::AttributePath,
        prototype::{
            AttributePrototypeError,
            AttributePrototypeSource,
            argument::{
                AttributePrototypeArgument,
                AttributePrototypeArgumentError,
                AttributePrototypeArgumentId,
                value_source::ValueSource,
            },
        },
        value::{
            AttributeValueError,
            ChildAttributeValuePair,
            DependentValueGraph,
            ValueIsFor,
            subscription::ValueSubscription,
        },
    },
    change_set::ChangeSetError,
    change_status::ChangeStatus,
    code_view::CodeViewError,
    diagram::{
        DiagramError,
        SummaryDiagramEdge,
        SummaryDiagramInferredEdge,
        SummaryDiagramManagementEdge,
        geometry::Geometry,
        view::{
            View,
            ViewId,
        },
    },
    func::{
        argument::FuncArgumentError,
        binding::FuncBindingError,
    },
    implement_add_edge_to,
    layer_db_types::{
        ComponentContent,
        ComponentContentV2,
    },
    module::{
        Module,
        ModuleError,
    },
    prop::{
        PropError,
        PropPath,
    },
    qualification::{
        QualificationError,
        QualificationSummaryError,
    },
    schema::variant::{
        SchemaVariantError,
        leaves::LeafKind,
        root_prop::component_type::ComponentType,
    },
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    validation::{
        ValidationError,
        ValidationOutput,
    },
    workspace_snapshot::{
        DependentValueRoot,
        WorkspaceSnapshotError,
        content_address::ContentAddressDiscriminants,
        dependent_value_root::DependentValueRootError,
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::WorkspaceSnapshotGraphError,
        node_weight::{
            ComponentNodeWeight,
            NodeWeight,
            NodeWeightError,
            attribute_prototype_argument_node_weight::ArgumentTargets,
            category_node_weight::CategoryNodeKind,
        },
    },
};

pub mod code;
pub mod debug;
pub mod delete;
pub mod diff;
pub mod frame;
pub mod inferred_connection_graph;
pub mod properties;
pub mod qualification;
pub mod resource;
pub mod socket;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ComponentError {
    #[error("action error: {0}")]
    Action(#[from] Box<ActionError>),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
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
    #[error("component {0} already has a geometry for view {1}")]
    ComponentAlreadyInView(ComponentId, ViewId),
    #[error("component has children, cannot change to component type")]
    ComponentHasChildren,
    #[error("component {0} has more than one value for the {1} prop")]
    ComponentHasTooManyValues(ComponentId, PropId),
    #[error("component {0} has an unexpected schema variant id")]
    ComponentIncorrectSchemaVariant(ComponentId),
    #[error("component {0} has no attribute value for the root/si/color prop")]
    ComponentMissingColorValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/name prop")]
    ComponentMissingNameValue(ComponentId),
    #[error("component {0} has no attribute value for the root/resource prop")]
    ComponentMissingResourceValue(ComponentId),
    #[error("component {0} has no attribute value for the root/si/type prop")]
    ComponentMissingTypeValue(ComponentId),
    #[error("component {0} has no materialized view for the root/si/type prop")]
    ComponentMissingTypeValueMaterializedView(ComponentId),
    #[error("component {0} has no attribute value for the {1} prop")]
    ComponentMissingValue(ComponentId, PropId),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("connection destination component {0} has no attribute value for input socket {1}")]
    DestinationComponentMissingAttributeValueForInputSocket(ComponentId, InputSocketId),
    #[error("diagram error: {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("frame error: {0}")]
    Frame(#[from] Box<FrameError>),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgumentError(#[from] FuncArgumentError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] Box<FuncBindingError>),
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("InferredConnectionGraph Error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("input socket {0} not found for component id {1}")]
    InputSocketNotFoundForComponentId(InputSocketId, ComponentId),
    #[error("input socket {0} has more than one attribute value")]
    InputSocketTooManyAttributeValues(InputSocketId),
    #[error("invalid component type update from {0} to {1}")]
    InvalidComponentTypeUpdate(ComponentType, ComponentType),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
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
    #[error("module error: {0}")]
    Module(#[from] ModuleError),
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
    #[error("component not found by name: {0}")]
    NotFoundByName(String),
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
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("found prop id ({0}) that is not a prop")]
    PropIdNotAProp(PropId),
    #[error("qualification error: {0}")]
    Qualification(#[from] QualificationError),
    #[error("ordering node not found for qualifications map {0} and component {1}")]
    QualificationNoOrderingNode(AttributeValueId, ComponentId),
    #[error("qualification summary error: {0}")]
    QualificationSummary(#[from] Box<QualificationSummaryError>),
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
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("split graph error: {0}")]
    SplitGraph(#[from] SplitGraphError),
    #[error(
        "too many explicit connection sources ({0:?}) for component ({1}) and input socket ({2}) with an arity of one"
    )]
    TooManyExplicitConnectionSources(Vec<ComponentId>, ComponentId, InputSocketId),
    #[error(
        "too many inferred connections ({0:?}) for input socket match ({1:?}) with an arity of one"
    )]
    TooManyInferredConnections(Vec<ComponentOutputSocket>, ComponentInputSocket),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("ulid decode error: {0}")]
    Ulid(#[from] ulid::DecodeError),
    #[error(
        "unexpected explicit source ({0}) and inferred source ({1}) for input socket match ({2:?}) with an arity of one"
    )]
    UnexpectedExplicitAndInferredSources(ComponentId, ComponentId, ComponentInputSocket),
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("value source for known prop attribute value {0} is not a prop id")]
    ValueSourceForPropValueNotPropId(AttributeValueId),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace pk not found on context")]
    WorkspacePkNone,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("workspace snapshot graph error: {0}")]
    WorkspaceSnapshotGraphError(#[from] WorkspaceSnapshotGraphError),
    #[error("attribute value {0} has wrong type for operation: {0}")]
    WrongAttributeValueType(AttributeValueId, ValueIsFor),
    #[error("Attribute Prototype Argument used by too many Attribute Prototypes: {0}")]
    WrongNumberOfPrototypesForAttributePrototypeArgument(AttributePrototypeArgumentId),
    #[error("WsEvent error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl From<ActionError> for ComponentError {
    fn from(err: ActionError) -> Self {
        Box::new(err).into()
    }
}
impl From<ActionPrototypeError> for ComponentError {
    fn from(err: ActionPrototypeError) -> Self {
        Box::new(err).into()
    }
}
impl From<DiagramError> for ComponentError {
    fn from(err: DiagramError) -> Self {
        Box::new(err).into()
    }
}
impl From<FrameError> for ComponentError {
    fn from(err: FrameError) -> Self {
        Box::new(err).into()
    }
}
impl From<FuncBindingError> for ComponentError {
    fn from(err: FuncBindingError) -> Self {
        Box::new(err).into()
    }
}

pub type ComponentResult<T> = Result<T, ComponentError>;

pub use si_id::ComponentId;

#[derive(Clone, Debug)]
pub struct Connection {
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
}

impl From<Component> for ComponentContentV2 {
    fn from(value: Component) -> Self {
        Self {
            timestamp: value.timestamp,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ControllingFuncData {
    pub func_id: FuncId,
    pub av_id: AttributeValueId,
    pub is_dynamic_func: bool,
}

impl Component {
    pub fn assemble(node_weight: &ComponentNodeWeight, content: ComponentContentV2) -> Self {
        Self {
            id: node_weight.id().into(),
            timestamp: content.timestamp,
            to_delete: node_weight.to_delete(),
        }
    }

    pub fn id(&self) -> ComponentId {
        self.id
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub async fn change_status(&self, ctx: &DalContext) -> ComponentResult<ChangeStatus> {
        let status = if self.exists_in_head(ctx).await? {
            if self.to_delete() {
                ChangeStatus::Deleted
            } else {
                ChangeStatus::Unmodified
            }
        } else {
            ChangeStatus::Added
        };

        Ok(status)
    }

    pub async fn exists_in_head(&self, ctx: &DalContext) -> ComponentResult<bool> {
        let head_ctx = ctx.clone_with_head().await?;

        Ok(Self::try_get_by_id(&head_ctx, self.id).await?.is_some())
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
    implement_add_edge_to!(
        source_id: ComponentId,
        destination_id: ComponentId,
        add_fn: add_manages_edge_to_component,
        discriminant: EdgeWeightKindDiscriminants::Manages,
        result: ComponentResult,
    );

    #[instrument(
        name = "component.new",
        level = "info",
        skip_all,
        fields(
            schema_variant.id = ?schema_variant_id
        ))]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
        view_id: ViewId,
    ) -> ComponentResult<Self> {
        let content = ComponentContentV2 {
            timestamp: Timestamp::now(),
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ComponentContent::V2(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let component =
            Self::new_with_content_address_and_no_geometry(ctx, name, schema_variant_id, hash)
                .await?;

        // Create geometry node
        Geometry::new_for_component(ctx, component.id, view_id).await?;

        Ok(component)
    }

    /// Create new component node but retain existing content address
    /// This is used to create the replacement nodes on upgrade, so geometries for it need
    /// to be created by hand. Anywhere else you want to use [Self::new](Self::new)
    pub async fn new_with_content_address_and_no_geometry(
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
        workspace_snapshot.add_or_replace_node(node_weight).await?;

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

        // Create attribute values for all socket corresponding to input and output sockets.
        for input_socket_id in
            InputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, input_socket_id, Some(id.into()), None, None).await?;

            DependentValueRoot::add_dependent_value_root(
                ctx,
                DependentValueRoot::Unfinished(attribute_value.id().into()),
            )
            .await?;
        }
        for output_socket_id in
            OutputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, output_socket_id, Some(id.into()), None, None).await?;

            DependentValueRoot::add_dependent_value_root(
                ctx,
                DependentValueRoot::Unfinished(attribute_value.id().into()),
            )
            .await?;
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
                .get_node_weight(prop_id)
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
            if ValidationOutput::get_format_for_attribute_value_id(ctx, attribute_value.id())
                .await?
                .is_some()
            {
                ctx.enqueue_compute_validations(attribute_value.id())
                    .await?;
            }

            if should_descend {
                match prop_kind {
                    PropKind::Object => {
                        let ordered_children = workspace_snapshot
                            .ordered_children_for_node(prop_id)
                            .await?
                            .ok_or(ComponentError::ObjectPropHasNoOrderingNode(prop_id))?;

                        for child_prop_id in ordered_children {
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
                    // We want to only add leaves to the DVU roots
                    _ => {
                        DependentValueRoot::add_dependent_value_root(
                            ctx,
                            DependentValueRoot::Unfinished(attribute_value.id().into()),
                        )
                        .await?;
                    }
                }
            } else {
                DependentValueRoot::add_dependent_value_root(
                    ctx,
                    DependentValueRoot::Unfinished(attribute_value.id().into()),
                )
                .await?;
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

        //set a component specific prototype for the root/si/type as we don't want it to ever change other than manually
        let sv_type = SchemaVariant::get_by_id(ctx, schema_variant_id)
            .await?
            .get_type(ctx)
            .await?;
        if let Some(sv_type) = sv_type {
            component.set_type(ctx, sv_type).await?;
        }

        // Find all create action prototypes for the variant and create actions for them.
        for prototype_id in SchemaVariant::find_action_prototypes_by_kind(
            ctx,
            schema_variant_id,
            ActionKind::Create,
        )
        .await?
        {
            Action::new(ctx, prototype_id, Some(component.id)).await?;
        }

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
        let mut dvu_roots = vec![];

        // Gather a bunch of data about the current schema variant
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
            let old_av = AttributeValue::get_by_id(ctx, old_av_id).await?;

            let maybe_old_component_prototype_id =
                AttributeValue::component_prototype_id(ctx, old_av_id).await?;
            let old_prop_id = AttributeValue::is_for(ctx, old_av_id)
                .await?
                .prop_id()
                .ok_or(ComponentError::ValueSourceForPropValueNotPropId(old_av_id))?;

            let prop_path = Prop::path_by_id(ctx, old_prop_id).await?.as_owned_parts();

            // Is there a matching prop in self for this prop in other? If there
            // is no matching prop do nothing (this means the prop was removed
            // from self, so can't get values from other)
            let Some(&new_prop_id) = new_props.get(&prop_path) else {
                continue;
            };

            let new_prop = Prop::get_by_id(ctx, new_prop_id).await?;
            let old_prop = Prop::get_by_id(ctx, old_prop_id).await?;

            // Prop kinds could have changed for the same prop. We could
            // try and coerce values, but it's safer to just skip.  Even if
            // there is a component specific prototype for this prop's value
            // in other, we don't want to copy it over, since the kind has
            // changed.
            if new_prop.kind != old_prop.kind {
                continue;
            }

            // Similarly, we should verify that the secret kind has not
            // changed if this is a secret prop. If it has changed, leave
            // the prop alone (effectively emptying the secret)
            if new_prop.secret_kind_widget_option() != old_prop.secret_kind_widget_option() {
                continue;
            }

            // If there is another av for this prop with the same path, get that to populate later
            let maybe_new_av_id = {
                let old_av_path = AttributeValue::get_path_for_id(ctx, old_av_id).await?;
                let mut new_av_id = None;
                for av_id_for_prop in
                    Component::attribute_values_for_prop_id(ctx, self.id, new_prop_id).await?
                {
                    let new_av_path = AttributeValue::get_path_for_id(ctx, av_id_for_prop).await?;

                    if old_av_path == new_av_path {
                        new_av_id = Some(av_id_for_prop);
                    }
                }
                new_av_id
            };

            let key = old_key_or_index
                .as_ref()
                .and_then(|key_or_index| match key_or_index {
                    KeyOrIndex::Key(key) => Some(key.to_owned()),
                    _ => None,
                });

            let new_av_id = match maybe_new_av_id {
                // The value exists in both old and new (thought it might be defaulted)
                Some(new_av_id) => {
                    dvu_roots.push(DependentValueRoot::Unfinished(new_av_id.into()));
                    match maybe_old_component_prototype_id {
                        // The old component has an explicit value set rather than using
                        // the default: set the value in the new component as well.
                        Some(old_component_prototype_id) => {
                            let old_prototype_func =
                                AttributePrototype::func(ctx, old_component_prototype_id).await?;
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
                        // should do the same, so there's not much to do, except for root/si/type!
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
                            // If root/si/type is not set by a component specific prototype,
                            // this means the component was created before we were always setting
                            // a component specific prototype for components.  If we hit this,
                            // just set the value here and now so it will have a component specific prototype
                            // from here on out.
                            //
                            // If for whatever reason, there isn't a value set yet for the type, set it to the old
                            // Prop's default value
                            if prop_path == ["root", "si", "type"] {
                                let old_value =
                                    old_av.value_or_default_or_null(ctx, old_prop_id).await?;
                                AttributeValue::set_value(ctx, new_av_id, Some(old_value)).await?;
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

                    new_av_id
                }
                // The new schema variant never had the value. If it's an array or map
                // element, we need to insert it.
                None => {
                    let Some(old_component_prototype_id) = maybe_old_component_prototype_id else {
                        continue;
                    };

                    let prototype_func = Func::get_by_id(
                        ctx,
                        AttributePrototype::func_id(ctx, old_component_prototype_id).await?,
                    )
                    .await?;

                    // Insert this value
                    let inserted_value = AttributeValue::new(
                        ctx,
                        new_prop_id,
                        Some(self.id),
                        new_parent_id,
                        key.clone(),
                    )
                    .await?;

                    // If the func for this av is dynamic, it will create its own child avs when
                    // executed, if necessary, so we can skip the rest of the loop
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
                    }

                    // If this av is for an object and it did not exist, it means it's a child of
                    // an array or map. We need to create the children of this object
                    // (and any direct object children) so that we don't get a malformed item in
                    // the new component
                    if new_prop.kind == PropKind::Object {
                        let mut queue: VecDeque<_> =
                            Prop::direct_child_props_ordered(ctx, new_prop_id)
                                .await?
                                .into_iter()
                                .map(|prop| (prop, inserted_value.id))
                                .collect();

                        while let Some((this_prop, parent_av_id)) = queue.pop_front() {
                            let attribute_value = AttributeValue::new(
                                ctx,
                                this_prop.id,
                                Some(self.id),
                                Some(parent_av_id),
                                None,
                            )
                            .await?;

                            for child_prop in
                                Prop::direct_child_props_ordered(ctx, this_prop.id).await?
                            {
                                if child_prop.kind == PropKind::Object {
                                    queue.push_back((child_prop, attribute_value.id()))
                                }
                            }
                        }
                    }

                    let old_value = old_av.value(ctx).await?;
                    AttributeValue::set_value(ctx, inserted_value.id, old_value).await?;
                    dvu_roots.push(DependentValueRoot::Unfinished(inserted_value.id.into()));

                    inserted_value.id
                }
            };

            for old_child_av_id in AttributeValue::get_child_av_ids_in_order(ctx, old_av_id).await?
            {
                let old_key_or_index =
                    AttributeValue::get_key_or_index_of_child_entry(ctx, old_child_av_id).await?;
                value_q.push_back((old_child_av_id, old_key_or_index, Some(new_av_id)));
            }
        }

        let component_graph = DependentValueGraph::new(ctx, dvu_roots).await?;
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

        for &apa_id in &apa_ids {
            let func_arg_id =
                AttributePrototypeArgument::func_argument_id_by_id(ctx, apa_id).await?;

            if let Some(source) = AttributePrototypeArgument::value_source_opt(ctx, apa_id).await? {
                match source {
                    ValueSource::InputSocket(input_socket_id) => {
                        // find matching input socket in self
                        let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;
                        match self_input_sockets.get(input_socket.name()) {
                            Some(self_input_socket_id) => new_value_sources.push((
                                func_arg_id,
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
                                func_arg_id,
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
                                .push((func_arg_id, ValueSource::Prop(*self_prop_id))),
                            None => {
                                return Ok(());
                            }
                        }
                    }
                    ValueSource::Secret(_)
                    | ValueSource::StaticArgumentValue(_)
                    | ValueSource::ValueSubscription(_) => {
                        // Should we determine if this secret is still compatible?
                        new_value_sources.push((func_arg_id, source));
                    }
                }
            }
        }

        // All inputs are valid, create the component specific override
        let new_prototype = AttributePrototype::new(ctx, component_prototype_func.id).await?;
        for (func_arg_id, value_source) in new_value_sources {
            AttributePrototypeArgument::new(ctx, new_prototype.id, func_arg_id, value_source)
                .await?;
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
        from_component_id: ComponentId,
    ) -> ComponentResult<()> {
        let from_sv_id = Component::schema_variant_id(ctx, from_component_id).await?;
        let dest_sv_id = Component::schema_variant_id(ctx, self.id).await?;

        if from_sv_id != dest_sv_id {
            return Err(ComponentError::CannotCloneFromDifferentVariants);
        }

        // Paste attribute value "values" from original component (or create them for maps/arrays)
        //
        // We could make this more efficient by skipping everything set by non builtins (si:setString, si:setObject, etc), since everything that is propagated will be re-propagated
        let from_root_id = Component::root_attribute_value_id(ctx, from_component_id).await?;
        let dest_root_id = Component::root_attribute_value_id(ctx, self.id).await?;
        let mut work_queue = VecDeque::from([(from_root_id, dest_root_id)]);
        // Paste attribute prototypes
        // - either updates component prototype to a copy of the original component
        // - or removes component prototype, restoring the schema one (needed because of manual update from the block above)        while
        while let Some((from_av_id, dest_av_id)) = work_queue.pop_front() {
            AttributeValue::clone_value_from(ctx, dest_av_id, from_av_id).await?;

            // Get children, possibly creating new ones if we don't have them yet
            for child_pair in
                AttributeValue::get_child_av_id_pairs_in_order(ctx, from_av_id, dest_av_id).await?
            {
                match child_pair {
                    ChildAttributeValuePair::Both(_, from_child_av_id, dest_child_av_id) => {
                        work_queue.push_back((from_child_av_id, dest_child_av_id));
                    }
                    // If the child is only in the copied component, we create a new one for
                    // ourselves
                    ChildAttributeValuePair::FirstOnly(key, from_child_av_id) => {
                        let dest_child_av_id = AttributeValue::new(
                            ctx,
                            AttributeValue::is_for(ctx, from_child_av_id).await?,
                            Some(self.id),
                            Some(dest_av_id),
                            key,
                        )
                        .await?
                        .id;
                        work_queue.push_back((from_child_av_id, dest_child_av_id));
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
    ) -> ComponentResult<Vec<Connection>> {
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
                    let prototype_id =
                        AttributePrototypeArgument::prototype_id(ctx, apa_id).await?;
                    let input_sources =
                        AttributePrototype::input_sources(ctx, prototype_id).await?;
                    if input_sources.len() > 1 {
                        debug!("More than 1 source for an attribute prototype");
                    }

                    if let Some(AttributePrototypeSource::InputSocket(input_socket, _)) =
                        input_sources.first()
                    {
                        outgoing_edges.push(Connection {
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

    pub async fn outgoing_connections(&self, ctx: &DalContext) -> ComponentResult<Vec<Connection>> {
        Self::outgoing_connections_for_id(ctx, self.id).await
    }

    /// Calls [`Self::incoming_connections_by_id`] by passing in the id from [`self`](Component).
    pub async fn incoming_connections(&self, ctx: &DalContext) -> ComponentResult<Vec<Connection>> {
        Self::incoming_connections_for_id(ctx, self.id).await
    }

    /// Finds all incoming connections for explicit edges (i.e. those coming from
    /// [`Components`](ComponentType::Component) and not from frames.
    #[instrument(
        name = "component.incoming_connections_for_id",
        level = "debug",
        skip(ctx)
    )]
    pub async fn incoming_connections_for_id(
        ctx: &DalContext,
        id: ComponentId,
    ) -> ComponentResult<Vec<Connection>> {
        let mut incoming_connections = vec![];

        for component_input_socket in ComponentInputSocket::list_for_component_id(ctx, id).await? {
            for (from_component_id, from_output_socket_id, apa) in
                component_input_socket.connections(ctx).await?
            {
                let created_info = {
                    let history_actor = ctx.history_actor();
                    let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
                    HistoryEventMetadata {
                        actor,
                        timestamp: apa.timestamp().created_at,
                    }
                };
                incoming_connections.push(Connection {
                    attribute_prototype_argument_id: apa.id(),
                    to_component_id: id,
                    from_component_id,
                    to_input_socket_id: component_input_socket.input_socket_id,
                    from_output_socket_id,
                    created_info,
                    deleted_info: None,
                });
            }
        }

        Ok(incoming_connections)
    }

    #[instrument(
        name = "component.input_sockets_with_connections",
        level = "debug",
        skip(ctx)
    )]
    pub async fn input_sockets_with_connections(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<InputSocketId>> {
        let mut input_socket_ids = Vec::new();
        for input_socket in ComponentInputSocket::list_for_component_id(ctx, component_id).await? {
            let prototype_id =
                AttributeValue::prototype_id(ctx, input_socket.attribute_value_id).await?;
            if !AttributePrototypeArgument::list_ids_for_prototype_and_destination(
                ctx,
                prototype_id,
                component_id,
            )
            .await?
            .is_empty()
            {
                input_socket_ids.push(input_socket.input_socket_id);
            }
        }

        Ok(input_socket_ids)
    }

    /// Gets the list of subscriptions pointing at this root AV, returning the subscriber AV
    /// as well as the path they are subscribed to.
    pub async fn subscribers(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<impl Iterator<Item = (AttributePath, AttributePrototypeArgumentId)>> {
        let root_av_id = Self::root_attribute_value_id(ctx, component_id).await?;
        Ok(AttributeValue::subscribers(ctx, root_av_id).await?)
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

    /// Returns all descendants (children of my children and on and on)
    async fn get_all_descendants_for_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        let mut all_descendants = Vec::new();
        let mut work_queue = VecDeque::from(Self::get_children_for_id(ctx, component_id).await?);
        while let Some(child_id) = work_queue.pop_front() {
            all_descendants.push(child_id);
            work_queue.extend(Self::get_children_for_id(ctx, child_id).await?);
        }
        Ok(all_descendants)
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
    ) -> ComponentResult<Option<(ComponentNodeWeight, ComponentContentV2)>> {
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

            return Ok(Some((component_node_weight, content.extract())));
        }

        Ok(None)
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(ComponentNodeWeight, ComponentContentV2)> {
        Self::try_get_node_weight_and_content(ctx, component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))
    }

    async fn try_get_node_weight_and_content_hash(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<(ComponentNodeWeight, ContentHash)>> {
        let id: Ulid = component_id.into();
        if let Some(node_weight) = ctx.workspace_snapshot()?.get_node_weight_opt(id).await {
            let hash = node_weight.content_hash();
            let component_node_weight = node_weight.get_component_node_weight()?;
            return Ok(Some((component_node_weight, hash)));
        }

        Ok(None)
    }

    /// Returns "true" if the [`Component`] exists on the underlying graph. Returns "false" if it
    /// does not.
    pub async fn exists(ctx: &DalContext, id: ComponentId) -> ComponentResult<bool> {
        Ok(Self::try_get_node_weight_and_content_hash(ctx, id)
            .await?
            .is_some())
    }

    /// List all IDs for all [`Components`](Component) in the workspace.
    pub async fn list_ids(ctx: &DalContext) -> ComponentResult<Vec<ComponentId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let component_category_node_id = workspace_snapshot
            .get_category_node_or_err(None, CategoryNodeKind::Component)
            .await?;

        let component_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                component_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut component_ids = Vec::with_capacity(component_node_indices.len());
        for index in component_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_component_node_weight()?;
            component_ids.push(node_weight.id.into())
        }
        component_ids.sort();

        Ok(component_ids)
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
                    components.push(Self::assemble(&node_weight, content.to_owned().extract()));
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

    pub async fn schema_id_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<SchemaId> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;
        Ok(SchemaVariant::schema_id(ctx, schema_variant_id).await?)
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
        ctx.workspace_snapshot()?
            .schema_variant_id_for_component_id(component_id)
            .await
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

    pub async fn get_by_name(ctx: &DalContext, name: &str) -> ComponentResult<ComponentId> {
        Self::find_by_name(ctx, name)
            .await?
            .ok_or(ComponentError::NotFoundByName(name.into()))
    }

    pub async fn find_by_name(
        ctx: &DalContext,
        name: &str,
    ) -> ComponentResult<Option<ComponentId>> {
        for component_id in Self::list_ids(ctx).await? {
            if name == Self::name_by_id(ctx, component_id).await? {
                return Ok(Some(component_id));
            }
        }
        Ok(None)
    }

    pub async fn geometry(&self, ctx: &DalContext, view_id: ViewId) -> ComponentResult<Geometry> {
        Ok(Geometry::get_by_component_and_view(ctx, self.id, view_id).await?)
    }

    pub async fn set_geometry(
        &mut self,
        ctx: &DalContext,
        view_id: ViewId,
        x: isize,
        y: isize,
        width: Option<isize>,
        height: Option<isize>,
    ) -> ComponentResult<Geometry> {
        let new_geometry = RawGeometry {
            x,
            y,
            width,
            height,
        };

        self.set_raw_geometry(ctx, new_geometry, view_id).await
    }

    pub async fn set_raw_geometry(
        &mut self,
        ctx: &DalContext,
        raw_geometry: RawGeometry,
        view_id: ViewId,
    ) -> ComponentResult<Geometry> {
        let mut geometry_pre = self.geometry(ctx, view_id).await?;
        if geometry_pre.into_raw() != raw_geometry {
            geometry_pre.update(ctx, raw_geometry).await?;
        }

        Ok(geometry_pre)
    }

    pub async fn set_resource_id(
        &self,
        ctx: &DalContext,
        resource_id: &str,
    ) -> ComponentResult<()> {
        let path = ["root", "si", "resourceId"];
        let sv_id = Self::schema_variant_id(ctx, self.id).await?;

        let Some(resource_prop_id) =
            Prop::find_prop_id_by_path_opt(ctx, sv_id, &PropPath::new(path)).await?
        else {
            return Ok(());
        };

        // If the name prop is controlled by an identity or other function,
        // don't override the prototype here
        if Prop::is_set_by_dependent_function(ctx, resource_prop_id).await? {
            return Ok(());
        }

        let av_for_resource_id =
            Self::attribute_value_for_prop_id(ctx, self.id(), resource_prop_id).await?;

        AttributeValue::update(
            ctx,
            av_for_resource_id,
            Some(serde_json::to_value(resource_id)?),
        )
        .await?;

        Ok(())
    }

    pub async fn set_name(&self, ctx: &DalContext, name: &str) -> ComponentResult<()> {
        let path = ["root", "si", "name"];
        let sv_id = Self::schema_variant_id(ctx, self.id).await?;
        let name_prop_id = Prop::find_prop_id_by_path(ctx, sv_id, &PropPath::new(path)).await?;
        // If the name prop is controlled by an identity or other function,
        // don't override the prototype here
        if Prop::is_set_by_dependent_function(ctx, name_prop_id).await? {
            return Ok(());
        }

        let av_for_name = Self::attribute_value_for_prop_id(ctx, self.id(), name_prop_id).await?;

        AttributeValue::update(ctx, av_for_name, Some(serde_json::to_value(name)?)).await?;

        Ok(())
    }

    pub async fn set_resource(
        &self,
        ctx: &DalContext,
        resource: ResourceData,
    ) -> ComponentResult<()> {
        let av_for_resource =
            Component::attribute_value_for_prop(ctx, self.id(), &["root", "resource"]).await?;

        AttributeValue::update(ctx, av_for_resource, Some(serde_json::to_value(resource)?)).await?;

        Ok(())
    }

    pub async fn clear_resource(&self, ctx: &DalContext) -> ComponentResult<()> {
        let av_for_resource =
            Component::attribute_value_for_prop(ctx, self.id(), &["root", "resource"]).await?;

        AttributeValue::update(ctx, av_for_resource, Some(serde_json::json!({}))).await?;

        Ok(())
    }

    /// Finds the [`ResourceData`] for a given [`Component`].
    pub async fn resource(&self, ctx: &DalContext) -> ComponentResult<Option<ResourceData>> {
        Self::resource_by_id(ctx, self.id).await
    }

    /// Finds the [`ResourceData`] for a given [`ComponentId`](Component).
    pub async fn resource_by_id(
        ctx: &DalContext,
        id: ComponentId,
    ) -> ComponentResult<Option<ResourceData>> {
        let value_id = Self::attribute_value_for_prop(ctx, id, &["root", "resource"]).await?;

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

    /// Returns the name of a [`Component`] for a given [`ComponentId`](Component).
    pub async fn name_by_id(ctx: &DalContext, id: ComponentId) -> ComponentResult<String> {
        let name_value_id =
            Self::attribute_value_for_prop(ctx, id, &["root", "si", "name"]).await?;

        let name_av = AttributeValue::get_by_id(ctx, name_value_id).await?;

        let view_result = name_av.view(ctx).await?;

        Ok(match view_result {
            Some(serde_value) => serde_json::from_value(serde_value)?,
            None => "".into(),
        })
    }

    /// Returns the name of the [`Component`].
    pub async fn name(&self, ctx: &DalContext) -> ComponentResult<String> {
        Self::name_by_id(ctx, self.id).await
    }

    // Returns the resource id from the prop tree
    pub async fn resource_id(&self, ctx: &DalContext) -> ComponentResult<String> {
        let prop_path = PropPath::new(["root", "si", "resourceId"]);
        let prop_id =
            Prop::find_prop_id_by_path_opt(ctx, self.schema_variant(ctx).await?.id, &prop_path)
                .await?;
        if let Some(prop_id) = prop_id {
            let resource_id_value_id =
                Self::attribute_value_for_prop_id(ctx, self.id, prop_id).await?;

            let resource_id_av = AttributeValue::get_by_id(ctx, resource_id_value_id).await?;

            Ok(match resource_id_av.view(ctx).await? {
                Some(serde_value) => serde_json::from_value(serde_value)?,
                None => "".into(),
            })
        } else {
            Ok("".into())
        }
    }

    pub async fn color(&self, ctx: &DalContext) -> ComponentResult<Option<String>> {
        let color_value_id =
            Component::attribute_value_for_prop(ctx, self.id(), &["root", "si", "color"]).await?;
        let color_av = AttributeValue::get_by_id(ctx, color_value_id).await?;

        Ok(match color_av.view(ctx).await? {
            Some(serde_value) => Some(serde_json::from_value(serde_value)?),
            None => None,
        })
    }
    pub async fn color_by_id(ctx: &DalContext, id: ComponentId) -> ComponentResult<Option<String>> {
        let color_value_id =
            Component::attribute_value_for_prop(ctx, id, &["root", "si", "color"]).await?;
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
            Self::attribute_value_for_prop(ctx, component_id, &["root", "si", "type"]).await?;
        let type_value = AttributeValue::get_by_id(ctx, type_value_id)
            .await?
            .view(ctx)
            .await?
            .ok_or(ComponentError::ComponentMissingTypeValueMaterializedView(
                component_id,
            ))?;

        Ok(serde_json::from_value(type_value)?)
    }
    /// Sets the [`AttributeValue`] for root/si/type to the given [`ComponentType`]
    /// NOTE: This does NOT ensure that this change is valid, nor does it account for
    /// needing to update other attribute values in cases where the new type is an up or
    /// down frame
    pub async fn set_type_by_id_unchecked(
        ctx: &DalContext,
        component_id: ComponentId,
        new_type: ComponentType,
    ) -> ComponentResult<()> {
        let type_value_id =
            Self::attribute_value_for_prop(ctx, component_id, &["root", "si", "type"]).await?;
        let value = serde_json::to_value(new_type)?;

        AttributeValue::update(ctx, type_value_id, Some(value)).await?;
        ctx.workspace_snapshot()?
            .clear_inferred_connection_graph()
            .await;

        Ok(())
    }

    pub async fn get_type(&self, ctx: &DalContext) -> ComponentResult<ComponentType> {
        Self::get_type_by_id(ctx, self.id()).await
    }

    /// For the given [`ComponentId`], updates the type.  If the type is changing from or to an Up/Down Frame,
    /// this ensures we update the necessary values given the changing data flows
    pub async fn set_type_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
        new_type: ComponentType,
    ) -> ComponentResult<()> {
        // cache the current type
        let current_type = Self::get_type_by_id(ctx, component_id).await?;

        let children = Self::get_children_for_id(ctx, component_id).await?;

        // see if this component is a parent or child
        let reference_id = match Self::get_parent_by_id(ctx, component_id).await? {
            Some(parent) => Some(parent),
            None => children.first().copied(),
        };

        // if the current component has children, and the new type is a component, return an error
        if new_type == ComponentType::Component && !children.is_empty() {
            return Err(ComponentError::ComponentHasChildren);
        }

        // no-op if we're not actually changing the type
        if new_type == current_type {
            return Ok(());
        }
        if let Some(reference_id) = reference_id {
            // this means the component is a child or parent,
            //so we need to ensure we update any necessary values
            match (new_type, current_type) {
                (ComponentType::Component, ComponentType::ConfigurationFrameDown)
                | (ComponentType::Component, ComponentType::ConfigurationFrameUp)
                | (ComponentType::ConfigurationFrameDown, ComponentType::Component)
                | (ComponentType::ConfigurationFrameDown, ComponentType::ConfigurationFrameUp)
                | (ComponentType::ConfigurationFrameUp, ComponentType::Component)
                | (ComponentType::ConfigurationFrameUp, ComponentType::ConfigurationFrameDown) => {
                    Frame::update_type_from_or_to_frame(ctx, component_id, reference_id, new_type)
                        .await?;
                }
                (new, old) => return Err(ComponentError::InvalidComponentTypeUpdate(old, new)),
            }
        } else {
            // this component stands alone, just set the type!
            Self::set_type_by_id_unchecked(ctx, component_id, new_type).await?;
        }

        Ok(())
    }

    async fn set_type(&self, ctx: &DalContext, new_type: ComponentType) -> ComponentResult<()> {
        let type_value_id =
            Component::attribute_value_for_prop(ctx, self.id(), &["root", "si", "type"]).await?;

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
        let all_relevant_prop_ids = Prop::all_parent_prop_ids_from_prop_id(ctx, prop_id).await?;
        let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;

        let mut work_queue = VecDeque::from([root_attribute_value_id]);
        let mut early_return = false;
        while let Some(attribute_value_id) = work_queue.pop_front() {
            let working_prop_id = AttributeValue::prop_id(ctx, attribute_value_id).await?;

            // We found one! But we might have more. This should ensure we finish everything
            // at the current rank, but don't descend.
            if prop_id == working_prop_id {
                early_return = true;
                result.push(attribute_value_id);
            }

            if !early_return && all_relevant_prop_ids.contains(&working_prop_id) {
                let children =
                    AttributeValue::get_child_av_ids_in_order(ctx, attribute_value_id).await?;
                if !children.is_empty() {
                    work_queue.extend(children);
                }
            }
        }
        Ok(result)
    }

    // Get a single attribute value for this component and a given prop path
    // Errors if there is no value, or if more than one value exists.
    pub async fn attribute_value_for_prop_id(
        ctx: &DalContext,
        component_id: ComponentId,
        prop_id: PropId,
    ) -> ComponentResult<AttributeValueId> {
        let values = Self::attribute_values_for_prop_id(ctx, component_id, prop_id).await?;
        if values.len() > 1 {
            return Err(ComponentError::ComponentHasTooManyValues(
                component_id,
                prop_id,
            ));
        }
        match values.first() {
            Some(value) => Ok(*value),
            None => Err(ComponentError::ComponentMissingValue(component_id, prop_id)),
        }
    }

    // Get a single attribute value for this component and a given prop path
    // Errors if there is no value, or if more than one value exists.
    pub async fn attribute_value_for_prop(
        ctx: &DalContext,
        component_id: ComponentId,
        prop_path: &[&str],
    ) -> ComponentResult<AttributeValueId> {
        let schema_variant_id = Self::schema_variant_id(ctx, component_id).await?;

        let prop_id =
            Prop::find_prop_id_by_path(ctx, schema_variant_id, &PropPath::new(prop_path)).await?;

        let result = Self::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;

        Ok(result)
    }

    pub async fn domain_prop_attribute_value(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<AttributeValueId> {
        Component::attribute_value_for_prop(ctx, self.id(), &["root", "domain"]).await
    }
    pub async fn resource_value_prop_attribute_value(
        &self,
        ctx: &DalContext,
    ) -> ComponentResult<AttributeValueId> {
        Component::attribute_value_for_prop(ctx, self.id(), &["root", "resource_value"]).await
    }

    pub async fn attribute_values_for_all_sockets(
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
            .remove_edge(
                parent_id,
                child_id,
                EdgeWeightKindDiscriminants::FrameContains,
            )
            .await?;
        ctx.workspace_snapshot()?
            .clear_inferred_connection_graph()
            .await;

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
                    source, destination,
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
        let destination_attribute_value_id = InputSocket::component_attribute_value_id(
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
        ctx.workspace_snapshot()?
            .clear_inferred_connection_graph()
            .await;

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

        let base_attribute_prototype_argument_node_weight = base_change_set_ctx
            .workspace_snapshot()?
            .get_node_weight(base_change_set_connection.attribute_prototype_argument_id)
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
                    return Err(AttributePrototypeArgumentError::MissingValueSource(
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
                    .into());
                }
            };

        let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;

        // Recreate the AttributePrototypeArgument & associated edges.
        // We only need to import the AttributePrototypeArgument node, as all of the other relevant
        // nodes should already exist.
        ctx.workspace_snapshot()?
            .add_or_replace_node(base_attribute_prototype_argument_node_weight.clone())
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
            let func = Func::get_by_id(ctx, func_id).await?;

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

    /// Returns the value of the "to_delete" field using solely the graph node weight.
    pub async fn is_set_to_delete(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Option<bool>> {
        match ctx
            .workspace_snapshot()?
            .get_node_weight_opt(component_id)
            .await
        {
            Some(component_node_weight) => Ok(Some(
                component_node_weight
                    .get_component_node_weight()?
                    .to_delete(),
            )),
            None => Ok(None),
        }
    }

    async fn modify<L>(self, ctx: &DalContext, lambda: L) -> ComponentResult<Self>
    where
        L: FnOnce(&mut Self) -> ComponentResult<()>,
    {
        let original_component = self.clone();
        let mut component = self;

        let before = ComponentContentV2::from(component.clone());
        lambda(&mut component)?;

        // The `to_delete` lives on the node itself, not in the content, so we need to be a little
        // more manual when updating that field.
        if component.to_delete != original_component.to_delete {
            let component_node_weight = ctx
                .workspace_snapshot()?
                .get_node_weight(original_component.id)
                .await?
                .get_component_node_weight()?;
            let mut new_component_node_weight = component_node_weight.clone();
            new_component_node_weight.set_to_delete(component.to_delete);
            ctx.workspace_snapshot()?
                .add_or_replace_node(NodeWeight::Component(new_component_node_weight))
                .await?;
        }

        let updated = ComponentContentV2::from(component.clone());
        if updated != before {
            let (hash, _) = ctx.layer_db().cas().write(
                Arc::new(ComponentContent::V2(updated.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )?;
            ctx.workspace_snapshot()?
                .update_content(component.id.into(), hash)
                .await?;
        }

        let component_node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(original_component.id)
            .await?
            .get_component_node_weight()?;

        Ok(Component::assemble(&component_node_weight, updated))
    }

    /// Remove a [Component] from the graph, and all related nodes
    #[instrument(level = "info", skip(ctx))]
    pub async fn remove(ctx: &DalContext, id: ComponentId) -> ComponentResult<()> {
        let component = Self::get_by_id(ctx, id).await?;
        let root_attribute_value_id = Self::root_attribute_value_id(ctx, id).await?;

        if let Some(parent_id) = component.parent(ctx).await? {
            // if we are removing a component with children, re-parent them if I have a parent
            // if this component doesn't have a parent, it's children will be orphaned anyways
            for child_id in Component::get_children_for_id(ctx, id).await? {
                Frame::upsert_parent(ctx, child_id, parent_id).await?;
            }
            Frame::orphan_child(ctx, id).await?;
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

        // Remove all geometries for the component
        Geometry::remove_all_for_component_id(ctx, id).await?;

        // Remove all actions for this component from queue
        Action::remove_all_for_component_id(ctx, id).await?;
        WsEvent::action_list_updated(ctx)
            .await?
            .publish_on_commit(ctx)
            .await?;

        // Deleting the root attribute value will remove all ValueSubscription edges that point to it.
        ctx.workspace_snapshot()?
            .remove_node_by_id(root_attribute_value_id)
            .await?;
        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;

        Ok(())
    }

    /// A [`Component`] is allowed to be removed from the graph if it meets the following
    /// requirements:
    ///
    /// 1. It doesn't have a populated resource.
    /// 2. It is not feeding data to a [`Component`] that has a populated resource.
    /// 3. It doesn't have descendants with resources
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

        // If I am a frame, and I have descendants with resources, I can't be removed
        let all_descendants = Self::get_all_descendants_for_id(ctx, self.id).await?;
        for descendant in all_descendants {
            let descendant_component = Self::get_by_id(ctx, descendant).await?;
            if descendant_component.resource(ctx).await?.is_some() {
                debug!(
                    "component {:?} cannot be removed because {:?} has resource",
                    self.id,
                    descendant_component.id()
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
                Action::new(ctx, prototype_id, Some(component_id)).await?;
            }
        } else if !to_delete {
            // Remove delete actions for component
            Action::remove_all_for_component_id(ctx, component_id).await?;
            WsEvent::action_list_updated(ctx)
                .await?
                .publish_on_commit(ctx)
                .await?;
        }

        Ok(modified)
    }

    /// If the attribute value is somewhere in 'root/domain', the component has a resource, and a single update function,
    /// and there isn't an update func already enqueued for this component, enqueue it!
    pub async fn enqueue_update_action_if_applicable(
        ctx: &DalContext,
        modified_attribute_value_id: AttributeValueId,
    ) -> ComponentResult<Option<Action>> {
        if let Some(prop_for_value) =
            AttributeValue::prop_opt(ctx, modified_attribute_value_id).await?
        {
            if prop_for_value
                .path(ctx)
                .await?
                .is_descendant_of(&PropPath::new(["root", "domain"]))
            {
                let component_id =
                    AttributeValue::component_id(ctx, modified_attribute_value_id).await?;
                if Component::resource_by_id(ctx, component_id)
                    .await?
                    .is_some()
                {
                    // then if the current component has an update action, enqueue it
                    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
                    let mut prototypes_for_variant = SchemaVariant::find_action_prototypes_by_kind(
                        ctx,
                        schema_variant_id,
                        ActionKind::Update,
                    )
                    .await?;

                    if prototypes_for_variant.len() > 1 {
                        // if there are multiple update funcs, not sure which one to enqueue!
                        return Ok(None);
                    }
                    if let Some(prototype_id) = prototypes_for_variant.pop() {
                        // don't enqueue the same action twice!
                        if Action::find_equivalent(ctx, prototype_id, Some(component_id))
                            .await?
                            .is_none()
                        {
                            let new_action =
                                Action::new(ctx, prototype_id, Some(component_id)).await?;
                            return Ok(Some(new_action));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    pub async fn autoconnect(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<(
        Vec<InputSocketId>,
        HashMap<
            ComponentInputSocket,
            (
                Vec<(ComponentId, OutputSocketId, serde_json::Value)>,
                Option<serde_json::Value>,
            ),
        >,
    )> {
        let mut available_input_sockets_to_connect = HashMap::new();
        let incoming_connections =
            Component::incoming_connections_for_id(ctx, component_id).await?;
        struct PotentialMatchData {
            attr_value_id: AttributeValueId,
            value: Option<serde_json::Value>,
        }
        // for a given component, check all domain props for values that have a component specific prototype
        let attribute_value_tree = AttributeValue::tree_for_component(ctx, component_id).await?;
        let mut flattened = Vec::new();
        let mut queue = VecDeque::from_iter(attribute_value_tree.keys().copied());

        while let Some(av_id) = queue.pop_front() {
            flattened.push(av_id);
            if let Some(children) = attribute_value_tree.get(&av_id) {
                queue.extend(children);
            }
        }

        for attribute_value_id in flattened {
            if AttributeValue::component_prototype_id(ctx, attribute_value_id)
                .await?
                .is_some()
            {
                // this is a component specific prototype. Let's see what the input args would have been though
                if let Some(prop) = AttributeValue::prop_opt(ctx, attribute_value_id).await? {
                    // get the schema variant defined prototype to see what it's potential inputs are
                    let schema_prototype = Prop::prototype_id(ctx, prop.id()).await?;
                    let attribute_prototype_arg_ids =
                        AttributePrototypeArgument::list_ids_for_prototype(ctx, schema_prototype)
                            .await?;
                    // if any of the arguments are for an Input Socket, that means this value *could* have
                    // been set by a socket
                    for attr_arg_id in attribute_prototype_arg_ids {
                        let value_source =
                            AttributePrototypeArgument::value_source(ctx, attr_arg_id).await?;
                        if let ValueSource::InputSocket(input_socket_id) = value_source {
                            // find this specific input socket and see if it already has a connection (actual one)
                            if !incoming_connections
                                .iter()
                                .any(|conn| conn.to_input_socket_id == input_socket_id)
                            {
                                // no existing connection, so let's queue this up to see if there's a matching socket
                                // somewhere!
                                let av = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
                                let view = av.view(ctx).await?;
                                let info_needed = PotentialMatchData {
                                    attr_value_id: attribute_value_id,
                                    value: view,
                                };
                                available_input_sockets_to_connect
                                    .insert(input_socket_id, info_needed);
                            }
                        }
                    }
                }
            }
        }

        // build up a map of potential matches.  We consider a match valid if the sockets can actually connect
        // and the value for the attribute value matches the output socket's value
        // OR for multi-arity sockets, we also check if the output socket matches a single entry in the array
        let mut potential_matches: HashMap<
            InputSocketId,
            Vec<(ComponentId, OutputSocketId, serde_json::Value)>,
        > = HashMap::new();

        // loop through all components + output sockets
        for component in Self::list(ctx).await? {
            // don't interrogate current component!
            if component.id() == component_id {
                continue;
            }
            // build a map of component output sockets and values
            let output_sockets = component.output_socket_attribute_values(ctx).await?;
            let outgoing_connections_to_check = component
                .outgoing_connections(ctx)
                .await?
                .iter()
                .map(|outgoing| outgoing.from_output_socket_id)
                .collect_vec();
            for output_socket_av in output_sockets {
                if let Some(output_socket_id) =
                    OutputSocket::find_for_attribute_value_id(ctx, output_socket_av).await?
                {
                    let is_single_arity = OutputSocket::get_by_id(ctx, output_socket_id)
                        .await?
                        .arity()
                        == SocketArity::One;
                    // First ensure that if the socket arity is single, there isn't an existing connection from this output socket
                    if !is_single_arity
                        || !outgoing_connections_to_check.contains(&output_socket_id)
                    {
                        let av = AttributeValue::get_by_id(ctx, output_socket_av).await?;
                        let output_value = av.view(ctx).await?;
                        // Check against the list of available input sockets for compatibility
                        for (input_socket_id, info) in &available_input_sockets_to_connect {
                            // Does this output socket fits this input socket? (including annotations!)
                            if OutputSocket::fits_input_by_id(
                                ctx,
                                *input_socket_id,
                                output_socket_id,
                            )
                            .await?
                            {
                                let input_socket =
                                    InputSocket::get_by_id(ctx, *input_socket_id).await?;

                                // does the output socket have a value?
                                if let (Some(input), Some(output)) = (&info.value, &output_value) {
                                    // does the output socket's value match what's set for the attribute value?
                                    if input == output {
                                        potential_matches.entry(*input_socket_id).or_default().push(
                                            (component.id(), output_socket_id, output.clone()),
                                        )
                                    }
                                    // if value for the prop we're trying to connect is an array, and the input socket is
                                    // multi-arity, see if the output socket's value matches an entry in the array

                                    // let's see if anything matches either case!
                                    if let serde_json::Value::Array(values) = input {
                                        if input_socket.arity() == SocketArity::Many
                                            && values.contains(output)
                                        {
                                            potential_matches
                                                .entry(*input_socket_id)
                                                .or_default()
                                                .push((
                                                    component.id(),
                                                    output_socket_id,
                                                    output.clone(),
                                                ))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // now we've got potential matches, so let's iterate over them and create connections for the ones that match 1:1
        // if there's a conflict, record it as such and surface it to the user!

        let mut connections_created = Vec::new();
        let mut conflicted_connections = HashMap::new();

        for (input_socket_id, matches) in potential_matches {
            // Get the input socket to check its arity
            let input_socket = InputSocket::get_by_id(ctx, input_socket_id).await?;

            if let Some(info_needed) = available_input_sockets_to_connect.get(&input_socket_id) {
                if let Some(serde_json::Value::Array(values)) = &info_needed.value {
                    if input_socket.arity() == SocketArity::Many {
                        // For array values with arity many, ensure each value has exactly one match
                        let mut valid_matches = Vec::new();
                        let mut coverage_for_all_items = true;
                        let mut duplicate_entry_for_items = false;
                        // but first let's see if any output socket matches the whole thing
                        let whole_array_match: Vec<_> = matches
                            .iter()
                            .filter(|(_, _, output_value)| {
                                Some(output_value) == info_needed.value.as_ref()
                            })
                            .collect();

                        for value in values {
                            let value_matches: Vec<_> = matches
                                .iter()
                                .filter(|(_, _, output_value)| output_value == value)
                                .collect();
                            if value_matches.len() == 1 {
                                valid_matches.push(value_matches[0]);
                            }
                            // if there's one value that doesn't exist, it means we don't have a full set
                            else if value_matches.is_empty() {
                                coverage_for_all_items = false;
                            }
                            // if there are multiple valid matches, record them anyways for returning conflicts later
                            else if value_matches.len() > 1 {
                                duplicate_entry_for_items = true;
                                valid_matches.extend(value_matches);
                            }
                        }

                        // Use the whole match iff (if and only if!):
                        // 1. There is exactly one connection that is a whole match AND
                        // 2. We don't have coverage for all of the entries by individual connections OR
                        let should_use_whole_match =
                            (whole_array_match.len() == 1) && !coverage_for_all_items;

                        // Use the individual matches iff:
                        // 1. There are no whole matches AND
                        // 2. We have coverage of all entries in the array AND
                        // 3. We do not have any duplicates!
                        let should_use_valid_matches = whole_array_match.is_empty()
                            && coverage_for_all_items
                            && !duplicate_entry_for_items;

                        // only 1 valid match on the whole array, yeehaw
                        if should_use_whole_match {
                            // Reset the prototype and create the one connection!
                            AttributeValue::use_default_prototype(ctx, info_needed.attr_value_id)
                                .await?;
                            let (source_component_id, output_socket_id, value) =
                                whole_array_match[0];

                            match Component::connect(
                                ctx,
                                *source_component_id,
                                *output_socket_id,
                                component_id,
                                input_socket_id,
                            )
                            .await
                            {
                                Ok(_) => connections_created.push(input_socket_id),
                                Err(err) => {
                                    warn!(si.error.message = ?err, "Failed to create connection to Component {}", *source_component_id);
                                    AttributeValue::set_value(
                                        ctx,
                                        info_needed.attr_value_id,
                                        Some(value.clone()),
                                    )
                                    .await?;
                                }
                            }
                        } else if should_use_valid_matches {
                            // Reset the prototype then create connections for each valid output socket match
                            AttributeValue::use_default_prototype(ctx, info_needed.attr_value_id)
                                .await?;
                            for &(source_component_id, output_socket_id, value) in &valid_matches {
                                match Component::connect(
                                    ctx,
                                    *source_component_id,
                                    *output_socket_id,
                                    component_id,
                                    input_socket_id,
                                )
                                .await
                                {
                                    Ok(_) => connections_created.push(input_socket_id),
                                    Err(err) => {
                                        warn!(si.error.message = ?err, "Failed to create connection to Component {}", *source_component_id);
                                        // need to reset this value if there was an error otherwise it's lost
                                        AttributeValue::set_value(
                                            ctx,
                                            info_needed.attr_value_id,
                                            Some(value.clone()),
                                        )
                                        .await?;
                                        // don't keep looping if one of the array values fails, just reset to what it was
                                        break;
                                    }
                                }
                            }
                        } else {
                            // conflicted!
                            // Return all whole matches we found, and all individual matches but only if we have full coverage
                            // maybe in the future we return partial matches??
                            let mut conflicted = Vec::new();
                            for wm in whole_array_match {
                                conflicted.push(wm.clone());
                            }
                            if coverage_for_all_items {
                                for im in valid_matches {
                                    conflicted.push(im.clone());
                                }
                            }
                            if !conflicted.is_empty() {
                                conflicted_connections.insert(
                                    ComponentInputSocket {
                                        component_id,
                                        input_socket_id,
                                        attribute_value_id: info_needed.attr_value_id,
                                    },
                                    (conflicted, info_needed.value.clone()),
                                );
                            }
                        }
                    }
                } else
                // For non-array values or arity one, just check if there's exactly one match
                // if so, use it!
                if matches.len() == 1 {
                    AttributeValue::use_default_prototype(ctx, info_needed.attr_value_id).await?;
                    let (source_component_id, output_socket_id, value) = &matches[0];
                    match Component::connect(
                        ctx,
                        *source_component_id,
                        *output_socket_id,
                        component_id,
                        input_socket_id,
                    )
                    .await
                    {
                        Err(err) => {
                            warn!(si.error.message = ?err, "Failed to create connection to Component {}", source_component_id);
                            AttributeValue::set_value(
                                ctx,
                                info_needed.attr_value_id,
                                Some(value.clone()),
                            )
                            .await?;
                        }
                        Ok(_) => connections_created.push(input_socket_id),
                    }
                } else if matches.len() > 1 {
                    // conflicted! let the user pick
                    conflicted_connections.insert(
                        ComponentInputSocket {
                            component_id,
                            input_socket_id,
                            attribute_value_id: info_needed.attr_value_id,
                        },
                        (matches, info_needed.value.clone()),
                    );
                }
            }
        }

        Ok((connections_created, conflicted_connections))
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
                        let prototype_id =
                            AttributePrototypeArgument::prototype_id(ctx, argument_using_id)
                                .await?;
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

    pub async fn duplicate_without_connections(
        &self,
        ctx: &DalContext,
        view_id: ViewId,
        component_geometry: RawGeometry,
    ) -> ComponentResult<Self> {
        let schema_variant = self.schema_variant(ctx).await?;

        let mut pasted_comp = Component::new(
            ctx,
            Self::generate_copy_name(self.name(ctx).await?),
            schema_variant.id(),
            view_id,
        )
        .await?;

        pasted_comp
            .set_geometry(
                ctx,
                view_id,
                component_geometry.x,
                component_geometry.y,
                component_geometry.width,
                component_geometry.height,
            )
            .await?;

        pasted_comp.clone_attributes_from(ctx, self.id()).await?;
        Ok(pasted_comp)
    }

    pub async fn duplicate(
        ctx: &mut DalContext,
        to_view_id: ViewId,
        components: Vec<ComponentId>,
    ) -> ComponentResult<Vec<ComponentId>> {
        let mut pasted_component_ids = vec![];
        let mut to_pasted_id = HashMap::new();

        for component_id in components.into_iter() {
            let component = Component::get_by_id(ctx, component_id).await?;
            let pasted_component = component
                .duplicate_without_connections(
                    ctx,
                    to_view_id,
                    RawGeometry {
                        x: 0,
                        y: 0,
                        width: None,
                        height: None,
                    },
                )
                .await?;
            pasted_component_ids.push(pasted_component.id());
            to_pasted_id.insert(component_id, pasted_component.id());

            // Copy manager connections
            for manager_id in Component::managers_by_id(ctx, component_id).await? {
                Component::manage_component(ctx, manager_id, pasted_component.id).await?;
            }
        }

        Ok(pasted_component_ids)
    }

    // Copy a batch of components, and replicate connections between them
    pub async fn batch_copy(
        ctx: &mut DalContext,
        to_view_id: ViewId,
        to_parent_id: Option<ComponentId>,
        components: Vec<(ComponentId, RawGeometry)>,
    ) -> ComponentResult<Vec<ComponentId>> {
        // Paste all the components and get the mapping from original to pasted
        let mut pasted_component_ids = vec![];
        let mut to_pasted_id = HashMap::new();
        for (component_id, raw_geometry) in components.into_iter() {
            let component = Component::get_by_id(ctx, component_id).await?;
            let pasted_component = component
                .duplicate_without_connections(ctx, to_view_id, raw_geometry)
                .await?;
            pasted_component_ids.push(pasted_component.id());
            to_pasted_id.insert(component_id, pasted_component.id());
        }

        let maybe_pasted = |id: ComponentId| to_pasted_id.get(&id).copied().unwrap_or(id);

        // Fix parentage and connections
        for (&component_id, &pasted_component_id) in &to_pasted_id {
            // Fix parentage:
            // 1. If the component's parent was in the batch, use the pasted version.
            // 2. Otherwise, set it to the place the user is pasting to.
            let pasted_parent_id = Component::get_parent_by_id(ctx, component_id)
                .await?
                .and_then(|parent_id| to_pasted_id.get(&parent_id).copied());
            if let Some(pasted_parent_id) = pasted_parent_id.or(to_parent_id) {
                Frame::upsert_parent(ctx, pasted_component_id, pasted_parent_id).await?;
            }

            // Copy manager connections
            for manager_id in Component::managers_by_id(ctx, component_id).await? {
                // If we were managed by a component that was also pasted, we should be managed by
                // the pasted version--otherwise we're still managed by the original
                Component::manage_component(
                    ctx,
                    maybe_pasted(manager_id),
                    maybe_pasted(component_id),
                )
                .await?;
            }

            // Copy incoming socket connections
            for connection in Component::incoming_connections_for_id(ctx, component_id).await? {
                Component::connect(
                    ctx,
                    maybe_pasted(connection.from_component_id),
                    connection.from_output_socket_id,
                    pasted_component_id,
                    connection.to_input_socket_id,
                )
                .await?;
            }

            // Find pasted components that subscribe to copied components, and
            // resubscribe them to the pasted component
            for (path, subscriber_apa_id) in Component::subscribers(ctx, component_id).await? {
                let subscriber_ap_id =
                    AttributePrototypeArgument::prototype_id(ctx, subscriber_apa_id).await?;
                let Some(subscriber_av_id) =
                    AttributePrototype::attribute_value_id(ctx, subscriber_ap_id).await?
                else {
                    continue;
                };
                let subscriber_id = AttributeValue::component_id(ctx, subscriber_av_id).await?;
                if pasted_component_ids.contains(&subscriber_id) {
                    let pasted_root_id =
                        Component::root_attribute_value_id(ctx, pasted_component_id).await?;
                    AttributePrototypeArgument::set_value_source(
                        ctx,
                        subscriber_apa_id,
                        ValueSource::ValueSubscription(ValueSubscription {
                            attribute_value_id: pasted_root_id,
                            path,
                        }),
                    )
                    .await?;
                }
            }
        }

        Ok(pasted_component_ids)
    }

    pub async fn add_to_view(
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: ViewId,
        raw_geometry: RawGeometry,
    ) -> ComponentResult<()> {
        if Geometry::try_get_by_component_and_view(ctx, component_id, view_id)
            .await?
            .is_some()
        {
            return Err(ComponentError::ComponentAlreadyInView(
                component_id,
                view_id,
            ));
        }

        let mut geometry = Geometry::new_for_component(ctx, component_id, view_id).await?;

        geometry.update(ctx, raw_geometry).await?;

        Ok(())
    }

    /// Finds all inferred incoming connections for the [`Component`]
    /// A connection is inferred if it's input socket is being driven
    /// by another component's output socket as a result of lineage
    /// via FrameContains Edges.
    #[instrument(level = "debug", skip(ctx))]
    pub async fn inferred_incoming_connections(
        ctx: &DalContext,
        to_component_id: ComponentId,
    ) -> ComponentResult<Vec<InferredConnection>> {
        let mut connections = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connection_graph =
            workspace_snapshot.inferred_connection_graph(ctx).await?;
        let incoming_connections = inferred_connection_graph
            .inferred_incoming_connections_for_component(ctx, to_component_id)
            .await?;

        for incoming_connection in incoming_connections {
            // add the check for to_delete on either to or from component
            // Both "deleted" and not deleted Components can feed data into
            // "deleted" Components. **ONLY** not deleted Components can feed
            // data into not deleted Components.
            let to_delete = !Self::should_data_flow_between_components(
                ctx,
                to_component_id,
                incoming_connection.source_component_id,
            )
            .await?;

            connections.push(InferredConnection {
                to_component_id,
                to_input_socket_id: incoming_connection.input_socket_id,
                from_component_id: incoming_connection.source_component_id,
                from_output_socket_id: incoming_connection.output_socket_id,
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
        let mut connections = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connections = workspace_snapshot.inferred_connection_graph(ctx).await?;
        let mut inferred_connections_for_component_stack = inferred_connections
            .inferred_connections_for_component_stack(ctx, self.id)
            .await?;
        inferred_connections_for_component_stack
            .retain(|inferred_connection| inferred_connection.source_component_id == self.id);

        for outgoing_connection in inferred_connections_for_component_stack {
            // add the check for to_delete on either to or from component
            // Both "deleted" and not deleted Components can feed data into
            // "deleted" Components. **ONLY** not deleted Components can feed
            // data into not deleted Components.
            let destination_component = outgoing_connection.destination_component_id;
            let source_component = self.id();

            let to_delete = !Self::should_data_flow_between_components(
                ctx,
                destination_component,
                source_component,
            )
            .await?;
            connections.push(InferredConnection {
                to_component_id: outgoing_connection.destination_component_id,
                to_input_socket_id: outgoing_connection.input_socket_id,
                from_component_id: outgoing_connection.source_component_id,
                from_output_socket_id: outgoing_connection.output_socket_id,
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
        // InputSocket -> Prototype: AttributePrototype
        let input_socket_prototype_id =
            AttributePrototype::find_for_input_socket(ctx, destination_input_socket_id)
                .await?
                .ok_or_else(|| InputSocketError::MissingPrototype(destination_input_socket_id))?;

        // -> PrototypeArgument:
        let attribute_prototype_arguments = ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                input_socket_prototype_id,
                Outgoing,
                EdgeWeightKindDiscriminants::PrototypeArgument,
            )
            .await?;

        for (_, _, attribute_prototype_arg_idx) in attribute_prototype_arguments {
            // AttributePrototypeArgument { source, target }
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
                    // -> PrototypeArgumentValue:
                    let data_sources = ctx
                        .workspace_snapshot()?
                        .edges_directed_for_edge_weight_kind(
                            attribute_prototype_argument_node_weight.id(),
                            Outgoing,
                            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                        )
                        .await?;

                    for (_, _, data_source_idx) in data_sources {
                        // OutputSocket
                        let node_weight = ctx
                            .workspace_snapshot()?
                            .get_node_weight(data_source_idx)
                            .await?;
                        if let Ok(output_socket_node_weight) = node_weight
                            .get_content_node_weight_of_kind(
                                ContentAddressDiscriminants::OutputSocket,
                            )
                        {
                            // OutputSocket
                            if output_socket_node_weight.id() == source_output_socket_id.into() {
                                AttributePrototypeArgument::remove(
                                    ctx,
                                    attribute_prototype_argument_node_weight.id().into(),
                                )
                                .await?;

                                let destination_attribute_value_id =
                                    InputSocket::component_attribute_value_id(
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
        ctx.workspace_snapshot()?
            .clear_inferred_connection_graph()
            .await;

        Ok(())
    }

    #[instrument(level = "debug", skip(ctx))]
    pub async fn upgrade_to_new_variant(
        ctx: &DalContext,
        original_component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<Component> {
        let original_component = Self::get_by_id(ctx, original_component_id).await?;

        // ================================================================================
        // Cache original component data
        // ================================================================================
        let snap = ctx.workspace_snapshot()?;

        let original_component_node_weight = snap.get_node_weight(original_component.id).await?;

        let original_component_name = Self::name_by_id(ctx, original_component_id).await?;
        let original_component_lineage_id = original_component_node_weight.lineage_id();

        let original_managed = original_component.get_managed(ctx).await?;
        let original_managers = original_component.managers(ctx).await?;
        let original_incoming_connections = original_component.incoming_connections(ctx).await?;
        let original_outgoing_connections = original_component.outgoing_connections(ctx).await?;
        let original_root_id =
            Component::root_attribute_value_id(ctx, original_component_id).await?;
        let original_subscriber_apas = AttributeValue::subscribers(ctx, original_root_id).await?;

        let original_parent = original_component.parent(ctx).await?;
        let original_children = Component::get_children_for_id(ctx, original_component_id).await?;

        let geometry_ids = Geometry::list_ids_by_component(ctx, original_component_id).await?;

        // ================================================================================
        // Create new component and run changes that depend on the old one still existing
        // ================================================================================
        let new_component_with_temp_id = Component::new_with_content_address_and_no_geometry(
            ctx,
            original_component_name.clone(),
            schema_variant_id,
            original_component_node_weight.content_hash(),
        )
        .await?;

        // Move geometries to new component
        for geometry_id in geometry_ids {
            snap.remove_edge(
                geometry_id,
                original_component_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?;
            snap.add_edge(
                geometry_id,
                EdgeWeight::new(EdgeWeightKind::Represents),
                new_component_with_temp_id.id,
            )
            .await?;
        }

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

        // Remove old component connections
        for &original_managed_id in &original_managed {
            Component::unmanage_component(ctx, original_component_id, original_managed_id).await?;
        }
        for &original_manager_id in &original_managers {
            Component::unmanage_component(ctx, original_manager_id, original_component_id).await?;
        }
        let mut original_subscriber_prototypes = vec![];
        for (path, apa_id) in original_subscriber_apas {
            let prototype_id = AttributePrototypeArgument::prototype_id(ctx, apa_id).await?;
            AttributePrototypeArgument::remove(ctx, apa_id).await?;
            original_subscriber_prototypes.push((path, prototype_id));
        }

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
        // Remove all children from the "old" frame before we delete it. We'll add them all to the
        // new frame after we've deleted the old one.
        for &child in &original_children {
            Frame::orphan_child(ctx, child).await?;
        }

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
        let upgraded_component = Self::get_by_id(ctx, original_component_id).await?;
        let mut diagram_sockets = HashMap::new();

        // Restore parent connection on new component
        if let Some(parent) = original_parent {
            Frame::upsert_parent(ctx, upgraded_component.id(), parent).await?;
        }

        let payload = upgraded_component
            .into_frontend_type(ctx, None, ChangeStatus::Unmodified, &mut diagram_sockets)
            .await?;
        WsEvent::component_upgraded(ctx, payload, upgraded_component.id())
            .await?
            .publish_on_commit(ctx)
            .await?;

        // Restore child connections on new component
        for child in original_children {
            Frame::upsert_parent(ctx, child, upgraded_component.id()).await?;
        }

        // Restore connections on new component
        for original_managed_id in original_managed {
            Component::manage_component(ctx, upgraded_component.id(), original_managed_id).await?;
        }
        for original_manager_id in original_managers {
            Component::manage_component(ctx, original_manager_id, upgraded_component.id()).await?;
        }
        for incoming in &original_incoming_connections {
            let socket = InputSocket::get_by_id(ctx, incoming.to_input_socket_id).await?;
            if let Some(socket) =
                InputSocket::find_with_name(ctx, socket.name(), schema_variant_id).await?
            {
                Component::connect(
                    ctx,
                    incoming.from_component_id,
                    incoming.from_output_socket_id,
                    upgraded_component.id(),
                    socket.id(),
                )
                .await?;
                let edge = SummaryDiagramEdge {
                    from_component_id: incoming.from_component_id,
                    from_socket_id: incoming.from_output_socket_id,
                    to_component_id: upgraded_component.id(),
                    to_socket_id: socket.id(),
                    // was Unmodified, but get_diagram shows them as Added
                    change_status: ChangeStatus::Added,
                    created_info: serde_json::to_value(&incoming.created_info)?,
                    deleted_info: serde_json::to_value(&incoming.deleted_info)?,
                    to_delete: false,
                    from_base_change_set: false,
                };
                WsEvent::connection_upserted(ctx, edge.into())
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            } else {
                debug!(
                    "Unable to reconnect to socket_id: {0} for component_id: {1}",
                    socket.id(),
                    upgraded_component.id()
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
                    upgraded_component.id(),
                    socket.id(),
                    outgoing.to_component_id,
                    outgoing.to_input_socket_id,
                )
                .await?;
                let edge = SummaryDiagramEdge {
                    from_component_id: upgraded_component.id(),
                    from_socket_id: socket.id(),
                    to_component_id: outgoing.to_component_id,
                    to_socket_id: outgoing.to_input_socket_id,
                    // was Unmodified, but get_diagram shows them as Added
                    change_status: ChangeStatus::Added,
                    created_info: serde_json::to_value(&outgoing.created_info)?,
                    deleted_info: serde_json::to_value(&outgoing.deleted_info)?,
                    to_delete: false,
                    from_base_change_set: false,
                };
                WsEvent::connection_upserted(ctx, edge.into())
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
            } else {
                debug!(
                    "Unable to reconnect to socket_id: {0} for component_id: {1}",
                    socket.id(),
                    upgraded_component.id()
                );
            }
        }

        // Reconnect subscribers
        let finalized_root_id =
            Component::root_attribute_value_id(ctx, upgraded_component.id()).await?;
        for (path, prototype_id) in original_subscriber_prototypes {
            AttributePrototype::add_arg_to_intrinsic(
                ctx,
                prototype_id,
                ValueSubscription {
                    attribute_value_id: finalized_root_id,
                    path,
                },
            )
            .await?;
        }

        Ok(upgraded_component)
    }

    async fn requeue_actions_for_upgraded_component(
        ctx: &DalContext,
        old_component_id: ComponentId,
        new_component_id: ComponentId,
        new_schema_variant_id: SchemaVariantId,
    ) -> ComponentResult<()> {
        // Remove any actions created for the new component as a side effect of the upgrade
        // Then loop through the existing queued actions for the old component and re-add them piecemeal.
        Action::remove_all_for_component_id(ctx, new_component_id).await?;

        let queued_for_old_component = Action::find_for_component_id(ctx, old_component_id).await?;
        let available_for_new_component =
            ActionPrototype::for_variant(ctx, new_schema_variant_id).await?;
        for existing_queued in queued_for_old_component {
            let action = Action::get_by_id(ctx, existing_queued).await?;
            let action_prototype_id = Action::prototype_id(ctx, existing_queued).await?;
            // what do we do about the various states?
            // maybe you shouldn't upgrade a component if an action
            // is dispatched or running for the current?
            match action.state() {
                ActionState::Failed | ActionState::OnHold | ActionState::Queued => {
                    let func_id = ActionPrototype::func_id(ctx, action_prototype_id).await?;
                    let queued_func = Func::get_by_id(ctx, func_id).await?;

                    for available_action_prototype in available_for_new_component.clone() {
                        let available_func_id =
                            ActionPrototype::func_id(ctx, available_action_prototype.id()).await?;
                        let available_func = Func::get_by_id(ctx, available_func_id).await?;

                        if available_func.name == queued_func.name
                            && available_func.kind == queued_func.kind
                        {
                            Action::new(
                                ctx,
                                available_action_prototype.id(),
                                Some(new_component_id),
                            )
                            .await?;
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
            .import_component_subgraph(&base_change_set_ctx.workspace_snapshot()?, component_id)
            .await?;

        let component = Component::get_by_id(ctx, component_id).await?;

        ctx.add_dependent_values_and_enqueue(component.input_socket_attribute_values(ctx).await?)
            .await?;

        Geometry::restore_all_for_component_id(ctx, component_id).await?;

        Ok(())
    }

    pub async fn exists_on_head(
        ctx: &DalContext,
        component_ids: &[ComponentId],
    ) -> ComponentResult<HashSet<ComponentId>> {
        let mut components = HashSet::new();
        let base_change_set_ctx = ctx.clone_with_base().await?;
        for &component_id in component_ids {
            let maybe_component =
                Component::try_get_by_id(&base_change_set_ctx, component_id).await?;
            if maybe_component.is_some() {
                components.insert(component_id);
            }
        }
        Ok(components)
    }

    pub async fn can_be_upgraded_by_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<bool> {
        let schema_variant = Component::schema_variant_for_component_id(ctx, component_id).await?;

        let schema_id = Component::schema_id_for_component_id(ctx, component_id).await?;
        let default_schema_variant_id =
            SchemaVariant::default_id_for_schema(ctx, schema_id).await?;

        let newest_schema_variant_id =
            match SchemaVariant::get_unlocked_for_schema(ctx, schema_id).await? {
                Some(unlocked_schema_variant) => unlocked_schema_variant.id(),
                None => default_schema_variant_id,
            };

        Ok(if newest_schema_variant_id != schema_variant.id() {
            // There's a chance that the exact same asset was installed in
            // different change sets and then applied to head. In that case,
            // there's no need to show the upgrade for this component, since the
            // upgrade will be effectively a no-op.
            let current_module = Module::find_for_member_id(ctx, schema_variant.id()).await?;
            let new_module = Module::find_for_member_id(ctx, newest_schema_variant_id).await?;

            match (current_module, new_module) {
                (Some(current_module), Some(new_module)) => {
                    current_module.root_hash() != new_module.root_hash()
                }
                _ => true,
            }
        } else {
            false
        })
    }

    /// Is there a newer version of the schema variant that this component is using?
    pub async fn can_be_upgraded(&self, ctx: &DalContext) -> ComponentResult<bool> {
        let schema_variant = self.schema_variant(ctx).await?;
        let schema = self.schema(ctx).await?;
        let default_schema_variant_id =
            SchemaVariant::default_id_for_schema(ctx, schema.id()).await?;

        let newest_schema_variant_id =
            match SchemaVariant::get_unlocked_for_schema(ctx, schema.id()).await? {
                Some(unlocked_schema_variant) => unlocked_schema_variant.id(),
                None => default_schema_variant_id,
            };

        Ok(if newest_schema_variant_id != schema_variant.id() {
            // There's a chance that the exact same asset was installed in
            // different change sets and then applied to head. In that case,
            // there's no need to show the upgrade for this component, since the
            // upgrade will be effectively a no-op.
            let current_module = Module::find_for_member_id(ctx, schema_variant.id()).await?;
            let new_module = Module::find_for_member_id(ctx, newest_schema_variant_id).await?;

            match (current_module, new_module) {
                (Some(current_module), Some(new_module)) => {
                    current_module.root_hash() != new_module.root_hash()
                }
                _ => true,
            }
        } else {
            false
        })
    }

    /// Remove a [`Manages`](`crate::edge_weight::EdgeWeightKind::Manages`)
    /// edge from a manager component to a managed component
    pub async fn unmanage_component(
        ctx: &DalContext,
        manager_component_id: ComponentId,
        managed_component_id: ComponentId,
    ) -> ComponentResult<()> {
        ctx.workspace_snapshot()?
            .remove_edge(
                manager_component_id,
                managed_component_id,
                EdgeWeightKindDiscriminants::Manages,
            )
            .await?;

        Ok(())
    }

    /// Add a [`Manages`](`crate::edge_weight::EdgeWeightKind::Manages`) edge
    /// from a manager component to a managed component, if the managed
    /// component is based on a managed schema
    pub async fn manage_component(
        ctx: &DalContext,
        manager_component_id: ComponentId,
        managed_component_id: ComponentId,
    ) -> ComponentResult<SummaryDiagramManagementEdge> {
        let manager_schema_id = Component::schema_for_component_id(ctx, manager_component_id)
            .await?
            .id();
        let managed_component_schema_id = Self::schema_for_component_id(ctx, managed_component_id)
            .await?
            .id();

        let guard = ctx.workspace_snapshot()?.enable_cycle_check().await;

        Component::add_manages_edge_to_component(
            ctx,
            manager_component_id,
            managed_component_id,
            EdgeWeightKind::Manages,
        )
        .await?;

        drop(guard);

        Ok(SummaryDiagramManagementEdge::new(
            manager_schema_id,
            managed_component_schema_id,
            manager_component_id,
            managed_component_id,
        ))
    }

    /// Return the IDs of all the [`Components`](Component) that manage this [`Component`](Component).
    pub async fn managers(&self, ctx: &DalContext) -> ComponentResult<Vec<ComponentId>> {
        Self::managers_by_id(ctx, self.id).await
    }

    /// Return the IDs of all the [`Components`](Component) that manage the [`Component`](Component) corresponding
    /// to the provided ID.
    pub async fn managers_by_id(
        ctx: &DalContext,
        id: ComponentId,
    ) -> ComponentResult<Vec<ComponentId>> {
        let mut result = Vec::new();

        let snapshot = ctx.workspace_snapshot()?;

        for source_idx in snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Manages)
            .await?
        {
            let node_weight = snapshot.get_node_weight(source_idx).await?;
            if let NodeWeight::Component(_) = &node_weight {
                result.push(node_weight.id().into());
            }
        }

        Ok(result)
    }

    /// Return the ids of all the components managed by this component
    pub async fn get_managed(&self, ctx: &DalContext) -> ComponentResult<Vec<ComponentId>> {
        let mut result = vec![];

        let snapshot = ctx.workspace_snapshot()?;

        for target_idx in snapshot
            .outgoing_targets_for_edge_weight_kind(self.id, EdgeWeightKindDiscriminants::Manages)
            .await?
        {
            let node_weight = snapshot.get_node_weight(target_idx).await?;
            if let NodeWeight::Component(_) = &node_weight {
                result.push(node_weight.id().into());
            }
        }

        Ok(result)
    }

    pub async fn into_frontend_type(
        &self,
        ctx: &DalContext,
        maybe_geometry: Option<&Geometry>,
        change_status: ChangeStatus,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> ComponentResult<DiagramComponentView> {
        let schema_variant = self.schema_variant(ctx).await?;

        let schema_sockets = match diagram_sockets.entry(schema_variant.id()) {
            hash_map::Entry::Vacant(entry) => {
                let (output_sockets, input_sockets) =
                    SchemaVariant::list_all_sockets(ctx, schema_variant.id()).await?;

                let (management_input_socket, management_output_socket) =
                    SchemaVariant::get_management_sockets(ctx, schema_variant.id()).await?;

                let mut sockets = vec![];
                sockets.push(management_input_socket);

                for socket in input_sockets {
                    sockets.push(DiagramSocket {
                        id: socket.id().to_string(),
                        label: socket.name().to_string(),
                        connection_annotations: socket
                            .connection_annotations()
                            .into_iter()
                            .map(|a| a.into())
                            .collect(),
                        direction: DiagramSocketDirection::Input,
                        max_connections: match socket.arity() {
                            SocketArity::Many => None,
                            SocketArity::One => Some(1),
                        },
                        is_required: Some(false),
                        node_side: DiagramSocketNodeSide::Left,
                        is_management: Some(false),
                        value: None,
                    });
                }

                if let Some(management_output_socket) = management_output_socket {
                    sockets.push(management_output_socket);
                }

                for socket in output_sockets {
                    sockets.push(DiagramSocket {
                        id: socket.id().to_string(),
                        label: socket.name().to_string(),
                        connection_annotations: socket
                            .connection_annotations()
                            .into_iter()
                            .map(|a| a.into())
                            .collect(),
                        direction: DiagramSocketDirection::Output,
                        max_connections: match socket.arity() {
                            SocketArity::Many => None,
                            SocketArity::One => Some(1),
                        },
                        is_required: Some(false),
                        node_side: DiagramSocketNodeSide::Right,
                        is_management: Some(false),
                        value: None,
                    });
                }
                entry.insert(sockets.to_owned());
                sockets
            }
            hash_map::Entry::Occupied(entry) => entry.get().to_owned(),
        };
        let mut sockets = Vec::new();
        for mut comp_socket in schema_sockets.clone() {
            if let Some(is_managed) = comp_socket.is_management {
                // management sockets do not have values, so don't try to get them
                // but we still want to return them, silly silly
                if is_managed {
                    sockets.push(comp_socket.clone());
                    continue;
                }
            }
            let socket_value = match comp_socket.direction {
                DiagramSocketDirection::Bidirectional => None,
                DiagramSocketDirection::Input => {
                    ComponentInputSocket::value_for_input_socket_id_for_component_id_opt(
                        ctx,
                        self.id(),
                        InputSocketId::from_str(&comp_socket.id)?,
                    )
                    .await?
                }
                DiagramSocketDirection::Output => {
                    ComponentOutputSocket::value_for_output_socket_id_for_component_id_opt(
                        ctx,
                        self.id(),
                        OutputSocketId::from_str(&comp_socket.id)?,
                    )
                    .await?
                }
            };
            comp_socket.value = socket_value;
            sockets.push(comp_socket);
        }
        let schema = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant.id()).await?;
        let schema_id = schema.id();

        let updated_info = {
            let history_actor = ctx.history_actor();
            let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
            serde_json::to_value(HistoryEventMetadata {
                actor,
                timestamp: self.timestamp().updated_at,
            })?
        };

        let created_info = {
            let history_actor = ctx.history_actor();
            let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
            serde_json::to_value(HistoryEventMetadata {
                actor,
                timestamp: self.timestamp().created_at,
            })?
        };

        let can_be_upgraded = self.can_be_upgraded(ctx).await?;

        let maybe_parent = self.parent(ctx).await?;

        let geometry = if let Some(geometry) = maybe_geometry {
            let view_id = Geometry::get_view_id_by_id(ctx, geometry.id()).await?;

            Some(GeometryAndView {
                view_id,
                geometry: geometry.into_raw(),
            })
        } else {
            None
        };

        Ok(DiagramComponentView {
            id: self.id(),
            component_id: self.id(),
            schema_name: schema.name().to_owned(),
            schema_id,
            schema_docs_link: schema_variant.link(),
            schema_variant_id: schema_variant.id(),
            schema_variant_name: schema_variant.version().to_owned(),
            schema_category: schema_variant.category().to_owned(),
            display_name: self.name(ctx).await?,
            resource_id: self.resource_id(ctx).await?,
            component_type: self.get_type(ctx).await?.to_string(),
            color: self.color(ctx).await?.unwrap_or("#111111".into()),
            change_status: change_status.into(),
            has_resource: self.resource(ctx).await?.is_some(),
            sockets,
            parent_id: maybe_parent,
            updated_info,
            created_info,
            deleted_info: serde_json::Value::Null,
            to_delete: self.to_delete(),
            can_be_upgraded,
            from_base_change_set: false,
            view_data: geometry,
        })
    }

    pub async fn into_frontend_type_for_default_view(
        &self,
        ctx: &DalContext,
        change_status: ChangeStatus,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> ComponentResult<DiagramComponentView> {
        let default_view_id = View::get_id_for_default(ctx).await?;
        let geometry = self.geometry(ctx, default_view_id).await?;

        self.into_frontend_type(ctx, Some(&geometry), change_status, diagram_sockets)
            .await
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCreatedPayload {
    pub component: DiagramComponentView,
    pub inferred_edges: Option<Vec<SummaryDiagramInferredEdge>>,
    change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentUpdatedPayload {
    pub component: DiagramComponentView,
    pub change_set_id: ChangeSetId,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentUpgradedPayload {
    component: DiagramComponentView,
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
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConnectionDeletedPayload {
    #[serde(rename_all = "camelCase")]
    AttributeValueEdge {
        from_component_id: ComponentId,
        to_component_id: ComponentId,
        from_socket_id: OutputSocketId,
        to_socket_id: InputSocketId,
    },
    #[serde(rename_all = "camelCase")]
    ManagementEdge {
        from_component_id: ComponentId,
        to_component_id: ComponentId,
    },
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ConnectionUpsertedPayload {
    AttribueValueEdge(SummaryDiagramEdge),
    ManagementEdge(SummaryDiagramManagementEdge),
}

impl From<SummaryDiagramEdge> for ConnectionUpsertedPayload {
    fn from(value: SummaryDiagramEdge) -> Self {
        ConnectionUpsertedPayload::AttribueValueEdge(value)
    }
}

impl From<SummaryDiagramManagementEdge> for ConnectionUpsertedPayload {
    fn from(value: SummaryDiagramManagementEdge) -> Self {
        ConnectionUpsertedPayload::ManagementEdge(value)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPosition {
    x: isize,
    y: isize,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSize {
    width: Option<isize>,
    height: Option<isize>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSetPosition {
    component_id: ComponentId,
    x: isize,
    y: isize,
    width: Option<isize>,
    height: Option<isize>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSetPositionPayload {
    change_set_id: ChangeSetId,
    view_id: ViewId,
    positions: Vec<ComponentSetPosition>,
    // Used so the client can ignore the messages it caused. created by the frontend, and not stored
    client_ulid: Option<ulid::Ulid>,
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
            None,
            None,
            WsPayload::SetComponentPosition(payload),
        )
        .await
    }

    pub async fn set_component_position(
        ctx: &DalContext,
        change_set_id: ChangeSetId,
        view_id: ViewId,
        geometries: Vec<(Ulid, RawGeometry)>,
        client_ulid: Option<ulid::Ulid>,
    ) -> WsEventResult<Self> {
        let mut positions: Vec<ComponentSetPosition> = vec![];
        for (component_id, geometry) in geometries {
            positions.push(ComponentSetPosition {
                component_id: component_id.into(),
                x: geometry.x,
                y: geometry.y,
                width: geometry.width,
                height: geometry.height,
            });
        }
        WsEvent::new(
            ctx,
            WsPayload::SetComponentPosition(ComponentSetPositionPayload {
                change_set_id,
                view_id,
                positions,
                client_ulid,
            }),
        )
        .await
    }

    pub async fn component_created(
        ctx: &DalContext,
        component: DiagramComponentView,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentCreated(ComponentCreatedPayload {
                change_set_id: ctx.change_set_id(),
                inferred_edges: None,
                component,
            }),
        )
        .await
    }

    pub async fn component_created_with_inferred_edges(
        ctx: &DalContext,
        component: DiagramComponentView,
        inferred_edges: Option<Vec<SummaryDiagramInferredEdge>>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ComponentCreated(ComponentCreatedPayload {
                change_set_id: ctx.change_set_id(),
                inferred_edges,
                component,
            }),
        )
        .await
    }

    pub async fn connection_upserted(
        ctx: &DalContext,
        payload: ConnectionUpsertedPayload,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ConnectionUpserted(payload)).await
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
            WsPayload::ConnectionDeleted(ConnectionDeletedPayload::AttributeValueEdge {
                from_component_id,
                to_component_id,
                from_socket_id,
                to_socket_id,
            }),
        )
        .await
    }

    pub async fn manages_edge_deleted(
        ctx: &DalContext,
        from_component_id: ComponentId,
        to_component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ConnectionDeleted(ConnectionDeletedPayload::ManagementEdge {
                from_component_id,
                to_component_id,
            }),
        )
        .await
    }

    pub async fn component_updated(
        ctx: &DalContext,
        payload: DiagramComponentView,
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
        payload: DiagramComponentView,
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
