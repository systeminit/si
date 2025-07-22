use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    sync::Arc,
};

pub use dependent_value_graph::DependentValueGraph;
pub use is_for::ValueIsFor;
use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Value,
    json,
};
use si_events::{
    FuncRunValue,
    ulid::Ulid,
};
use si_pkg::{
    AttributeValuePath,
    KeyOrIndex,
};
use si_split_graph::SplitGraphError;
use subscription::ValueSubscription;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{
    RwLock,
    TryLockError,
};

use super::{
    path::AttributePath,
    prototype::argument::{
        AttributePrototypeArgument,
        AttributePrototypeArgumentError,
        AttributePrototypeArgumentId,
        static_value::StaticArgumentValue,
        value_source::{
            ValueSource,
            ValueSourceError,
        },
    },
};
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
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
    Secret,
    SecretError,
    TransactionsError,
    attribute::prototype::{
        AttributePrototypeError,
        AttributePrototypeSource,
    },
    change_set::ChangeSetError,
    component::inferred_connection_graph::InferredConnectionGraphError,
    func::{
        FuncExecutionPk,
        argument::{
            FuncArgument,
            FuncArgumentError,
        },
        intrinsics::IntrinsicFunc,
        runner::{
            FuncRunner,
            FuncRunnerError,
        },
    },
    implement_add_edge_to,
    prop::PropError,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    validation::{
        ValidationError,
        ValidationOutput,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        dependent_value_root::DependentValueRootError,
        edge_weight::{
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::WorkspaceSnapshotGraphError,
        node_weight::{
            AttributeValueNodeWeight,
            NodeWeight,
            NodeWeightDiscriminants,
            NodeWeightError,
            PropNodeWeight,
        },
        serde_value_to_string_type,
        traits::attribute_value::AttributeValueExt,
    },
};

pub mod debug;
pub mod dependent_value_graph;
pub mod is_for;
pub mod subscription;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AttributeValueError {
    #[error("action error: {0}")]
    Action(String),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error(
        "attribute prototype argument {0} has a value source {1:?} but no value for that prop found in component {2}"
    )]
    AttributePrototypeArgumentMissingValueInSourceComponent(
        AttributePrototypeArgumentId,
        ValueSource,
        ComponentId,
    ),
    #[error("attribute value {0} has no prototype")]
    AttributeValueMissingPrototype(AttributeValueId),
    #[error("attribute value {0} has more than one edge to a prop")]
    AttributeValueMultiplePropEdges(AttributeValueId),
    #[error("before func error: {0}")]
    BeforeFunc(String),
    #[error("Cannot create nested values for {0} since it is not the value for a prop")]
    CannotCreateNestedValuesForNonPropValues(AttributeValueId),
    #[error("Cannot create attribute value for root prop without component id")]
    CannotCreateRootPropValueWithoutComponentId,
    #[error("Cannot create attribute value for socket without component id")]
    CannotCreateSocketValueWithoutComponentId,
    #[error("cannot explicitly set the value of {0} because it is for an input or output socket")]
    CannotExplicitlySetSocketValues(AttributeValueId),
    #[error("cannot set child of value {0} because it has a dynamic prototype")]
    CannotSetChildOfDynamicValue(AttributeValueId),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error(
        "scalar attribute value {parent_id} has child {child_id} but has scalar type {parent_kind}"
    )]
    ChildOfScalar {
        parent_id: AttributeValueId,
        child_id: AttributeValueId,
        parent_kind: PropKind,
    },
    #[error(
        "socket {socket:?} has a child attribute value {child_id} (should only have a single av)"
    )]
    ChildOfSocket {
        socket: ValueIsFor,
        child_id: AttributeValueId,
    },
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("dependent value root error: {0}")]
    DependentValueRoot(#[from] DependentValueRootError),
    #[error("duplicate key or index {key_or_index} for attribute values {child1} and {child2}")]
    DuplicateKeyOrIndex {
        key_or_index: KeyOrIndex,
        child1: AttributeValueId,
        child2: AttributeValueId,
    },
    #[error("array element missing from parent ordering node: {0}")]
    ElementMissingFromOrderingNode(AttributeValueId),
    #[error("empty attribute prototype arguments for group name: {0}")]
    EmptyAttributePrototypeArgumentsForGroup(String),
    #[error("object field is not a child prop of the object prop: {0}")]
    FieldNotChildOfObject(AttributeValueId),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] Box<FuncArgumentError>),
    #[error("function result failure: kind={kind}, message={message}, backend={backend}")]
    FuncBackendResultFailure {
        kind: String,
        message: String,
        backend: String,
    },
    #[error("func runner error: {0}")]
    FuncRunner(#[from] Box<FuncRunnerError>),
    #[error("func runner result sender was dropped before sending")]
    FuncRunnerSend,
    #[error("helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("attempt to access out-of-range element {0} of array {1} with length {2}")]
    IndexOutOfRange(usize, AttributeValueId, usize),
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("cannot insert for prop kind: {0}")]
    InsertionForInvalidPropKind(PropKind),
    #[error("jsonptr parse error parsing {0}: {1}")]
    JsonptrParseError(String, jsonptr::ParseError),
    #[error("jsonptr parse index error parsing {0}: {1}")]
    JsonptrParseIndexError(String, jsonptr::index::ParseIndexError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing attribute value with id: {0}")]
    MissingForId(AttributeValueId),
    #[error("missing key for map entry {0}")]
    MissingKeyForMapEntry(AttributeValueId),
    #[error("attribute value {0} missing prop edge when one was expected")]
    MissingPropEdge(AttributeValueId),
    #[error("missing prototype for attribute value {0}")]
    MissingPrototype(AttributeValueId),
    #[error(
        "found multiple child attribute values ({0} and {1}, at minimum) for the same prop: {2}"
    )]
    MultipleAttributeValuesSameProp(AttributeValueId, AttributeValueId, PropId),
    #[error("found multiple props ({0} and {1}, at minimum) for attribute value: {2}")]
    MultiplePropsFound(PropId, PropId, AttributeValueId),
    #[error("found multiple prototypes for attribute value id {0}")]
    MultiplePrototypesFound(AttributeValueId),
    #[error("attribute value {0} has no child named {1}")]
    NoChildWithName(AttributeValueId, String),
    #[error("no component prototype found for attribute value: {0}")]
    NoComponentPrototype(AttributeValueId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight mismatch, expected {0} to be {1:?}")]
    NodeWeightMismatch(Ulid, NodeWeightDiscriminants),
    #[error("attribute value does not have ordering node as expected: {0}")]
    NoOrderingNodeForAttributeValue(AttributeValueId),
    #[error("attribute value not found for component ({0}) and input socket ({1})")]
    NotFoundForComponentAndInputSocket(ComponentId, InputSocketId),
    #[error("attribute value {0} has no outgoing edge to a prop or socket")]
    OrphanedAttributeValue(AttributeValueId),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] Box<OutputSocketError>),
    #[error("parent prop of map or array not found: {0}")]
    ParentAttributeValueMissing(AttributeValueId),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("array or map prop missing element prop: {0}")]
    PropMissingElementProp(PropId),
    #[error("array or map prop has more than one child prop: {0}")]
    PropMoreThanOneChild(PropId),
    #[error("prop not found for attribute value: {0}")]
    PropNotFound(AttributeValueId),
    #[error("secret error: {0}")]
    Secret(#[from] Box<SecretError>),
    #[error("serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(
        "attribute value {0} with type {1} must be set to 1 subscription, attempted to include {2} subscriptions"
    )]
    SingleValueMustHaveOneSubscription(AttributeValueId, PropKind, usize),
    #[error("Split  graph error: {0}")]
    SplitGraph(#[from] SplitGraphError),
    #[error("Cannot set subscription with function that isn't builtin or transformation")]
    SubscribingWithInvalidFunction,
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("type mismatch: expected prop kind {0}, got {1}")]
    TypeMismatch(PropKind, String),
    #[error("unexpected graph layout: {0}")]
    UnexpectedGraphLayout(&'static str),
    #[error("reached unreachable code")]
    Unreachable,
    #[error("validation error: {0}")]
    Validation(#[from] Box<ValidationError>),
    #[error("value source error: {0}")]
    ValueSource(#[from] Box<ValueSourceError>),
    #[error("workspace error: {0}")]
    Workspace(String),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("Workspace Snapshot Graph error: {0}")]
    WorkspaceSnapshotGraph(#[from] WorkspaceSnapshotGraphError),
}

impl From<ComponentError> for AttributeValueError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}
impl From<FuncRunnerError> for AttributeValueError {
    fn from(value: FuncRunnerError) -> Self {
        Box::new(value).into()
    }
}
impl From<SecretError> for AttributeValueError {
    fn from(value: SecretError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for AttributeValueError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for AttributeValueError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for AttributeValueError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncArgumentError> for AttributeValueError {
    fn from(value: FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for AttributeValueError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for AttributeValueError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for AttributeValueError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValidationError> for AttributeValueError {
    fn from(value: ValidationError) -> Self {
        Box::new(value).into()
    }
}

impl From<ValueSourceError> for AttributeValueError {
    fn from(value: ValueSourceError) -> Self {
        Box::new(value).into()
    }
}

pub type AttributeValueResult<T> = Result<T, AttributeValueError>;

pub use si_id::AttributeValueId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    pub id: AttributeValueId,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<ContentAddress>,
    // DEPRECATED, should always be None
    pub func_execution_pk: Option<FuncExecutionPk>,
}

///
/// Returned from AttributeValue::get_child_av_id_pairs_in_order(ctx, first, second)
///
#[remain::sorted]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildAttributeValuePair {
    Both(Option<String>, AttributeValueId, AttributeValueId),
    FirstOnly(Option<String>, AttributeValueId),
    SecondOnly(Option<String>, AttributeValueId),
}

impl ChildAttributeValuePair {
    pub fn key(&self) -> Option<&String> {
        match self {
            Self::Both(key, _, _) | Self::FirstOnly(key, _) | Self::SecondOnly(key, _) => {
                key.into()
            }
        }
    }
    pub fn first(&self) -> Option<AttributeValueId> {
        match self {
            Self::Both(_, first, _) | Self::FirstOnly(_, first) => Some(*first),
            Self::SecondOnly(_, _) => None,
        }
    }
    pub fn second(&self) -> Option<AttributeValueId> {
        match self {
            Self::Both(_, _, second) | Self::SecondOnly(_, second) => Some(*second),
            Self::FirstOnly(_, _) => None,
        }
    }
}

impl From<AttributeValueNodeWeight> for AttributeValue {
    fn from(value: AttributeValueNodeWeight) -> Self {
        Self {
            id: value.id().into(),
            unprocessed_value: value.unprocessed_value(),
            value: value.value(),
            func_execution_pk: None,
        }
    }
}

impl AttributeValue {
    pub fn id(&self) -> AttributeValueId {
        self.id
    }

    implement_add_edge_to!(
        source_id: AttributeValueId,
        destination_id: AttributeValueId,
        add_fn: add_edge_to_attribute_value,
        discriminant: EdgeWeightKindDiscriminants::Contain,
        result: AttributeValueResult,
    );
    implement_add_edge_to!(
        source_id: AttributeValueId,
        destination_id: AttributePrototypeId,
        add_fn: add_edge_to_attribute_prototype,
        discriminant: EdgeWeightKindDiscriminants::Prototype,
        result: AttributeValueResult,
    );
    implement_add_edge_to!(
        source_id: AttributeValueId,
        destination_id: PropId,
        add_fn: add_edge_to_prop,
        discriminant: EdgeWeightKindDiscriminants::Prop,
        result: AttributeValueResult,
    );
    implement_add_edge_to!(
        source_id: AttributeValueId,
        destination_id: OutputSocketId,
        add_fn: add_edge_to_output_socket,
        discriminant: EdgeWeightKindDiscriminants::Socket,
        result: AttributeValueResult,
    );
    implement_add_edge_to!(
        source_id: AttributeValueId,
        destination_id: InputSocketId,
        add_fn: add_edge_to_input_socket,
        discriminant: EdgeWeightKindDiscriminants::Socket,
        result: AttributeValueResult,
    );

    pub async fn new(
        ctx: &DalContext,
        is_for: impl Into<ValueIsFor>,
        component_id: Option<ComponentId>,
        maybe_parent_attribute_value: Option<AttributeValueId>,
        key: Option<String>,
    ) -> AttributeValueResult<Self> {
        let id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let lineage_id = ctx.workspace_snapshot()?.generate_ulid().await?;
        let node_weight = NodeWeight::new_attribute_value(id, lineage_id, None, None);
        let is_for = is_for.into();

        let ordered = if let Some(prop_id) = is_for.prop_id() {
            ctx.workspace_snapshot()?
                .get_node_weight(prop_id)
                .await?
                .get_prop_node_weight()?
                .kind()
                .ordered()
        } else {
            false
        };

        if ordered {
            ctx.workspace_snapshot()?
                .add_ordered_node(node_weight.clone())
                .await?;
        } else {
            ctx.workspace_snapshot()?
                .add_or_replace_node(node_weight.clone())
                .await?;
        };

        let av: Self = node_weight.get_attribute_value_node_weight()?.into();
        match is_for {
            ValueIsFor::Prop(prop_id) => {
                Self::add_edge_to_prop(ctx, av.id, prop_id, EdgeWeightKind::Prop).await?;

                match maybe_parent_attribute_value {
                    Some(pav_id) => {
                        Self::add_edge_to_attribute_value_ordered(
                            ctx,
                            pav_id,
                            id.into(),
                            EdgeWeightKind::Contain(key),
                        )
                        .await?;
                    }
                    None => {
                        // Component --Use--> AttributeValue
                        Component::add_edge_to_root_attribute_value(
                            ctx,
                            component_id.ok_or(
                                AttributeValueError::CannotCreateRootPropValueWithoutComponentId,
                            )?,
                            id.into(),
                            EdgeWeightKind::Root,
                        )
                        .await?;
                    }
                }
            }
            is_for_socket => {
                // Attach value to component via SocketValue edge and to Socket
                if let Some(socket_id) = is_for_socket.output_socket_id() {
                    Self::add_edge_to_output_socket(ctx, av.id, socket_id, EdgeWeightKind::Socket)
                        .await?;
                } else if let Some(socket_id) = is_for_socket.input_socket_id() {
                    Self::add_edge_to_input_socket(ctx, av.id, socket_id, EdgeWeightKind::Socket)
                        .await?;
                } else {
                    return Err(AttributeValueError::UnexpectedGraphLayout(
                        "we expected a ValueIsFor for a socket type here but did not get one",
                    ));
                }

                Component::add_edge_to_socket_attribute_value(
                    ctx,
                    component_id
                        .ok_or(AttributeValueError::CannotCreateSocketValueWithoutComponentId)?,
                    id.into(),
                    EdgeWeightKind::SocketValue,
                )
                .await?;
            }
        }
        Ok(av)
    }

    /// Update the value.
    ///
    /// If this is an object, map or array value, update() will also update child values.
    #[instrument(
        name = "attribute_value.update",
        level = "info",
        skip_all,
        fields(
            attribute_value.id = ?attribute_value_id
        ))]
    pub async fn update(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) -> AttributeValueResult<()> {
        Self::vivify_value_and_parent_values(ctx, attribute_value_id).await?;
        Self::set_value(ctx, attribute_value_id, value.clone()).await?;
        Self::populate_nested_values(ctx, attribute_value_id, value).await?;

        ctx.add_dependent_values_and_enqueue(vec![attribute_value_id])
            .await?;

        Ok(())
    }

    pub async fn is_for(
        ctx: &DalContext,
        value_id: AttributeValueId,
    ) -> AttributeValueResult<ValueIsFor> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let prop_targets = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(value_id, EdgeWeightKindDiscriminants::Prop)
            .await?;

        if prop_targets.len() > 1 {
            return Err(WorkspaceSnapshotError::UnexpectedNumberOfIncomingEdges(
                EdgeWeightKindDiscriminants::Prop,
                NodeWeightDiscriminants::Content,
                value_id.into(),
            ))?;
        }

        if let Some(prop_target) = prop_targets.first().copied() {
            let prop_id = workspace_snapshot
                .get_node_weight(prop_target)
                .await?
                .get_prop_node_weight()?
                .id();
            return Ok(ValueIsFor::Prop(prop_id.into()));
        }

        let socket_targets = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(value_id, EdgeWeightKindDiscriminants::Socket)
            .await?;

        if socket_targets.len() > 1 {
            return Err(WorkspaceSnapshotError::UnexpectedNumberOfIncomingEdges(
                EdgeWeightKindDiscriminants::Socket,
                NodeWeightDiscriminants::Content,
                value_id.into(),
            ))?;
        }

        let socket_target = socket_targets
            .first()
            .ok_or(AttributeValueError::OrphanedAttributeValue(value_id))?;

        let socket_node_weight = workspace_snapshot.get_node_weight(*socket_target).await?;

        if socket_node_weight.get_input_socket_node_weight().is_ok() {
            return Ok(ValueIsFor::InputSocket(socket_node_weight.id().into()));
        }

        if let Some(output_socket) = socket_node_weight
            .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::OutputSocket)
        {
            return Ok(ValueIsFor::OutputSocket(output_socket.id().into()));
        }

        // Legacy format for InputSocket. We really shouldn't encounter this anymore.
        if let Some(input_socket) = socket_node_weight
            .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::InputSocket)
        {
            return Ok(ValueIsFor::InputSocket(input_socket.id().into()));
        }

        Err(WorkspaceSnapshotError::UnexpectedEdgeTarget(
            socket_node_weight.id(),
            value_id.into(),
            EdgeWeightKindDiscriminants::Socket,
        )
        .into())
    }

    #[instrument(
        name = "attribute_value.execute_prototype_function",
        level = "info",
        skip_all,
        fields(
            si.attribute_value.id = %attribute_value_id,
        ),
    )]
    pub async fn execute_prototype_function(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        read_lock: Arc<RwLock<()>>,
    ) -> AttributeValueResult<(FuncRunValue, Func, Vec<AttributeValueId>)> {
        // When functions are being executed in the dependent values update job,
        // we need to ensure we are not reading our input sources from a graph
        // that is in the process of being mutated on another thread, since it
        // will be incomplete (some nodes will not have all their edges added
        // yet, for example, or a reference replacement may still be in
        // progress). To handle this here, we grab a read lock, which will be
        // locked for writing in the dependent values update job while the
        // execution result is being written to the graph.

        let read_guard = read_lock.read().await;

        // Prepare arguments for prototype function execution.
        let value_is_for = Self::is_for(ctx, attribute_value_id).await?;
        let (prototype_func_id, prepared_args, input_attribute_value_ids) =
            Self::prepare_arguments_for_prototype_function_execution(ctx, attribute_value_id)
                .await?;

        let result_channel = FuncRunner::run_attribute_value(
            ctx,
            attribute_value_id,
            prototype_func_id,
            prepared_args.clone(),
        )
        .await?;

        // We have gathered all our inputs and so no longer need a lock on the graph. Be sure not to
        // add graph walk operations below this drop.
        drop(read_guard);

        let mut func_values = result_channel
            .await
            .map_err(|_| AttributeValueError::FuncRunnerSend)??;

        // If the value is for a prop, we need to make sure container-type props are initialized
        // properly when the unprocessed value is populated.
        if let ValueIsFor::Prop(prop_id) = value_is_for {
            match func_values.unprocessed_value() {
                Some(unprocessed_value) => {
                    let prop = Prop::get_by_id(ctx, prop_id).await?;
                    match prop.kind {
                        PropKind::Object | PropKind::Map => {
                            func_values.set_processed_value(Some(json!({})))
                        }
                        PropKind::Array => func_values.set_processed_value(Some(json!([]))),
                        _ => func_values.set_processed_value(Some(unprocessed_value.to_owned())),
                    }
                }
                None => func_values.set_processed_value(None),
            }
        };

        let content_value: Option<si_events::CasValue> =
            func_values.value().cloned().map(Into::into);
        let content_unprocessed_value: Option<si_events::CasValue> =
            func_values.unprocessed_value().cloned().map(Into::into);

        let value_address = match content_value {
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

        let unprocessed_value_address = match content_unprocessed_value {
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

        let func = Func::get_by_id(ctx, prototype_func_id).await?;
        if !func.is_intrinsic() {
            FuncRunner::update_run(ctx, func_values.func_run_id(), |func_run| {
                func_run.set_success(unprocessed_value_address, value_address);
            })
            .await?;
        }

        Ok((func_values, func, input_attribute_value_ids))
    }

    #[instrument(level = "debug" skip(ctx))]
    pub async fn prepare_arguments_for_prototype_function_execution(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<(FuncId, Value, Vec<AttributeValueId>)> {
        // Cache the values we need for preparing arguments for execution.
        let prototype_id = Self::prototype_id(ctx, attribute_value_id).await?;
        let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let destination_component_id = Self::component_id(ctx, attribute_value_id).await?;

        // Collect metadata for which attribute values are used to execute the prototype function for this one.
        let mut input_attribute_value_ids = Vec::new();

        // Gather the raw func bindings args into a map.
        let mut func_binding_args: HashMap<String, Vec<Value>> = HashMap::new();
        let apa_ids = AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        for apa_id in apa_ids {
            let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
            let expected_source_component_id = apa
                .targets()
                .map(|targets| targets.source_component_id)
                .unwrap_or(destination_component_id);

            if apa
                .targets()
                .is_none_or(|targets| targets.destination_component_id == destination_component_id)
            {
                // If the "source" Component is marked for deletion, and we (the destination) are
                // *NOT*, then we should ignore the argument as data should not flow from things
                // that are marked for deletion to ones that are not.
                let destination_component =
                    Component::get_by_id(ctx, destination_component_id).await?;
                let source_component =
                    Component::get_by_id(ctx, expected_source_component_id).await?;
                if source_component.to_delete() && !destination_component.to_delete() {
                    continue;
                }

                let func_arg_id =
                    AttributePrototypeArgument::func_argument_id_by_id(ctx, apa_id).await?;
                let func_arg_name = ctx
                    .workspace_snapshot()?
                    .get_node_weight(func_arg_id)
                    .await?
                    .get_func_argument_node_weight()?
                    .name()
                    .to_owned();
                let values_for_arg = match AttributePrototypeArgument::value_source(ctx, apa_id)
                    .await?
                {
                    ValueSource::ValueSubscription(subscription) => {
                        let value = match subscription.resolve(ctx).await? {
                            Some(av_id) => Self::view(ctx, av_id).await?,
                            None => None,
                        };

                        vec![value.unwrap_or(Value::Null)]
                    }
                    ValueSource::StaticArgumentValue(static_argument_value_id) => {
                        vec![
                            StaticArgumentValue::get_by_id(ctx, static_argument_value_id)
                                .await?
                                .value,
                        ]
                    }
                    ValueSource::Secret(secret_id) => {
                        vec![Secret::payload_for_prototype_execution(ctx, secret_id).await?]
                    }
                    other_source @ ValueSource::InputSocket(..)
                    | other_source @ ValueSource::OutputSocket(..)
                    | other_source @ ValueSource::Prop(..) => {
                        let mut values = vec![];

                        for av_id in other_source
                            .attribute_values_for_component_id(ctx, expected_source_component_id)
                            .await?
                        {
                            input_attribute_value_ids.push(av_id);
                            // XXX: We need to properly handle the difference between "there is
                            // XXX: no value" vs "the value is null", but right now we collapse
                            // XXX: the two to just be "null" when passing these to a function.
                            values.push(Self::view(ctx, av_id).await?.unwrap_or(Value::Null));
                        }

                        values
                    }
                };

                func_binding_args
                    .entry(func_arg_name)
                    .and_modify(|values| values.extend(values_for_arg.clone()))
                    .or_insert(values_for_arg);
            }
        }
        // if we haven't found func binding args by now, let's see if there are any inferred inputs
        // Note: a given input only takes args through inferences (i.e. frames) if it doesn't have any
        // explicitly configured args

        if func_binding_args.is_empty() {
            let inferred_inputs = Self::get_inferred_input_values(ctx, attribute_value_id).await?;

            if !inferred_inputs.is_empty() {
                let input_func = AttributePrototype::func_id(ctx, prototype_id).await?;
                if let Some(func_arg) = FuncArgument::list_for_func(ctx, input_func).await?.pop() {
                    func_binding_args.insert(func_arg.name, inferred_inputs);
                }
            }
        }

        // The value map above could possibly have multiple values per func
        // argument name if there are multiple inputs (for example, more than
        // one connection to an input socket). We need to transform these vecs
        // to a serde_json array before sending them to the function executor.
        // We also want to send a single value if there is only a single input,
        // since that is the typical case and what is expected by most attribute
        // functions.
        let mut args_map = HashMap::new();
        for (arg_name, values) in func_binding_args {
            match values.len() {
                1 => {
                    args_map.insert(arg_name, values[0].to_owned());
                }
                2.. => {
                    args_map.insert(arg_name, serde_json::to_value(values)?);
                }
                _ => {
                    return Err(
                        AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(arg_name),
                    );
                }
            }
        }

        // Serialize the raw args and we're good to go.
        let prepared_func_binding_args = serde_json::to_value(args_map)?;

        Ok((
            prototype_func_id,
            prepared_func_binding_args,
            input_attribute_value_ids,
        ))
    }

    #[instrument(level = "info", skip(ctx))]
    async fn get_inferred_input_values(
        ctx: &DalContext,
        input_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<Value>> {
        let maybe_input_socket_id = match Self::is_for(ctx, input_attribute_value_id).await? {
            ValueIsFor::InputSocket(input_socket_id) => Some(input_socket_id),
            _ => None,
        };

        let Some(input_socket_id) = maybe_input_socket_id else {
            return Ok(vec![]);
        };

        let component_id = Self::component_id(ctx, input_attribute_value_id).await?;

        let mut inputs = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut inferred_connection_graph =
            workspace_snapshot.inferred_connection_graph(ctx).await?;
        let inferred_connections = inferred_connection_graph
            .inferred_connections_for_input_socket(ctx, component_id, input_socket_id)
            .await?;
        let mut connections = Vec::with_capacity(inferred_connections.len());
        for inferred_connection in inferred_connections {
            // Both deleted and non deleted components can feed data into deleted components.
            // ** ONLY ** non-deleted components can feed data into non-deleted components
            if Component::should_data_flow_between_components(
                ctx,
                inferred_connection.destination_component_id,
                inferred_connection.source_component_id,
            )
            .await?
            {
                connections.push(inferred_connection);
            }
        }
        connections.sort_by_key(|conn| conn.source_component_id);

        for inferred_connection in connections {
            // XXX: We need to properly handle the difference between "there is
            // XXX: no value" vs "the value is null", but right now we collapse
            // XXX: the two to just be "null" when passing these to a function.
            let av_id = OutputSocket::component_attribute_value_id(
                ctx,
                inferred_connection.output_socket_id,
                inferred_connection.source_component_id,
            )
            .await?;
            let view = AttributeValue::view(ctx, av_id)
                .await?
                .unwrap_or(Value::Null);
            inputs.push(view);
        }

        Ok(inputs)
    }

    pub async fn prototype_func_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<FuncId> {
        let prototype_id = Self::prototype_id(ctx, attribute_value_id).await?;
        Ok(AttributePrototype::func_id(ctx, prototype_id).await?)
    }

    pub async fn is_set_by_dependent_function(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<bool> {
        let func_id = Self::prototype_func_id(ctx, attribute_value_id).await?;
        Ok(Func::is_dynamic(ctx, func_id).await?)
    }

    pub async fn is_set_by_unset(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<bool> {
        let func_id = Self::prototype_func_id(ctx, attribute_value_id).await?;
        let func = ctx
            .workspace_snapshot()?
            .get_node_weight(func_id)
            .await?
            .get_func_node_weight()?;
        Ok(func.name() == IntrinsicFunc::Unset.name())
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn set_values_from_func_run_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_run_value: FuncRunValue,
        func: Func,
    ) -> AttributeValueResult<()> {
        // We need to ensure the parent value tree for this value is set. But we don't want to
        // vivify the current attribute value since that would override the function which sets it
        // (and we're setting it ourselves, just below). Note that this will override the
        // prototypes for all parent values to intrinsic setters. But, a value set by an attribute
        // function other than an intrinsic setter (si:setString, etc) must not be the child of
        // *another* value set by an attribute function (other than another intrinsic setter).
        // Otherwise it would be impossible to determine the function that sets the value (two
        // functions would set it with two different sets of inputs). So vivify the parent and
        // above, but not this value.
        if let Some(parent_attribute_value_id) = Self::parent_id(ctx, attribute_value_id).await? {
            Self::vivify_value_and_parent_values(ctx, parent_attribute_value_id).await?;
        }

        let should_populate_nested = Self::prop_opt(ctx, attribute_value_id)
            .await?
            .map(|prop| prop.kind.is_container())
            .unwrap_or(false);

        let unprocessed_value = func_run_value.unprocessed_value().cloned();

        Self::set_real_values(ctx, attribute_value_id, func_run_value, func).await?;

        if should_populate_nested {
            Self::populate_nested_values(ctx, attribute_value_id, unprocessed_value).await?;
        }

        Ok(())
    }

    #[instrument(level="info" skip_all)]
    pub async fn update_from_prototype_function(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        // this lock is never locked for writing so is effectively a no-op here
        let read_lock = Arc::new(RwLock::new(()));
        // Don't need to pass in an Inferred Dependency Graph for one off updates, we can just calculate
        let (execution_result, func, _) =
            Self::execute_prototype_function(ctx, attribute_value_id, read_lock).await?;

        Self::set_values_from_func_run_value(ctx, attribute_value_id, execution_result, func)
            .await?;

        Ok(())
    }

    pub async fn component_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<ComponentId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // walk the contain edges to the root attribute value
        let mut current_attribute_value_id = attribute_value_id;
        while let Some(parent_target) = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                current_attribute_value_id,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await?
            .first()
            .copied()
        {
            current_attribute_value_id = workspace_snapshot
                .get_node_weight(parent_target)
                .await?
                .id()
                .into();
        }

        // current_attribute_value_id is now the root attribute value. Check if it has a socket
        // edge or a root edge. (Whether it is a value for a socket or for a prop)
        let component_target = match workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                current_attribute_value_id,
                EdgeWeightKindDiscriminants::Root,
            )
            .await?
            .first()
            .copied()
        {
            Some(component_target) => component_target,
            None => workspace_snapshot
                .incoming_sources_for_edge_weight_kind(
                    current_attribute_value_id,
                    EdgeWeightKindDiscriminants::SocketValue,
                )
                .await?
                .first()
                .copied()
                .ok_or(AttributeValueError::OrphanedAttributeValue(
                    current_attribute_value_id,
                ))?,
        };

        Ok(workspace_snapshot
            .get_node_weight(component_target)
            .await?
            .id()
            .into())
    }

    async fn element_prop_id_for_id(
        ctx: &DalContext,
        parent_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<PropId> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // Find the array or map prop.
        let prop_index = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                parent_attribute_value_id,
                EdgeWeightKindDiscriminants::Prop,
            )
            .await?
            .first()
            .copied()
            .ok_or(AttributeValueError::MissingPropEdge(
                parent_attribute_value_id,
            ))?;

        let prop_node_weight = workspace_snapshot
            .get_node_weight(prop_index)
            .await?
            .get_prop_node_weight()?;

        // Ensure it actually is an array or map prop.
        if prop_node_weight.kind() != PropKind::Array && prop_node_weight.kind() != PropKind::Map {
            return Err(AttributeValueError::InsertionForInvalidPropKind(
                prop_node_weight.kind(),
            ));
        }

        // Find a singular child prop for the map or an array prop (i.e. the "element" or "entry" prop").
        let prop_id = PropId::from(prop_node_weight.id());
        let child_prop_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                prop_node_weight.id(),
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;
        if child_prop_indices.len() > 1 {
            return Err(AttributeValueError::PropMoreThanOneChild(prop_id));
        }
        let element_prop_index = child_prop_indices
            .first()
            .ok_or(AttributeValueError::PropMissingElementProp(prop_id))?
            .to_owned();

        Ok(workspace_snapshot
            .get_node_weight(element_prop_index)
            .await?
            .get_prop_node_weight()?
            .clone()
            .id()
            .into())
    }

    /// Add a new element to an array.
    pub async fn insert(
        ctx: &DalContext,
        parent_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        let element_prop_id = Self::element_prop_id_for_id(ctx, parent_attribute_value_id).await?;

        // Create the "element" attribute value in the array or map alongside an attribute prototype for it.
        let new_attribute_value = Self::new(
            ctx,
            element_prop_id,
            None,
            Some(parent_attribute_value_id),
            key,
        )
        .await?;

        let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
        AttributePrototype::new(ctx, func_id).await?;

        // The element has been created an inserted. Now, we can update it with the provided value.
        Self::update(ctx, new_attribute_value.id, value).await?;

        Ok(new_attribute_value.id())
    }

    async fn vivify_value_and_parent_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let mut current_attribute_value_id = Some(attribute_value_id);

        while let Some(attribute_value_id) = current_attribute_value_id {
            let prop_kind = {
                let prop_id = match Self::is_for(ctx, attribute_value_id).await?.prop_id() {
                    Some(prop_id) => prop_id,
                    // Only prop values can be "vivified", but we don't return an error here to
                    // simplify the use of this function
                    None => return Ok(()),
                };

                let prop_node = {
                    ctx.workspace_snapshot()?
                        .get_node_weight(prop_id)
                        .await?
                        .get_prop_node_weight()?
                };

                prop_node.kind()
            };

            let attribute_value = Self::get_by_id(ctx, attribute_value_id).await?;

            // If value is for scalar, just go to parent
            if !prop_kind.is_scalar() {
                // if value of non-scalar is set, we're done, else set the empty value
                if attribute_value.value.is_some() {
                    return Ok(());
                } else {
                    Self::set_value(ctx, attribute_value_id, prop_kind.empty_value()).await?;
                }
            }

            current_attribute_value_id = Self::parent_id(ctx, attribute_value_id).await?;
        }

        Ok(())
    }

    /// Set child values of descendant AVs (maps, arrays and objects) to the corresponding JSON values.
    ///
    /// NOTE: this does not set the top-level value, only children, children's children, etc.
    /// The caller is responsible for setting the top-level value--for example, using set_value().
    async fn populate_nested_values(
        ctx: &DalContext,
        root_id: AttributeValueId,
        root_value: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        let mut work_queue = VecDeque::from([(root_id, root_value)]);

        while let Some((av_id, value)) = work_queue.pop_front() {
            let prop_id = Self::prop_id(ctx, av_id).await?;
            let child_values = match Prop::node_weight(ctx, prop_id).await?.kind() {
                PropKind::Object => {
                    Self::vivify_children_for_object_value(ctx, av_id, prop_id, value).await?
                }
                PropKind::Array => {
                    Self::vivify_children_for_array_value(ctx, av_id, prop_id, value).await?
                }
                PropKind::Map => {
                    Self::vivify_children_for_map_value(ctx, av_id, prop_id, value).await?
                }
                _ => {
                    vec![] // no children
                }
            };

            // Process children (add to the work queue or view stack)
            //
            // NOTE This could probably be more elegant! But probably not without turning the
            // whole loop inside out first, which I don't have time for right now. This centralizes
            // the logic for setting child values, at least.
            for (child_av_id, child_value) in child_values {
                // Push onto the right queue, and get the value that we will set
                let child_prop_id = Self::prop_id(ctx, child_av_id).await?;
                let child_prop_kind = Prop::node_weight(ctx, child_prop_id).await?.kind();
                let (child_value, nested_values) = match child_prop_kind {
                    // Unset objects are set to None but still enqueued so that children are created
                    PropKind::Object if child_value.is_none() => (None, Some(child_value)),

                    // Unset maps and arrays are set to None and not enqueued. They are also
                    // *not* placed on the view stack, because they will not be rendered if unset.
                    PropKind::Map | PropKind::Array if child_value.is_none() => (None, None),

                    // Objects, maps and arrays set a top-level value and break apart their child_value to
                    // give to their children.
                    PropKind::Object | PropKind::Map => (Some(json!({})), Some(child_value)),
                    PropKind::Array => (Some(json!([])), Some(child_value)),

                    // Scalar types are set directly to their value and put directly on the view stack,
                    // even if they have no value (because they will be rendered as null).
                    PropKind::Boolean
                    | PropKind::Float
                    | PropKind::Integer
                    | PropKind::Json
                    | PropKind::String => (child_value, None),
                };

                if let Some(nested_values) = nested_values {
                    work_queue.push_back((child_av_id, nested_values));
                }
                Self::set_value(ctx, child_av_id, child_value).await?;
            }
        }

        Ok(())
    }

    pub async fn map_children(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<HashMap<String, AttributeValueId>> {
        let mut result = HashMap::new();

        let snapshot = ctx.workspace_snapshot()?;

        for (edge_weight, _, target_idx) in snapshot.edges_directed(id, Outgoing).await? {
            let EdgeWeightKind::Contain(Some(key)) = edge_weight.kind() else {
                continue;
            };

            let target_id: AttributeValueId =
                snapshot.get_node_weight(target_idx).await?.id().into();
            result.insert(key.to_owned(), target_id);
        }

        Ok(result)
    }

    pub async fn map_child_opt(
        ctx: &DalContext,
        id: AttributeValueId,
        name: &str,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        Ok(Self::map_children(ctx, id).await?.get(name).copied())
    }

    pub async fn map_child(
        ctx: &DalContext,
        id: AttributeValueId,
        name: &str,
    ) -> AttributeValueResult<AttributeValueId> {
        Self::map_child_opt(ctx, id, name)
            .await?
            .ok_or(AttributeValueError::NoChildWithName(id, name.to_string()))
    }

    /// Return a hashset of all the keys contained by this attribute value (if any)
    pub async fn child_keys_for_id(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<HashSet<String>> {
        let snapshot = ctx.workspace_snapshot()?;

        Ok(snapshot
            .edges_directed_for_edge_weight_kind(id, Outgoing, EdgeWeightKindDiscriminants::Contain)
            .await?
            .iter()
            .filter_map(|(edge_weight, _, _)| {
                if let EdgeWeightKind::Contain(Some(key)) = edge_weight.kind() {
                    Some(key.to_owned())
                } else {
                    None
                }
            })
            .collect())
    }

    pub async fn object_children(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<HashMap<String, AttributeValueId>> {
        let mut result = HashMap::new();
        for child_av_id in ctx
            .workspace_snapshot()?
            .outgoing_targets_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Contain)
            .await?
        {
            result.insert(
                Self::prop(ctx, child_av_id.into()).await?.name,
                child_av_id.into(),
            );
        }
        Ok(result)
    }

    pub async fn object_child_opt(
        ctx: &DalContext,
        id: AttributeValueId,
        name: &str,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        Ok(Self::object_children(ctx, id).await?.get(name).copied())
    }

    pub async fn object_child(
        ctx: &DalContext,
        id: AttributeValueId,
        name: &str,
    ) -> AttributeValueResult<AttributeValueId> {
        Self::object_child_opt(ctx, id, name)
            .await?
            .ok_or(AttributeValueError::NoChildWithName(id, name.to_string()))
    }

    pub async fn view(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        match AttributeValue::is_for(ctx, attribute_value_id).await? {
            ValueIsFor::Prop(_) => {
                ctx.workspace_snapshot()?
                    .attribute_value_view(ctx, attribute_value_id)
                    .await
            }
            ValueIsFor::OutputSocket(_) | ValueIsFor::InputSocket(_) => {
                let attribute_value = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
                Ok(attribute_value.value(ctx).await?)
            }
        }
    }

    /// Create the immediate child AVs for an object if they don't exist, and remove any other
    /// AVs.
    ///
    /// New child AVs will be vivified in the order defined by the schema. This will not correct
    /// mis-ordered AVs.
    async fn vivify_children_for_object(
        ctx: &DalContext,
        parent_av_id: AttributeValueId,
        parent_prop_id: PropId,
    ) -> AttributeValueResult<()> {
        // Get a map of existing child_av_ids by prop
        let child_av_ids = Self::child_av_ids(ctx, parent_av_id).await?;
        let mut existing_children = HashMap::with_capacity(child_av_ids.len());
        for child_av_id in child_av_ids {
            let Some(child_prop_id) = Self::prop_id_opt(ctx, child_av_id).await? else {
                warn!(
                    "Removing child AV {child_av_id} (no prop) parent object AV {parent_av_id} (parent prop {parent_prop_id})"
                );
                Self::remove(ctx, child_av_id).await?;
                continue;
            };
            if let Some(duplicate_av_id) = existing_children.insert(child_prop_id, child_av_id) {
                warn!(
                    "Removing duplicate child AV {duplicate_av_id} (prop {child_prop_id}) from parent object AV {parent_av_id} (parent prop {parent_prop_id})"
                );
                Self::remove(ctx, duplicate_av_id).await?;
                continue;
            }
        }

        // Create child AVs that do not exist, in prop order
        for child_prop_id in Prop::direct_child_prop_ids_ordered(ctx, parent_prop_id).await? {
            if existing_children.remove(&child_prop_id).is_none() {
                Self::new(ctx, child_prop_id, None, Some(parent_av_id), None).await?;
            }
        }

        // Remove any extra child AVs that are not in the JSON object (i.e. any we didn't use).
        // (should not happen, graph fixup)
        for (child_prop_id, child_av_id) in existing_children {
            warn!(
                "Removing extra child AV {child_av_id} with (prop {child_prop_id}) from parent object AV {parent_av_id} (prop {parent_prop_id})"
            );
            Self::remove(ctx, child_av_id).await?;
        }

        Ok(())
    }

    /// Create the immediate child AVs for an object if they don't exist, and associate them
    /// with the associated value from the map. Does *not* set the values.
    ///
    /// Returns a list of all child AVs and child values that should be put in them. If the JSON
    /// does not have a value for a child prop, the AV is still created, and the value is None
    /// (unset).
    ///
    /// Creates child attribute values for all props in the object, whether there is a value
    /// present or not. Reuses any existing child attribute values.
    ///
    /// The value is treated as a complete spec: any missing fields are treated as undefined
    /// (None), and return to their default value. Extra fields in the value are ignored and
    /// thrown away. Stray AVs in the prop are removed, as well.
    async fn vivify_children_for_object_value(
        ctx: &DalContext,
        parent_av_id: AttributeValueId,
        parent_prop_id: PropId,
        value: Option<serde_json::Value>,
    ) -> AttributeValueResult<Vec<(AttributeValueId, Option<serde_json::Value>)>> {
        // vivify child AVs for each prop in the object
        Self::vivify_children_for_object(ctx, parent_av_id, parent_prop_id).await?;

        // Get the map of field names to values
        let mut field_values = match value {
            Some(Value::Object(values)) => values,
            None => Default::default(), // empty object (unsets all children on unset)
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Object,
                    serde_value_to_string_type(&value),
                ));
            }
        };

        // Associate each child AV with the corresponding JSON value from the map
        let field_av_ids = Self::child_av_ids(ctx, parent_av_id).await?;
        let mut new_children = Vec::with_capacity(field_av_ids.len());
        for field_av_id in field_av_ids {
            let field_prop_id = Self::prop_id(ctx, field_av_id).await?;
            let field_value =
                field_values.remove(Prop::node_weight(ctx, field_prop_id).await?.name());
            new_children.push((field_av_id, field_value));
        }

        Ok(new_children)
    }

    /// Set one level of children of an array to the values in a JSON array, then enqueues any
    /// nested children (i.e. array of objects or array of arrays) for further processing.
    ///
    /// This reuses existing child attribute values where possible, and creates new ones when needed.
    /// Existing child attribute values are removed if they are not in the JSON array.
    ///
    /// This only sets the first level of the values, and enqueues the children for further
    /// processing if they are arrays, maps or objects.
    async fn vivify_children_for_array_value(
        ctx: &DalContext,
        parent_av_id: AttributeValueId,
        parent_prop_id: PropId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<Vec<(AttributeValueId, Option<serde_json::Value>)>> {
        let element_prop_id = Prop::element_prop_id(ctx, parent_prop_id).await?;

        let element_values = match maybe_value {
            Some(serde_json::Value::Array(array)) => array,
            None => Default::default(), // empty array (removes all children on unset)

            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Array,
                    serde_value_to_string_type(&value),
                ));
            }
        };

        // Get an iterator of existing elements in the array so we can reuse them.
        // Any that are not reused will be removed at the end.
        let mut existing_elements = Self::get_child_av_ids_in_order(ctx, parent_av_id)
            .await?
            .into_iter();

        // Associate each child element AV with the corresponding JSON value
        let mut new_children = Vec::with_capacity(element_values.len());
        for element_value in element_values {
            // Create the AV if it doesn't exist
            let element_av_id = match existing_elements.next() {
                Some(element_av_id) => element_av_id,
                None => Self::new(ctx, element_prop_id, None, Some(parent_av_id), None)
                    .await?
                    .id(),
            };
            new_children.push((element_av_id, Some(element_value)));
        }

        // Remove unused child AVs that are not in the JSON array
        for extra_element in existing_elements {
            Self::remove(ctx, extra_element).await?;
        }

        Ok(new_children)
    }

    /// Set one level of children in a map from a JSON object, then enqueues nested children
    /// for further processing.
    ///
    /// New child attributes values are inserted in the order they appear in the JSON object.
    ///
    /// Child entry attribute values are reused where possible, and created when needed. Existing
    /// child entries are removed if they are not in the map.
    async fn vivify_children_for_map_value(
        ctx: &DalContext,
        parent_av_id: AttributeValueId,
        prop_id: PropId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<Vec<(AttributeValueId, Option<serde_json::Value>)>> {
        let snapshot = ctx.workspace_snapshot()?;

        let entry_values = match maybe_value {
            Some(Value::Object(entries)) => entries,
            None => Default::default(), // empty map (removes all children on unset)
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Map,
                    serde_value_to_string_type(&value),
                ));
            }
        };

        // Get existing map entries, removing duplicates
        let mut existing_entries = HashMap::new();
        for (edge_weight, _, target_id) in snapshot.edges_directed(parent_av_id, Outgoing).await? {
            let EdgeWeightKind::Contain(key) = edge_weight.kind else {
                continue;
            };
            let child_av_id = AttributeValueId::from(target_id);
            let Some(key) = key else {
                warn!("Removing non-map edge {child_av_id} from parent map AV {parent_av_id}");
                Self::remove(ctx, child_av_id).await?;
                continue;
            };
            if let Some(duplicate_av_id) = existing_entries.insert(key, child_av_id) {
                warn!(
                    "Removing duplicate map entry AV {duplicate_av_id} for parent map AV {parent_av_id}"
                );
                Self::remove(ctx, duplicate_av_id).await?;
                continue;
            }
        }
        let entry_prop_id = Prop::element_prop_id(ctx, prop_id).await?;

        let mut new_children = Vec::with_capacity(entry_values.len());
        for (key, entry_value) in entry_values.into_iter() {
            // Reuse the entry if it exists; add one if not
            let entry_av_id = match existing_entries.remove(&key) {
                Some(entry_av_id) => entry_av_id,
                None => Self::new(ctx, entry_prop_id, None, Some(parent_av_id), Some(key))
                    .await?
                    .id(),
            };
            new_children.push((entry_av_id, Some(entry_value)));
        }

        // Remove leftover map nodes entirely
        for av_id in existing_entries.into_values() {
            ctx.workspace_snapshot()?.remove_node_by_id(av_id).await?;
        }

        Ok(new_children)
    }

    pub async fn parent_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        Ok(
            match ctx
                .workspace_snapshot()?
                .incoming_sources_for_edge_weight_kind(
                    attribute_value_id,
                    EdgeWeightKindDiscriminants::Contain,
                )
                .await?
                .first()
                .copied()
            {
                Some(parent_idx) => Some(
                    ctx.workspace_snapshot()?
                        .get_node_weight(parent_idx)
                        .await?
                        .id()
                        .into(),
                ),
                None => None,
            },
        )
    }

    // AttributePrototypes for a value can be defined at the schema level, where
    // they are connected by a prototype edge from the prop or socket that the
    // AttributeValue is for. But they can also be defined at the component
    // level, via prototype edge outgoing from the AttributeValue to the
    // prototype. This fetches the component level prototype id, if it exists.
    pub async fn component_prototype_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        ctx.workspace_snapshot()?
            .component_prototype_id(attribute_value_id)
            .await
    }

    /// The id of the prototype that controls this attribute value at the level of the schema
    /// variant
    pub async fn schema_variant_prototype_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributePrototypeId> {
        let is_for_ulid: Ulid = Self::is_for(ctx, attribute_value_id).await?.into();
        let workspace_snapshot = ctx.workspace_snapshot()?;

        // find an incoming contain edge if any, to grab the key for this value if it is part of a map
        let mut key = None;
        for (edge_weight, _, _) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                attribute_value_id,
                Incoming,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await?
        {
            if let EdgeWeightKind::Contain(contain_key) = edge_weight.kind() {
                contain_key.clone_into(&mut key);
            }
        }

        let mut prototype_target = None;
        let mut none_prototype_target = None;
        for (edge_weight, _, target_idx) in workspace_snapshot
            .edges_directed_for_edge_weight_kind(
                is_for_ulid,
                Outgoing,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?
        {
            if let EdgeWeightKind::Prototype(prototype_key) = edge_weight.kind() {
                if &key == prototype_key {
                    prototype_target = Some(target_idx);
                    break;
                }
                if prototype_key.is_none() {
                    none_prototype_target = Some(target_idx);
                }
            }
        }

        let real_prototype_target = prototype_target.or(none_prototype_target).ok_or(
            AttributeValueError::AttributeValueMissingPrototype(attribute_value_id),
        )?;

        Ok(workspace_snapshot
            .get_node_weight(real_prototype_target)
            .await?
            .id()
            .into())
    }

    pub async fn key(&self, ctx: &DalContext) -> AttributeValueResult<Option<String>> {
        Self::key_for_id(ctx, self.id()).await
    }

    pub async fn key_for_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<String>> {
        Ok(ctx
            .workspace_snapshot()?
            .edges_directed_for_edge_weight_kind(
                attribute_value_id,
                Incoming,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await?
            .first()
            .and_then(|(edge_weight, _, _)| match edge_weight.kind() {
                EdgeWeightKind::Contain(key) => key.to_owned(),
                _ => None,
            }))
    }

    /// Returns the most specific prototype id for this attribute value. If a component specific
    /// prototype id is defined, that will be returned. Otherwise, the schema variant specific
    /// prototype id is returned.
    pub async fn prototype_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributePrototypeId> {
        let maybe_prototype_id = Self::component_prototype_id(ctx, attribute_value_id).await?;

        match maybe_prototype_id {
            Some(prototype_id) => Ok(prototype_id),
            // If there is no Prototype edge the prototype for this value is defined at the schema variant level
            None => Ok(Self::schema_variant_prototype_id(ctx, attribute_value_id).await?),
        }
    }

    /// Set's the component specific prototype id for this attribute value.
    /// Removes the existing component specific prototype if it exists.
    #[instrument(level = "debug", skip(ctx))]
    pub async fn set_component_prototype_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        attribute_prototype_id: AttributePrototypeId,
        key: Option<String>,
    ) -> AttributeValueResult<()> {
        let maybe_existing_prototype_id =
            Self::component_prototype_id(ctx, attribute_value_id).await?;

        if let Some(existing_prototype_id) = maybe_existing_prototype_id {
            AttributePrototype::remove(ctx, existing_prototype_id).await?;
        }

        Self::add_edge_to_attribute_prototype(
            ctx,
            attribute_value_id,
            attribute_prototype_id,
            EdgeWeightKind::Prototype(key),
        )
        .await?;

        Ok(())
    }
    #[instrument(level = "info", skip(ctx))]
    pub async fn use_default_prototype(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        Self::update(ctx, attribute_value_id, None).await?;
        let prototype_id = Self::component_prototype_id(ctx, attribute_value_id)
            .await?
            .ok_or(AttributeValueError::NoComponentPrototype(
                attribute_value_id,
            ))?;

        ctx.workspace_snapshot()?
            .remove_edge(
                attribute_value_id,
                prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?;

        if !Self::is_set_by_dependent_function(ctx, attribute_value_id).await? {
            Self::update_from_prototype_function(ctx, attribute_value_id).await?;
        }
        ctx.add_dependent_values_and_enqueue(vec![attribute_value_id])
            .await?;

        Ok(())
    }

    /// Set the top-level value for this AV.
    #[instrument(name = "attribute_value.set_value", level = "debug", skip_all)]
    pub async fn set_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) -> AttributeValueResult<()> {
        let mut normalized_value = value.to_owned();
        let prop_id = match Self::is_for(ctx, attribute_value_id).await? {
            ValueIsFor::Prop(prop_id) => prop_id,
            _ => {
                // Attribute values for input and output sockets should only be set by
                // functions (usually identity) since they get their values from inter-component
                // connections
                return Err(AttributeValueError::CannotExplicitlySetSocketValues(
                    attribute_value_id,
                ));
            }
        };

        let intrinsic_func = {
            let prop_node = Prop::node_weight(ctx, prop_id).await?;

            if let Some(inner_value) = &value {
                // Unfortunately, there isn't a good way to consistently track "there is no value", and "the
                // value is null" when dealing with JavaScript/JSON, so for now, we need to treat
                // "null" the same as "there is no value".
                if inner_value.is_null() {
                    normalized_value = None;

                    IntrinsicFunc::Unset
                } else {
                    prop_node.kind().intrinsic_set_func()
                }
            } else {
                // None for the value means there is no value, so we use unset, but if it's a
                // literal serde_json::Value::Null it means the value is set, but to null
                IntrinsicFunc::Unset
            }
        };
        let func_id = Func::find_intrinsic(ctx, intrinsic_func).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        let prototype = AttributePrototype::new(ctx, func_id).await?;

        Self::set_component_prototype_id(ctx, attribute_value_id, prototype.id(), None).await?;

        let func_args = match normalized_value {
            Some(value) => {
                let func_arg_id = FuncArgument::single_arg_for_func(ctx, func_id).await?;

                let func_arg_name = {
                    ctx.workspace_snapshot()?
                        .get_node_weight(func_arg_id)
                        .await?
                        .get_func_argument_node_weight()?
                        .name()
                        .to_owned()
                };

                AttributePrototypeArgument::new_static_value(
                    ctx,
                    prototype.id(),
                    func_arg_id,
                    value.to_owned(),
                )
                .await?;

                json!({ func_arg_name: value } )
            }
            None => serde_json::Value::Null,
        };

        let result_channel =
            FuncRunner::run_attribute_value(ctx, attribute_value_id, func_id, func_args).await?;
        let func_values = result_channel
            .await
            .map_err(|_| AttributeValueError::FuncRunnerSend)??;

        Self::set_real_values(ctx, attribute_value_id, func_values, func).await?;
        Ok(())
    }

    async fn set_real_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_run_value: FuncRunValue,
        func: Func,
    ) -> AttributeValueResult<()> {
        let av_node_weight = Self::node_weight(ctx, attribute_value_id).await?;

        let content_value: Option<si_events::CasValue> =
            func_run_value.value().cloned().map(Into::into);
        let content_unprocessed_value: Option<si_events::CasValue> =
            func_run_value.unprocessed_value().cloned().map(Into::into);

        let value_address = match content_value {
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

        let unprocessed_value_address = match content_unprocessed_value {
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

        if !func.is_intrinsic() {
            FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
                func_run.set_success(unprocessed_value_address, value_address);
            })
            .await?;
        }

        let mut new_av_node_weight = av_node_weight.clone();

        new_av_node_weight.set_value(value_address.map(ContentAddress::JsonValue));
        new_av_node_weight
            .set_unprocessed_value(unprocessed_value_address.map(ContentAddress::JsonValue));

        ctx.workspace_snapshot()?
            .add_or_replace_node(NodeWeight::AttributeValue(new_av_node_weight))
            .await?;

        if ValidationOutput::get_format_for_attribute_value_id(ctx, attribute_value_id)
            .await?
            .is_some()
        {
            ctx.enqueue_compute_validations(attribute_value_id).await?;
        }

        // Enqueue update actions if they exist

        // FIXME(paulo): This is wrong, we should not enqueue updates here, since this branch is triggered from DVU
        // Which would make the rebaser dispatch the update action if the DVU is running on head, without user intervention
        /*
        {
            let component_id = Self::component_id(ctx, attribute_value_id).await?;
            let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

            for prototype_id in SchemaVariant::find_action_prototypes_by_kind(
                ctx,
                schema_variant_id,
                ActionKind::Update,
            )
            .await
            .map_err(|err| AttributeValueError::Action(err.to_string()))?
            {
                if Action::find_equivalent(ctx, prototype_id, Some(component_id))
                    .await
                    .map_err(|err| AttributeValueError::Action(err.to_string()))?
                    .is_none()
                {
                    Action::new(ctx, prototype_id, Some(component_id))
                        .await
                        .map_err(|err| AttributeValueError::Action(err.to_string()))?;
                }
            }
        }
        */

        Ok(())
    }

    async fn clone_node_weight_values_from(
        ctx: &DalContext,
        dest_av_id: AttributeValueId,
        from_av_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let mut dest_node_weight = Self::node_weight(ctx, dest_av_id).await?;
        let from_node_weight = Self::node_weight(ctx, from_av_id).await?;
        dest_node_weight.set_unprocessed_value(from_node_weight.unprocessed_value());
        dest_node_weight.set_value(from_node_weight.value());
        ctx.workspace_snapshot()?
            .add_or_replace_node(NodeWeight::AttributeValue(dest_node_weight))
            .await?;

        Ok(())
    }

    pub async fn clone_value_from(
        ctx: &DalContext,
        dest_av_id: AttributeValueId,
        from_av_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        // If the old component has a non-link value (prototype), copy it over
        if let Some(from_prototype_id) = Self::component_prototype_id(ctx, from_av_id).await? {
            let from_func_id = AttributePrototype::func_id(ctx, from_prototype_id).await?;
            let dest_prototype = AttributePrototype::new(ctx, from_func_id).await?;

            for from_apa_id in
                AttributePrototypeArgument::list_ids_for_prototype(ctx, from_prototype_id).await?
            {
                let from_func_arg_id =
                    AttributePrototypeArgument::func_argument_id_by_id(ctx, from_apa_id).await?;
                let from_value_source =
                    AttributePrototypeArgument::value_source(ctx, from_apa_id).await?;

                AttributePrototypeArgument::new(
                    ctx,
                    dest_prototype.id(),
                    from_func_arg_id,
                    from_value_source,
                )
                .await?;
            }

            Self::set_component_prototype_id(ctx, dest_av_id, dest_prototype.id, None).await?;

            let sources = AttributePrototype::input_sources(ctx, dest_prototype.id).await?;
            for source in sources {
                match source {
                    AttributePrototypeSource::AttributeValue(_, _) => {
                        continue;
                    }
                    AttributePrototypeSource::Prop(prop_id, key) => {
                        Prop::add_edge_to_attribute_prototype(
                            ctx,
                            prop_id,
                            dest_prototype.id,
                            EdgeWeightKind::Prototype(key),
                        )
                        .await?;
                    }
                    AttributePrototypeSource::InputSocket(socket_id, key) => {
                        InputSocket::add_edge_to_attribute_prototype(
                            ctx,
                            socket_id,
                            dest_prototype.id,
                            EdgeWeightKind::Prototype(key),
                        )
                        .await?;
                    }
                    AttributePrototypeSource::OutputSocket(socket_id, key) => {
                        OutputSocket::add_edge_to_attribute_prototype(
                            ctx,
                            socket_id,
                            dest_prototype.id,
                            EdgeWeightKind::Prototype(key),
                        )
                        .await?;
                    }
                }
            }
        } else if let Some(existing_prototype_id) =
            Self::component_prototype_id(ctx, dest_av_id).await?
        {
            AttributePrototype::remove(ctx, existing_prototype_id).await?;
        }

        if !Self::is_set_by_dependent_function(ctx, dest_av_id).await? {
            Self::clone_node_weight_values_from(ctx, dest_av_id, from_av_id).await?;
        }

        Ok(())
    }

    /// Set the source of this attribute value to one or more subscriptions.
    ///
    /// This overwrites or overrides any existing value; if your intent is to append
    /// subscriptions, you should first call AttributeValue::subscriptions() and append to that
    /// list.
    pub async fn set_to_subscriptions(
        ctx: &DalContext,
        subscriber_av_id: AttributeValueId,
        subscriptions: Vec<ValueSubscription>,
        func_id: Option<FuncId>,
    ) -> AttributeValueResult<()> {
        let func_id = if let Some(func_id) = func_id {
            func_id
        } else {
            // TODO(victor) remove this new ui comes around
            // Pick the prototype for the function based on prop type: if it's Array, use
            // si:normalizeToArray, otherwise use si:identity
            let intrinsic_func = match Self::prop(ctx, subscriber_av_id).await?.kind {
                PropKind::Array => IntrinsicFunc::NormalizeToArray,
                kind @ PropKind::Boolean
                | kind @ PropKind::Integer
                | kind @ PropKind::Json
                | kind @ PropKind::Map
                | kind @ PropKind::Object
                | kind @ PropKind::String
                | kind @ PropKind::Float => {
                    if subscriptions.len() != 1 {
                        return Err(AttributeValueError::SingleValueMustHaveOneSubscription(
                            subscriber_av_id,
                            kind,
                            subscriptions.len(),
                        ));
                    }
                    IntrinsicFunc::Identity
                }
            };
            Func::find_intrinsic(ctx, intrinsic_func).await?
        };

        let prototype_id = AttributePrototype::new(ctx, func_id).await?.id();
        Self::set_component_prototype_id(ctx, subscriber_av_id, prototype_id, None).await?;

        // Add the subscriptions as the argument
        let arg_id = FuncArgument::single_arg_for_func(ctx, func_id).await?;
        for subscription in subscriptions {
            AttributePrototypeArgument::new(ctx, prototype_id, arg_id, subscription).await?;
        }

        // DVU all the way!
        ctx.add_dependent_values_and_enqueue(vec![subscriber_av_id])
            .await?;

        Ok(())
    }

    /// Subscriptions from this attribute value to others. If this attribute value is unset or
    /// is not set solely to subscriptions, this returns None.
    pub async fn subscriptions(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<Vec<ValueSubscription>>> {
        let Some(prototype_id) = Self::component_prototype_id(ctx, attribute_value_id).await?
        else {
            return Ok(None);
        };
        let mut subscriptions = vec![];
        for apa_id in AttributePrototype::list_arguments(ctx, prototype_id).await? {
            let ValueSource::ValueSubscription(subscription) =
                AttributePrototypeArgument::value_source(ctx, apa_id).await?
            else {
                return Ok(None);
            };
            subscriptions.push(subscription);
        }
        Ok(Some(subscriptions))
    }

    /// Subscriptions to attributes under this AV.
    pub async fn subscribers(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<impl Iterator<Item = (AttributePath, AttributePrototypeArgumentId)>>
    {
        Ok(ctx
            .workspace_snapshot()?
            .edges_directed(attribute_value_id, Direction::Incoming)
            .await?
            .into_iter()
            .filter_map(|(edge, source, _)| match edge.kind {
                EdgeWeightKind::ValueSubscription(path) => Some((path, source.into())),
                _ => None,
            }))
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Self> {
        Ok(Self::node_weight(ctx, attribute_value_id).await?.into())
    }

    pub async fn node_weight(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueNodeWeight> {
        Ok(ctx
            .workspace_snapshot()?
            .get_node_weight(attribute_value_id)
            .await?
            .get_attribute_value_node_weight()?)
    }

    pub async fn prop_opt(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<Prop>> {
        let prop_id = Self::prop_id_opt(ctx, attribute_value_id).await?;
        Ok(match prop_id {
            Some(prop_id) => Some(Prop::get_by_id(ctx, prop_id).await?),
            None => None,
        })
    }

    pub async fn prop(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<PropNodeWeight> {
        let prop_id = Self::prop_id(ctx, attribute_value_id).await?;
        Ok(Prop::node_weight(ctx, prop_id).await?)
    }

    pub async fn prop_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<PropId> {
        Self::prop_id_opt(ctx, attribute_value_id)
            .await?
            .ok_or(AttributeValueError::PropNotFound(attribute_value_id))
    }

    async fn prop_id_opt(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<PropId>> {
        let mut props = ctx
            .workspace_snapshot()?
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Prop,
            )
            .await?
            .into_iter()
            .map(PropId::from);

        let prop_id_opt = props.next();

        // Check for multiple props
        if let Some(prop_id) = prop_id_opt {
            if let Some(other_prop_id) = props.next() {
                return Err(AttributeValueError::MultiplePropsFound(
                    other_prop_id,
                    prop_id,
                    attribute_value_id,
                ));
            }
        }

        Ok(prop_id_opt)
    }

    async fn fetch_value_from_store(
        ctx: &DalContext,
        maybe_content_address: Option<ContentAddress>,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        Ok(match maybe_content_address {
            Some(value_address) => ctx
                .layer_db()
                .cas()
                .try_read_as::<si_events::CasValue>(&value_address.content_hash())
                .await?
                .map(Into::into),
            None => None,
        })
    }

    pub async fn value(&self, ctx: &DalContext) -> AttributeValueResult<Option<serde_json::Value>> {
        Self::fetch_value_from_store(ctx, self.value).await
    }

    async fn default_value(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        match Self::is_for(ctx, self.id).await? {
            ValueIsFor::Prop(prop_id) => Ok(Prop::default_value(ctx, prop_id).await?),
            ValueIsFor::InputSocket(_) | ValueIsFor::OutputSocket(_) => Ok(None),
        }
    }

    pub async fn value_or_default(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        match self.value(ctx).await? {
            Some(value) => Ok(Some(value)),
            None => self.default_value(ctx).await,
        }
    }

    pub async fn value_or_default_or_null(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> AttributeValueResult<serde_json::Value> {
        match self.value(ctx).await? {
            Some(value) => Ok(value),
            None => Ok(Prop::default_value(ctx, prop_id)
                .await?
                .unwrap_or(serde_json::Value::Null)),
        }
    }

    pub async fn unprocessed_value(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        Self::fetch_value_from_store(ctx, self.unprocessed_value).await
    }

    // Child AV IDs, ordered as they are in the graph, without reordering.
    pub async fn child_av_ids(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let snapshot = ctx.workspace_snapshot()?;
        Ok(match snapshot.ordered_children_for_node(id).await? {
            Some(children) => children.into_iter().map(Into::into).collect(),
            None => vec![],
        })
    }

    /// Get the child attribute values for this attribute value, if any exist.
    /// Returns them in order. All container values (Object, Map, Array), are
    /// ordered, so this will always return the child attribute values of a
    /// container values.
    pub async fn get_child_av_ids_in_order(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let prop = Self::prop(ctx, id).await?;
        match prop.kind {
            PropKind::Boolean
            | PropKind::Integer
            | PropKind::Float
            | PropKind::Json
            | PropKind::String => Ok(vec![]),
            PropKind::Array | PropKind::Map => Self::child_av_ids(ctx, id).await,
            // Unlike maps or arrays, we want to walk object
            // attribute values in prop order, not attribute
            // value order, so that we always return them in the
            // same order (the order the props were created for
            // the schema variant), not the order they were set
            // on the attribute value
            //
            // TODO doing this on read papers over the fact that we're storing them in an
            // order we do not prefer. We should really just ensure it's impossible to
            // create a node with these out of sync, or remove the ordering node from
            // AttributeValue object-type props altogether.
            PropKind::Object => {
                // NOTE probably can get the unordered ones if it comes down to it.
                let child_ids = Self::child_av_ids(ctx, id).await?;
                let child_prop_ids =
                    Prop::direct_child_prop_ids_ordered(ctx, prop.id.into()).await?;

                // Get the mapping from PropId -> AttributeValueId
                let mut av_prop_map = HashMap::with_capacity(child_ids.len());
                for &child_id in &child_ids {
                    let child_prop_id = Self::prop_id(ctx, child_id).await?;
                    av_prop_map.insert(child_prop_id, child_id);
                }

                // For each PropId (in schema order), look up the AttributeValueId
                let mut child_ids_in_prop_order: Vec<AttributeValueId> = child_prop_ids
                    .iter()
                    .filter_map(|child_prop_id| av_prop_map.get(child_prop_id).copied())
                    .collect();

                // Make sure we actually returned all the children
                if child_ids_in_prop_order.len() != child_ids.len() {
                    for &child_id in &child_ids {
                        if !child_ids_in_prop_order.contains(&child_id) {
                            child_ids_in_prop_order.push(child_id);
                        }
                    }
                    // TODO this appears because we're skipping above errors until the bug is fixed
                    // // Unreachable: child_ids_in_prop_order can only be <= av_prop_map.
                    // return Err(AttributeValueError::Unreachable);
                }

                Ok(child_ids_in_prop_order)
            }
        }
    }

    /// Get the child attribute values for this attribute value, if any exist.
    /// Returns them in order. All container values (Object, Map, Array), are
    /// ordered, so this will always return the child attribute values of a
    /// container values.
    pub async fn get_child_avs_in_order(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValue>> {
        let child_ids = Self::get_child_av_ids_in_order(ctx, id).await?;
        let mut child_avs = Vec::with_capacity(child_ids.len());
        for child_id in child_ids {
            child_avs.push(Self::get_by_id(ctx, child_id).await?);
        }
        Ok(child_avs)
    }

    ///
    /// Get matching child attributes, in order.
    ///
    pub async fn get_child_av_id_pairs_in_order(
        ctx: &DalContext,
        first_parent: AttributeValueId,
        second_parent: AttributeValueId,
    ) -> AttributeValueResult<Vec<ChildAttributeValuePair>> {
        // Add the children of the first parent first, in order.
        let first_children = Self::get_child_av_ids_in_order(ctx, first_parent).await?;

        // The resulting pairs
        let mut pairs: Vec<ChildAttributeValuePair> = Vec::with_capacity(first_children.len());

        // The index in `pairs` for a given key
        let mut pair_index = HashMap::<KeyOrIndex, usize>::with_capacity(first_children.len());

        // Go through
        for (index, first_child) in first_children.iter().enumerate() {
            let key = Self::key_for_id(ctx, *first_child).await?;
            let key_or_index = match &key {
                Some(key) => KeyOrIndex::Key(key.clone()),
                None => KeyOrIndex::Index(index as i64),
            };
            if let Some(old_index) = pair_index.get(&key_or_index) {
                return Err(AttributeValueError::DuplicateKeyOrIndex {
                    key_or_index,
                    // It's impossible for get() to fail here, so the or() can't happen
                    child1: *first_children.get(*old_index).unwrap_or(first_child),
                    child2: *first_child,
                });
            }
            pair_index.insert(key_or_index, pairs.len());
            pairs.push(ChildAttributeValuePair::FirstOnly(key, *first_child));
        }

        let second_children = Self::get_child_av_ids_in_order(ctx, second_parent).await?;
        for (index, second_child) in second_children.into_iter().enumerate() {
            let key = Self::key_for_id(ctx, second_child).await?;
            let key_or_index = match &key {
                Some(key) => KeyOrIndex::Key(key.clone()),
                None => KeyOrIndex::Index(index as i64),
            };
            match pair_index.get(&key_or_index) {
                None => {
                    pair_index.insert(key_or_index, pairs.len());
                    pairs.push(ChildAttributeValuePair::SecondOnly(key, second_child));
                }
                Some(index) => match pairs[*index] {
                    ChildAttributeValuePair::FirstOnly(_, first_child) => {
                        pairs[*index] =
                            ChildAttributeValuePair::Both(key, first_child, second_child);
                    }
                    ChildAttributeValuePair::SecondOnly(_, old_second_child)
                    | ChildAttributeValuePair::Both(_, _, old_second_child) => {
                        return Err(AttributeValueError::DuplicateKeyOrIndex {
                            key_or_index,
                            child1: old_second_child,
                            child2: second_child,
                        });
                    }
                },
            }
        }

        Ok(pairs)
    }

    /// Check whether this AV can be directly updated or removed by the user.
    /// This will return an error if the AV is dynamically set (if it is the child of a dynamic
    /// function).
    pub async fn ensure_updateable(
        ctx: &DalContext,
        mut id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        while let Some(parent_id) = Self::parent_id(ctx, id).await? {
            if let Some(prototype_id) = Self::component_prototype_id(ctx, parent_id).await? {
                if AttributePrototype::is_dynamic(ctx, prototype_id).await? {
                    return Err(AttributeValueError::CannotSetChildOfDynamicValue(parent_id));
                }
            }
            id = parent_id;
        }
        Ok(())
    }

    /// Remove this AV.
    pub async fn remove(ctx: &DalContext, id: AttributeValueId) -> AttributeValueResult<()> {
        let parent_av_id = Self::parent_id(ctx, id).await?;

        // If there are *direct* subscribers to this AV, they are no longer valid subscriptions
        // and should be treated the same as if the subscription existed but lead nowhere.
        // To do this, we remove the subscribing APA entirely
        //
        // NOTE: this makes the system behave slightly differently if the subscriber is passing
        // multiple subscriptions to a single argument. If the transform function is called
        // with multiple missing subscriptions, they will be passed as nulls in an array, and
        // removing the APA will remove the null. e.g. if we remove one of the subscriptions,
        // instead of calling si:normalizeToArray([null, null]), the caller will call
        // si:normalizeToArray(null). For now, we're not worrying about this distinction, as
        // the new UI never creates multiple arguments for a single subscription, and we don't
        // know if we ever want to add it back.
        for (_, subscriber_apa_id) in Self::subscribers(ctx, id).await? {
            AttributePrototypeArgument::remove(ctx, subscriber_apa_id).await?;
        }

        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;

        if let Some(parent_av_id) = parent_av_id {
            let (root_av_id, parent_path) = Self::path_from_root(ctx, parent_av_id).await?;
            let parent_path = AttributePath::JsonPointer(parent_path);

            let mut dependent_value_ids = vec![parent_av_id];

            for (subscription_path, apa_id) in Self::subscribers(ctx, root_av_id).await? {
                // If the subscription path IS the parent path, or the path to the parent's child, it's gonna be affected by the deletion of the arg.
                // We need to enqueue all the siblings of the removed too item since ordering changes on an array can affect multiple subscriptions
                if subscription_path.is_under(&parent_path) {
                    let prototype_id =
                        AttributePrototypeArgument::prototype_id(ctx, apa_id).await?;

                    let Some(subscriber_av_id) =
                        AttributePrototype::attribute_value_id(ctx, prototype_id).await?
                    else {
                        continue;
                    };

                    dependent_value_ids.push(subscriber_av_id);
                }
            }

            ctx.add_dependent_values_and_enqueue(dependent_value_ids)
                .await?;
        }

        Ok(())
    }

    pub async fn list_input_socket_sources_for_id(
        ctx: &DalContext,
        av_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<InputSocketId>> {
        let prototype_id = Self::prototype_id(ctx, av_id).await?;
        Ok(AttributePrototype::list_input_socket_sources_for_id(ctx, prototype_id).await?)
    }

    // The JSON pointer path to this attribute value, relative to its AV root.
    // Returns the root attribute value id as well as the path.
    // - for a domain prop AV: (:root_av_id, "/domain/ExposedPorts/1")
    // - for a socket AV, it returns the current id and empty path: (:id, "/")
    #[instrument(level = "debug", skip_all)]
    pub async fn path_from_root(
        ctx: &DalContext,
        mut child_id: AttributeValueId,
    ) -> AttributeValueResult<(AttributeValueId, String)> {
        let mut pointer = jsonptr::PointerBuf::new();
        while let Some((parent_id, key)) = Self::parent_and_map_key(ctx, child_id).await? {
            // Only props can have child AVs at the moment.
            match Self::prop(ctx, parent_id).await?.kind {
                PropKind::Object => {
                    let child_prop = Self::prop(ctx, child_id).await?;
                    pointer.push_front(child_prop.name)
                }
                PropKind::Map => {
                    let Some(key) = key else {
                        return Err(AttributeValueError::MissingKeyForMapEntry(child_id));
                    };
                    pointer.push_front(key)
                }
                PropKind::Array => {
                    let index = Self::child_array_index(ctx, parent_id, child_id).await?;
                    pointer.push_front(index)
                }
                parent_kind @ PropKind::Boolean
                | parent_kind @ PropKind::Float
                | parent_kind @ PropKind::Integer
                | parent_kind @ PropKind::Json
                | parent_kind @ PropKind::String => {
                    return Err(AttributeValueError::ChildOfScalar {
                        parent_id,
                        child_id,
                        parent_kind,
                    });
                }
            };
            child_id = parent_id;
        }
        Ok((child_id, pointer.to_string()))
    }

    /// This includes the key/index in the path, unlike the [`PropPath`] which doesn't
    /// include the key/index
    #[instrument(level = "debug", skip_all)]
    pub async fn get_path_for_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<String>> {
        let mut parts = VecDeque::new();
        let mut work_queue = VecDeque::from([attribute_value_id]);

        while let Some(mut attribute_value_id) = work_queue.pop_front() {
            let attribute_path = match Self::is_for(ctx, attribute_value_id).await? {
                ValueIsFor::Prop(prop_id) => {
                    let prop_name = Prop::get_by_id(ctx, prop_id).await?.name;
                    // check the parent of this attribute value
                    // if the parent is an array or map, we need to add the key/index to the attribute value path
                    if let Some(parent_attribute_value_id) =
                        Self::parent_id(ctx, attribute_value_id).await?
                    {
                        let key_or_index =
                            Self::get_key_or_index_of_child_entry(ctx, attribute_value_id).await?;

                        attribute_value_id = parent_attribute_value_id;
                        work_queue.push_back(attribute_value_id);
                        AttributeValuePath::Prop {
                            path: prop_name,
                            key_or_index,
                        }
                    } else {
                        AttributeValuePath::Prop {
                            path: prop_name,
                            key_or_index: None,
                        }
                    }
                }
                ValueIsFor::InputSocket(input_socket_id) => {
                    let input_socket_name = InputSocket::get_by_id(ctx, input_socket_id)
                        .await?
                        .name()
                        .to_string();
                    AttributeValuePath::InputSocket(input_socket_name)
                }
                ValueIsFor::OutputSocket(output_socket_id) => {
                    let output_socket_name = OutputSocket::get_by_id(ctx, output_socket_id)
                        .await?
                        .name()
                        .to_string();
                    AttributeValuePath::OutputSocket(output_socket_name)
                }
            };
            parts.push_front(attribute_path);
        }
        if !parts.is_empty() {
            Ok(Some(
                AttributeValuePath::assemble_from_parts_with_separator(parts, Some("/")),
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn get_key_or_index_of_child_entry(
        ctx: &DalContext,
        child_id: AttributeValueId,
    ) -> AttributeValueResult<Option<KeyOrIndex>> {
        Ok(match Self::parent_and_map_key(ctx, child_id).await? {
            Some((pav_id, map_key)) => match Self::is_for(ctx, pav_id).await? {
                ValueIsFor::Prop(prop_id) => match Prop::get_by_id(ctx, prop_id).await?.kind {
                    PropKind::Array => {
                        match ctx
                            .workspace_snapshot()?
                            .ordered_children_for_node(pav_id)
                            .await?
                        {
                            Some(order) => {
                                let index =
                                    order.iter().position(|id| *id == child_id.into()).ok_or(
                                        NodeWeightError::MissingKeyForChildEntry(child_id.into()),
                                    )?;

                                Some(KeyOrIndex::Index(index as i64))
                            }
                            None => None,
                        }
                    }
                    PropKind::Map => map_key.map(KeyOrIndex::Key),
                    _ => None,
                },
                _ => None,
            },
            None => None,
        })
    }

    // Get the parent attribute value id and optional map key (if it's a child of a map)
    async fn parent_and_map_key(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Option<(AttributeValueId, Option<String>)>> {
        for (edge, source, _) in ctx
            .workspace_snapshot()?
            .edges_directed(id, Direction::Incoming)
            .await?
        {
            if let EdgeWeightKind::Contain(key) = edge.kind {
                return Ok(Some((source.into(), key)));
            }
        }
        Ok(None)
    }

    async fn child_array_index(
        ctx: &DalContext,
        parent_id: AttributeValueId,
        child_id: AttributeValueId,
    ) -> AttributeValueResult<usize> {
        ctx.workspace_snapshot()?
            .ordered_children_for_node(parent_id)
            .await?
            .ok_or(AttributeValueError::NoOrderingNodeForAttributeValue(
                parent_id,
            ))?
            .iter()
            .position(|&id| id == child_id.into())
            .ok_or(AttributeValueError::ElementMissingFromOrderingNode(
                child_id,
            ))
    }

    pub async fn tree_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> AttributeValueResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
        let mut child_values = HashMap::new();
        // Get the root attribute value and load it into the work queue.
        let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;

        let mut work_queue = VecDeque::from([root_attribute_value_id]);
        while let Some(attribute_value_id) = work_queue.pop_front() {
            let children = Self::get_child_av_ids_in_order(ctx, attribute_value_id).await?;
            child_values.insert(attribute_value_id, children.clone());

            // Load the work queue with the child attribute value.
            work_queue.extend(children);
        }
        Ok(child_values)
    }

    /// Walk the tree below `id` and gather up all children if the children are
    /// children of an object. The returned list is in breadth-first pre-order
    pub async fn all_object_children_to_leaves(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let mut values = vec![];
        let mut work_queue = VecDeque::from([id]);

        while let Some(attribute_value_id) = work_queue.pop_front() {
            if let ValueIsFor::Prop(prop_id) = Self::is_for(ctx, attribute_value_id).await? {
                let prop = Prop::get_by_id(ctx, prop_id).await?;
                if prop.kind == PropKind::Object {
                    for child_value_id in
                        Self::get_child_av_ids_in_order(ctx, attribute_value_id).await?
                    {
                        values.push(child_value_id);
                        work_queue.push_back(child_value_id);
                    }
                }
            }
        }

        Ok(values)
    }
}
