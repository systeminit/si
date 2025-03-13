// FIXME(nick): restore this module comment with the new paradigm.
// An [`AttributeValue`] represents which [`FuncBinding`](crate::func::binding::FuncBinding)
// and [`FuncBindingReturnValue`] provide attribute's value. Moreover, it tracks whether the
// value is proxied or not. Proxied values "point" to another [`AttributeValue`] to provide
// the attribute's value.
//
// ## Updating [`AttributeValues`](AttributeValue)
//
// Let's say you want to update a
// [`PropertyEditorValue`](crate::property_editor::values::PropertyEditorValue) in the UI or a
// "field" on a [`Component`](crate::Component) in general. The key to doing so is the following
// process:
//
// 1) Find the appropriate [`AttributeValue`] in a [`context`](crate::AttributeContext) that is
//   either "exactly specific" to what you need or "less specific" than what you need (see the
//   [`module`](crate::attribute::context) for more information)
// 2) Find its parent, which almost all [`AttributeValues`](AttributeValue) should have if they are
//   in the lineage of a [`RootProp`](crate::RootProp) (usually, the
//   [`standard model accessor`](crate::standard_accessors) that contains the parent will suffice
//   in finding the parent)
// 3) Use [`AttributeValue::update_for_context()`] with the appropriate key and
//   [`context`](crate::AttributeContext) while ensuring that if you reuse the key and/or
//   [`context`](crate::AttributeContext) from the [`AttributeValue`](crate::AttributeValue)
//   that you found, that it is _exactly_ what you need (i.e. if the key changes or the
//   [`context`](crate::AttributeContext) is in a lesser specificity than what you need, you
//   mutate them accordingly)
//
// Often, you may not have all the information necessary to find the [`AttributeValue`] that you
// would like to update. Ideally, you would use one of the existing accessor methods off
// [`AttributeValue`] with contextual information such as a [`PropId`](crate::Prop),
// a [`ComponentId`](crate::Component)), a parent [`AttributeValue`], a key, etc.
//
// In situations where we do not have minimal information to find the _correct_ [`AttributeValue`]
// from existing accessor queries, we can leveraging existing queries from other structs and write
// new queries for those structs and specific use cases. For example, since most members of the
// [`RootProp`](crate::RootProp) tree are stable across [`SchemaVariants`](crate::SchemaVariant),
// we can use [`Component::root_prop_child_attribute_value_for_component()`](crate::Component::root_prop_child_attribute_value_for_component)
// to find the [`AttributeValue`] whose [`context`](crate::AttributeContext) corresponds to a
// direct child [`Prop`](crate::Prop) of the [`RootProp`](crate::RootProp).

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use async_recursion::async_recursion;
use indexmap::IndexMap;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use si_events::ulid::Ulid;
use si_events::FuncRunValue;
use si_pkg::{AttributeValuePath, KeyOrIndex};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::{RwLock, TryLockError};

pub use dependent_value_graph::DependentValueGraph;

use crate::attribute::prototype::{AttributePrototypeError, AttributePrototypeSource};
use crate::change_set::ChangeSetError;
use crate::component::inferred_connection_graph::InferredConnectionGraphError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::runner::{FuncRunner, FuncRunnerError};
use crate::func::FuncExecutionPk;
use crate::prop::PropError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::validation::{ValidationError, ValidationOutput};
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{EdgeWeightKind, EdgeWeightKindDiscriminants};
use crate::workspace_snapshot::node_weight::{
    AttributeValueNodeWeight, NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::{serde_value_to_string_type, WorkspaceSnapshotError};
use crate::{
    implement_add_edge_to, AttributePrototype, AttributePrototypeId, Component, ComponentError,
    ComponentId, DalContext, Func, FuncError, FuncId, HelperError, InputSocket, InputSocketId,
    OutputSocket, OutputSocketId, Prop, PropId, PropKind, Secret, SecretError, TransactionsError,
};

use super::prototype::argument::static_value::StaticArgumentValue;
use super::prototype::argument::value_source::ValueSourceError;
use super::prototype::argument::{
    value_source::ValueSource, AttributePrototypeArgument, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId,
};

pub use is_for::ValueIsFor;

pub mod debug;
pub mod dependent_value_graph;
pub mod is_for;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AttributeValueError {
    #[error("action error: {0}")]
    Action(String),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype argument {0} has a value source {1:?} but no value for that prop found in component {2}")]
    AttributePrototypeArgumentMissingValueInSourceComponent(
        AttributePrototypeArgumentId,
        ValueSource,
        ComponentId,
    ),
    #[error("attribute prototype argument {0} has no value source")]
    AttributePrototypeArgumentMissingValueSource(AttributePrototypeArgumentId),
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
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("duplicate key or index {key_or_index} for attribute values {child1} and {child2}")]
    DuplicateKeyOrIndex {
        key_or_index: KeyOrIndex,
        child1: AttributeValueId,
        child2: AttributeValueId,
    },
    #[error("empty attribute prototype arguments for group name: {0}")]
    EmptyAttributePrototypeArgumentsForGroup(String),
    #[error("object field is not a child prop of the object prop: {0}")]
    FieldNotChildOfObject(AttributeValueId),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
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
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("cannot insert for prop kind: {0}")]
    InsertionForInvalidPropKind(PropKind),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing attribute prototype argument source: {0}")]
    MissingAttributePrototypeArgumentSource(AttributePrototypeArgumentId),
    #[error("missing attribute value with id: {0}")]
    MissingForId(AttributeValueId),
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
    #[error("no component prototype found for attribute value: {0}")]
    NoComponentPrototype(AttributeValueId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight mismatch, expected {0:?} to be {1:?}")]
    NodeWeightMismatch(NodeIndex, NodeWeightDiscriminants),
    #[error("attribute value does not have ordering node as expected: {0}")]
    NoOrderingNodeForAttributeValue(AttributeValueId),
    #[error("attribute value not found for component ({0}) and input socket ({1})")]
    NotFoundForComponentAndInputSocket(ComponentId, InputSocketId),
    #[error("attribute value {0} has no outgoing edge to a prop or socket")]
    OrphanedAttributeValue(AttributeValueId),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] OutputSocketError),
    #[error("parent prop of map or array not found: {0}")]
    ParentAttributeValueMissing(AttributeValueId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("array or map prop missing element prop: {0}")]
    PropMissingElementProp(PropId),
    #[error("array or map prop has more than one child prop: {0}")]
    PropMoreThanOneChild(PropId),
    #[error("prop not found for attribute value: {0}")]
    PropNotFound(AttributeValueId),
    #[error("trying to delete av that's not related to child of map or array: {0}")]
    RemovingWhenNotChildOrMapOrArray(AttributeValueId),
    #[error("secret error: {0}")]
    Secret(#[from] Box<SecretError>),
    #[error("serde_json: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
    Validation(#[from] ValidationError),
    #[error("value source error: {0}")]
    ValueSource(#[from] ValueSourceError),
    #[error("workspace error: {0}")]
    Workspace(String),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

impl From<ComponentError> for AttributeValueError {
    fn from(value: ComponentError) -> Self {
        Self::Component(Box::new(value))
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
                .get_node_weight_by_id(prop_id)
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
        .await
        .map_err(Box::new)?;

        // We have gathered all our inputs and so no longer need a lock on the graph. Be sure not to
        // add graph walk operations below this drop.
        drop(read_guard);

        let mut func_values = result_channel
            .await
            .map_err(|_| AttributeValueError::FuncRunnerSend)?
            .map_err(Box::new)?;

        // If the value is for a prop, we need to make sure container-type props are initialized
        // properly when the unprocessed value is populated.
        if let ValueIsFor::Prop(prop_id) = value_is_for {
            match func_values.unprocessed_value() {
                Some(unprocessed_value) => {
                    let prop = Prop::get_by_id(ctx, prop_id).await?;
                    match prop.kind {
                        PropKind::Object | PropKind::Map => {
                            func_values.set_processed_value(Some(serde_json::json!({})))
                        }
                        PropKind::Array => {
                            func_values.set_processed_value(Some(serde_json::json!([])))
                        }
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

        let func = Func::get_by_id_or_error(ctx, prototype_func_id).await?;
        if !func.is_intrinsic() {
            ctx.layer_db()
                .func_run()
                .set_values_and_set_state_to_success(
                    func_values.func_run_id(),
                    unprocessed_value_address,
                    value_address,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
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

            if apa.targets().map_or(true, |targets| {
                targets.destination_component_id == destination_component_id
            }) {
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
                    .get_node_weight_by_id(func_arg_id)
                    .await?
                    .get_func_argument_node_weight()?
                    .name()
                    .to_owned();
                let values_for_arg =
                    match AttributePrototypeArgument::value_source_by_id(ctx, apa_id)
                        .await?
                        .ok_or(
                            AttributeValueError::AttributePrototypeArgumentMissingValueSource(
                                apa_id,
                            ),
                        )? {
                        ValueSource::StaticArgumentValue(static_argument_value_id) => {
                            vec![
                                StaticArgumentValue::get_by_id(ctx, static_argument_value_id)
                                    .await?
                                    .value,
                            ]
                        }
                        ValueSource::Secret(secret_id) => {
                            vec![Secret::payload_for_prototype_execution(ctx, secret_id)
                                .await
                                .map_err(Box::new)?]
                        }
                        other_source => {
                            let mut values = vec![];

                            for av_id in other_source
                                .attribute_values_for_component_id(
                                    ctx,
                                    expected_source_component_id,
                                )
                                .await?
                            {
                                input_attribute_value_ids.push(av_id);
                                let attribute_value = AttributeValue::get_by_id(ctx, av_id).await?;
                                // XXX: We need to properly handle the difference between "there is
                                // XXX: no value" vs "the value is null", but right now we collapse
                                // XXX: the two to just be "null" when passing these to a function.
                                values
                                    .push(attribute_value.view(ctx).await?.unwrap_or(Value::Null));
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
        let maybe_input_socket_id =
            match AttributeValue::is_for(ctx, input_attribute_value_id).await? {
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
        let mut connections = Vec::new();
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
            let output_av = AttributeValue::get_by_id(
                ctx,
                OutputSocket::component_attribute_value_for_output_socket_id(
                    ctx,
                    inferred_connection.output_socket_id,
                    inferred_connection.source_component_id,
                )
                .await?,
            )
            .await?;
            let view = output_av.view(ctx).await?.unwrap_or(Value::Null);
            inputs.push(view);
        }

        Ok(inputs)
    }

    pub async fn prototype_func(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Func> {
        let prototype_id = Self::prototype_id(ctx, attribute_value_id).await?;
        let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        Ok(Func::get_by_id_or_error(ctx, prototype_func_id).await?)
    }

    pub async fn is_set_by_dependent_function(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<bool> {
        Ok(Self::prototype_func(ctx, attribute_value_id)
            .await?
            .is_dynamic())
    }

    pub async fn is_set_by_unset(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<bool> {
        Ok(
            Self::prototype_func(ctx, attribute_value_id).await?.name
                == IntrinsicFunc::Unset.name(),
        )
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
        if let Some(parent_attribute_value_id) =
            Self::parent_attribute_value_id(ctx, attribute_value_id).await?
        {
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
            AttributeValue::execute_prototype_function(ctx, attribute_value_id, read_lock).await?;

        AttributeValue::set_values_from_func_run_value(
            ctx,
            attribute_value_id,
            execution_result,
            func,
        )
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
                let prop_id = match AttributeValue::is_for(ctx, attribute_value_id)
                    .await?
                    .prop_id()
                {
                    Some(prop_id) => prop_id,
                    // Only prop values can be "vivified", but we don't return an error here to
                    // simplify the use of this function
                    None => return Ok(()),
                };

                let prop_node = {
                    ctx.workspace_snapshot()?
                        .get_node_weight_by_id(prop_id)
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

            current_attribute_value_id =
                AttributeValue::parent_attribute_value_id(ctx, attribute_value_id).await?;
        }

        Ok(())
    }

    async fn create_nested_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        func_id: FuncId,
        prop_id: PropId,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        let prop_kind = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            let prop_node_index = workspace_snapshot.get_node_index_by_id(prop_id).await?;
            if let NodeWeight::Prop(prop_inner) =
                workspace_snapshot.get_node_weight(prop_node_index).await?
            {
                prop_inner.kind()
            } else {
                return Err(AttributeValueError::NodeWeightMismatch(
                    prop_node_index,
                    NodeWeightDiscriminants::Prop,
                ));
            }
        };

        let new_attribute_value =
            Self::new(ctx, prop_id, None, Some(attribute_value_id), key).await?;

        AttributePrototype::new(ctx, func_id).await?;

        match prop_kind {
            PropKind::Object | PropKind::Map => {
                Self::set_value(
                    ctx,
                    new_attribute_value.id,
                    if value.is_some() {
                        Some(serde_json::json!({}))
                    } else {
                        None
                    },
                )
                .await?;
            }
            PropKind::Array => {
                Self::set_value(
                    ctx,
                    new_attribute_value.id,
                    if value.is_some() {
                        Some(serde_json::json!([]))
                    } else {
                        None
                    },
                )
                .await?;
            }
            _ => {
                Self::set_value(ctx, new_attribute_value.id, value).await?;
            }
        }

        Ok(new_attribute_value.id)
    }

    pub async fn order(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<Vec<AttributeValueId>>> {
        Ok(ctx
            .workspace_snapshot()?
            .ordering_node_for_container(self.id())
            .await?
            .map(|node| node.order().clone().into_iter().map(Into::into).collect()))
    }

    async fn populate_nested_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        // Cache the unset func id before getting the workspace snapshot.
        let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;
        // Remove child attribute value edges
        for attribute_value_target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await?
        {
            let current_target_id = workspace_snapshot
                .get_node_weight(attribute_value_target)
                .await?
                .id();

            workspace_snapshot
                .remove_node_by_id(current_target_id)
                .await?;
        }

        let mut work_queue = VecDeque::from([(attribute_value_id, value)]);

        let mut view_stack = Vec::new();

        while let Some((attribute_value_id, maybe_value)) = work_queue.pop_front() {
            let (prop_kind, prop_id) = {
                let prop_id = Self::is_for(ctx, attribute_value_id)
                    .await?
                    .prop_id()
                    .ok_or(
                        AttributeValueError::CannotCreateNestedValuesForNonPropValues(
                            attribute_value_id,
                        ),
                    )?;
                let prop = Prop::get_by_id(ctx, prop_id).await?;

                (prop.kind, prop_id)
            };

            view_stack.push(attribute_value_id);

            let (work_queue_extension, view_stack_extension) = match prop_kind {
                PropKind::Object => {
                    Self::process_populate_nested_values_for_object(
                        ctx,
                        prop_id,
                        attribute_value_id,
                        unset_func_id,
                        maybe_value,
                    )
                    .await?
                }
                PropKind::Array => {
                    Self::process_populate_nested_values_for_array(
                        ctx,
                        prop_id,
                        attribute_value_id,
                        unset_func_id,
                        maybe_value,
                    )
                    .await?
                }
                PropKind::Map => {
                    Self::process_populate_nested_values_for_map(
                        ctx,
                        prop_id,
                        attribute_value_id,
                        unset_func_id,
                        maybe_value,
                    )
                    .await?
                }
                _ => continue,
            };

            // Extend the work queue by what was found when processing the container, if applicable.
            work_queue.extend(work_queue_extension);
            view_stack.extend(view_stack_extension);
        }

        Ok(())
    }

    pub async fn map_children(
        ctx: &DalContext,
        map_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<HashMap<String, AttributeValueId>> {
        let mut result = HashMap::new();

        let snapshot = ctx.workspace_snapshot()?;

        for (edge_weight, _, target_idx) in snapshot
            .edges_directed(map_attribute_value_id, Outgoing)
            .await?
            .into_iter()
        {
            let EdgeWeightKind::Contain(Some(key)) = edge_weight.kind() else {
                continue;
            };

            let target_id: AttributeValueId =
                snapshot.get_node_weight(target_idx).await?.id().into();
            result.insert(key.to_owned(), target_id);
        }

        Ok(result)
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

    #[async_recursion]
    pub async fn view(&self, ctx: &DalContext) -> AttributeValueResult<Option<serde_json::Value>> {
        let attribute_value_id = self.id;

        match AttributeValue::is_for(ctx, attribute_value_id).await? {
            ValueIsFor::Prop(prop_id) => {
                let self_value = self.value(ctx).await?;
                if self_value.is_none() {
                    // If we are None, but our prototype hasn't been overridden
                    // for this component, look for a default value
                    if Self::component_prototype_id(ctx, self.id).await?.is_none() {
                        return Ok(Prop::default_value(ctx, prop_id).await?);
                    }
                    return Ok(None);
                }

                let prop = Prop::get_by_id(ctx, prop_id).await?;

                match prop.kind {
                    PropKind::Object => {
                        let mut object_view: IndexMap<String, serde_json::Value> = IndexMap::new();

                        for child_av in
                            Self::get_child_avs_in_order(ctx, attribute_value_id).await?
                        {
                            if let Some(view) = child_av.view(ctx).await? {
                                let prop = Self::prop(ctx, child_av.id).await?;
                                object_view.insert(prop.name, view);
                            }
                        }

                        Ok(Some(serde_json::to_value(object_view)?))
                    }
                    PropKind::Map => {
                        let mut map_view = IndexMap::new();

                        for child_av in
                            Self::get_child_avs_in_order(ctx, attribute_value_id).await?
                        {
                            if let Some(key) = child_av.key(ctx).await? {
                                if let Some(view) = child_av.view(ctx).await? {
                                    map_view.insert(key.to_owned(), view);
                                }
                            }
                        }

                        Ok(Some(serde_json::to_value(map_view)?))
                    }
                    PropKind::Array => {
                        let mut array_view = Vec::new();

                        for element_av in
                            Self::get_child_avs_in_order(ctx, attribute_value_id).await?
                        {
                            if let Some(view) = element_av.view(ctx).await? {
                                array_view.push(view);
                            }
                        }

                        Ok(Some(serde_json::to_value(array_view)?))
                    }
                    _ => Ok(self_value),
                }
            }
            ValueIsFor::OutputSocket(_) | ValueIsFor::InputSocket(_) => Ok(self.value(ctx).await?),
        }
    }

    async fn process_populate_nested_values_for_object(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_value_id: AttributeValueId,
        unset_func_id: FuncId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<(
        VecDeque<(AttributeValueId, Option<Value>)>,
        Vec<AttributeValueId>,
    )> {
        let maybe_object_map = match maybe_value {
            Some(Value::Object(map)) => Some(map),
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Object,
                    serde_value_to_string_type(&value),
                ));
            }
            None => None,
        };

        let prop_map = {
            let child_prop_indexes = ctx
                .workspace_snapshot()?
                .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)
                .await?;

            let mut prop_map = HashMap::new();
            for node_index in child_prop_indexes {
                if let NodeWeight::Prop(prop_inner) = ctx
                    .workspace_snapshot()?
                    .get_node_weight(node_index)
                    .await?
                {
                    prop_map.insert(
                        prop_inner.name().to_string(),
                        (prop_inner.id(), prop_inner.kind()),
                    );
                }
            }
            prop_map
        };

        // Remove keys from our value if there is no corresponding child prop
        let maybe_object_map = maybe_object_map.map(|mut map| {
            map.retain(|k, _| prop_map.contains_key(k));
            map
        });

        let mut view_stack_extension = vec![];
        let mut work_queue_extension = VecDeque::new();
        for (key, (prop_id, prop_kind)) in prop_map.into_iter() {
            let field_value = maybe_object_map
                .as_ref()
                .and_then(|map| map.get(&key).cloned());

            let new_attribute_value_id = Self::create_nested_value(
                ctx,
                attribute_value_id,
                field_value.clone(),
                unset_func_id,
                PropId::from(prop_id),
                None,
            )
            .await?;

            match prop_kind {
                PropKind::Array | PropKind::Map => {
                    if field_value.is_some() {
                        work_queue_extension.push_back((new_attribute_value_id, field_value));
                    }
                }
                PropKind::Object => {
                    work_queue_extension.push_back((new_attribute_value_id, field_value));
                }
                _ => view_stack_extension.push(new_attribute_value_id),
            }
        }
        Ok((work_queue_extension, view_stack_extension))
    }

    async fn process_populate_nested_values_for_array(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_value_id: AttributeValueId,
        unset_func_id: FuncId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<(
        VecDeque<(AttributeValueId, Option<Value>)>,
        Vec<AttributeValueId>,
    )> {
        let mut work_queue_extension = VecDeque::new();
        let mut view_stack_extension = vec![];

        let array_items = match maybe_value {
            Some(serde_json::Value::Array(array)) => {
                if array.is_empty() {
                    return Ok((work_queue_extension, view_stack_extension));
                }
                array
            }
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Array,
                    serde_value_to_string_type(&value),
                ));
            }
            None => return Ok((work_queue_extension, view_stack_extension)),
        };

        let (element_prop_id, element_prop_kind) = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            // find the child element prop
            let child_props = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)
                .await?;

            if child_props.len() > 1 {
                return Err(AttributeValueError::PropMoreThanOneChild(prop_id));
            }

            let element_prop_index = child_props
                .first()
                .ok_or(AttributeValueError::PropMissingElementProp(prop_id))?
                .to_owned();

            match workspace_snapshot
                .get_node_weight(element_prop_index)
                .await?
            {
                NodeWeight::Prop(prop_inner) => (prop_inner.id(), prop_inner.kind()),
                _ => {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        element_prop_index,
                        NodeWeightDiscriminants::Prop,
                    ));
                }
            }
        };

        for array_item in array_items {
            // TODO: should we type check the values here against the element prop?
            let array_item_value = Some(array_item);
            let new_attribute_value_id = Self::create_nested_value(
                ctx,
                attribute_value_id,
                array_item_value.clone(),
                unset_func_id,
                PropId::from(element_prop_id),
                None,
            )
            .await?;

            match element_prop_kind {
                PropKind::Array | PropKind::Map => {
                    if array_item_value.is_some() {
                        work_queue_extension.push_back((new_attribute_value_id, array_item_value));
                    }
                }
                PropKind::Object => {
                    work_queue_extension.push_back((new_attribute_value_id, array_item_value));
                }
                _ => view_stack_extension.push(new_attribute_value_id),
            }
        }

        Ok((work_queue_extension, view_stack_extension))
    }

    pub async fn parent_attribute_value_id(
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
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let maybe_prototype_idx = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?
            .first()
            .copied();

        Ok(match maybe_prototype_idx {
            Some(prototype_idx) => Some(
                workspace_snapshot
                    .get_node_weight(prototype_idx)
                    .await?
                    .id()
                    .into(),
            ),
            None => None,
        })
    }

    /// The id of the prototype that controls this attribute value at the level of the schema
    /// variant
    pub async fn schema_variant_prototype_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributePrototypeId> {
        let is_for_ulid: Ulid = AttributeValue::is_for(ctx, attribute_value_id)
            .await?
            .into();
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
        let maybe_prototype_id =
            AttributeValue::component_prototype_id(ctx, attribute_value_id).await?;

        match maybe_prototype_id {
            Some(prototype_id) => Ok(prototype_id),
            // If there is no Prototype edge the prototype for this value is defined at the schema variant level
            None => Ok(AttributeValue::schema_variant_prototype_id(ctx, attribute_value_id).await?),
        }
    }

    async fn process_populate_nested_values_for_map(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_value_id: AttributeValueId,
        unset_func_id: FuncId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<(
        VecDeque<(AttributeValueId, Option<Value>)>,
        Vec<AttributeValueId>,
    )> {
        let mut work_queue_extension = VecDeque::new();
        let mut view_stack_extension = vec![];

        let map_map = match maybe_value {
            Some(Value::Object(map)) => {
                if map.is_empty() {
                    return Ok((work_queue_extension, view_stack_extension));
                }
                map
            }
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Map,
                    serde_value_to_string_type(&value),
                ));
            }
            None => return Ok((work_queue_extension, view_stack_extension)),
        };

        let (element_prop_id, element_prop_kind) = {
            let workspace_snapshot = ctx.workspace_snapshot()?;

            // find the child element prop
            let child_props = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)
                .await?;

            if child_props.len() > 1 {
                return Err(AttributeValueError::PropMoreThanOneChild(prop_id));
            }

            let element_prop_index = child_props
                .first()
                .ok_or(AttributeValueError::PropMissingElementProp(prop_id))?
                .to_owned();

            match workspace_snapshot
                .get_node_weight(element_prop_index)
                .await?
            {
                NodeWeight::Prop(prop_inner) => (prop_inner.id(), prop_inner.kind()),
                _ => {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        element_prop_index,
                        NodeWeightDiscriminants::Prop,
                    ));
                }
            }
        };

        for (key, value) in map_map.into_iter() {
            let value = Some(value);
            let new_attribute_value_id = Self::create_nested_value(
                ctx,
                attribute_value_id,
                value.clone(),
                unset_func_id,
                PropId::from(element_prop_id),
                Some(key.to_owned()),
            )
            .await?;

            match element_prop_kind {
                PropKind::Array | PropKind::Map => {
                    if value.is_some() {
                        work_queue_extension.push_back((new_attribute_value_id, value));
                    }
                }
                PropKind::Object => {
                    work_queue_extension.push_back((new_attribute_value_id, value));
                }
                _ => view_stack_extension.push(new_attribute_value_id),
            }
        }
        Ok((work_queue_extension, view_stack_extension))
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
        let prototype_id = AttributeValue::component_prototype_id(ctx, attribute_value_id)
            .await?
            .ok_or(AttributeValueError::NoComponentPrototype(
                attribute_value_id,
            ))?;

        ctx.workspace_snapshot()?
            .remove_edge_for_ulids(
                attribute_value_id,
                prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?;

        if !AttributeValue::is_set_by_dependent_function(ctx, attribute_value_id).await? {
            AttributeValue::update_from_prototype_function(ctx, attribute_value_id).await?;
        }
        ctx.add_dependent_values_and_enqueue(vec![attribute_value_id])
            .await?;

        Ok(())
    }

    #[instrument(name = "attribute_value.set_value", level = "debug", skip_all)]
    pub async fn set_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) -> AttributeValueResult<()> {
        let mut normalized_value = value.to_owned();
        let prop_id = match AttributeValue::is_for(ctx, attribute_value_id).await? {
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
            let prop_node = ctx
                .workspace_snapshot()?
                .get_node_weight_by_id(prop_id)
                .await?
                .get_prop_node_weight()?;

            if let Some(inner_value) = &value {
                // Unfortunately, there isn't a good way to consistently track "there is no value", and "the
                // value is null" when dealing with JavaScript/JSON, so for now, we need to treat
                // "null" the same as "there is no value".
                if inner_value.is_null() {
                    normalized_value = None;

                    IntrinsicFunc::Unset
                } else {
                    IntrinsicFunc::from(prop_node.kind())
                }
            } else {
                // None for the value means there is no value, so we use unset, but if it's a
                // literal serde_json::Value::Null it means the value is set, but to null
                IntrinsicFunc::Unset
            }
        };
        let func_id = Func::find_intrinsic(ctx, intrinsic_func).await?;
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        let prototype = AttributePrototype::new(ctx, func_id).await?;

        Self::set_component_prototype_id(ctx, attribute_value_id, prototype.id(), None).await?;

        let func_args = match normalized_value {
            Some(value) => {
                let func_arg_id = *FuncArgument::list_ids_for_func(ctx, func_id)
                    .await?
                    .first()
                    .ok_or(FuncArgumentError::IntrinsicMissingFuncArgumentEdge(
                        intrinsic_func.name().into(),
                        func_id,
                    ))?;

                let func_arg_name = {
                    ctx.workspace_snapshot()?
                        .get_node_weight_by_id(func_arg_id)
                        .await?
                        .get_func_argument_node_weight()?
                        .name()
                        .to_owned()
                };

                AttributePrototypeArgument::new(ctx, prototype.id(), func_arg_id)
                    .await?
                    .set_value_from_static_value(ctx, value.to_owned())
                    .await?;

                serde_json::json!({ func_arg_name: value } )
            }
            None => serde_json::Value::Null,
        };

        let result_channel =
            FuncRunner::run_attribute_value(ctx, attribute_value_id, func_id, func_args)
                .await
                .map_err(Box::new)?;
        let func_values = result_channel
            .await
            .map_err(|_| AttributeValueError::FuncRunnerSend)?
            .map_err(Box::new)?;

        Self::set_real_values(ctx, attribute_value_id, func_values, func).await?;
        Ok(())
    }

    async fn set_real_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        func_run_value: FuncRunValue,
        func: Func,
    ) -> AttributeValueResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let av_node_weight = workspace_snapshot
            .get_node_weight_by_id(attribute_value_id)
            .await?
            .get_attribute_value_node_weight()?;

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
            ctx.layer_db()
                .func_run()
                .set_values_and_set_state_to_success(
                    func_run_value.func_run_id(),
                    unprocessed_value_address,
                    value_address,
                    ctx.events_tenancy(),
                    ctx.events_actor(),
                )
                .await?;
        }

        let mut new_av_node_weight = av_node_weight.clone();

        new_av_node_weight.set_value(value_address.map(ContentAddress::JsonValue));
        new_av_node_weight
            .set_unprocessed_value(unprocessed_value_address.map(ContentAddress::JsonValue));

        workspace_snapshot
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
            let component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
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
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut dest_node_weight = workspace_snapshot
            .get_node_weight_by_id(dest_av_id)
            .await?
            .get_attribute_value_node_weight()?;
        let from_node_weight = workspace_snapshot
            .get_node_weight_by_id(from_av_id)
            .await?
            .get_attribute_value_node_weight()?;
        dest_node_weight.set_unprocessed_value(from_node_weight.unprocessed_value());
        dest_node_weight.set_value(from_node_weight.value());
        workspace_snapshot
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
        if let Some(from_prototype_id) =
            AttributeValue::component_prototype_id(ctx, from_av_id).await?
        {
            let from_func_id = AttributePrototype::func_id(ctx, from_prototype_id).await?;
            let dest_prototype = AttributePrototype::new(ctx, from_func_id).await?;

            for from_apa_id in
                AttributePrototypeArgument::list_ids_for_prototype(ctx, from_prototype_id).await?
            {
                let from_func_arg_id =
                    AttributePrototypeArgument::func_argument_id_by_id(ctx, from_apa_id).await?;
                let from_value_source =
                    AttributePrototypeArgument::value_source_by_id(ctx, from_apa_id)
                        .await?
                        .ok_or(
                            AttributeValueError::MissingAttributePrototypeArgumentSource(
                                from_apa_id,
                            ),
                        )?;

                let dest_apa =
                    AttributePrototypeArgument::new(ctx, dest_prototype.id(), from_func_arg_id)
                        .await?;
                AttributePrototypeArgument::set_value_source(ctx, dest_apa.id(), from_value_source)
                    .await?;
            }

            AttributeValue::set_component_prototype_id(ctx, dest_av_id, dest_prototype.id, None)
                .await?;

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
            AttributeValue::component_prototype_id(ctx, dest_av_id).await?
        {
            AttributePrototype::remove(ctx, existing_prototype_id).await?;
        }

        if !Self::is_set_by_dependent_function(ctx, dest_av_id).await? {
            Self::clone_node_weight_values_from(ctx, dest_av_id, from_av_id).await?;
        }

        Ok(())
    }

    pub async fn get_by_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let node_idx = workspace_snapshot
            .get_node_index_by_id(attribute_value_id)
            .await?;
        let node_weight = workspace_snapshot
            .get_node_weight(node_idx)
            .await?
            .get_attribute_value_node_weight()?;

        Ok(node_weight.into())
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
    ) -> AttributeValueResult<Prop> {
        let prop_id = Self::prop_id(ctx, attribute_value_id).await?;
        Ok(Prop::get_by_id(ctx, prop_id).await?)
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
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut maybe_prop_id = None;
        for target in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Prop,
            )
            .await?
        {
            let target_node_weight = workspace_snapshot.get_node_weight(target).await?;
            if let NodeWeight::Prop(prop_node_weight) = &target_node_weight {
                if let Some(already_found_prop_id) = maybe_prop_id {
                    return Err(AttributeValueError::MultiplePropsFound(
                        prop_node_weight.id().into(),
                        already_found_prop_id,
                        attribute_value_id,
                    ));
                }

                maybe_prop_id = Some(target_node_weight.id().into());
            }
        }

        Ok(maybe_prop_id)
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

    pub async fn value_or_default(
        &self,
        ctx: &DalContext,
        prop_id: PropId,
    ) -> AttributeValueResult<serde_json::Value> {
        match Self::fetch_value_from_store(ctx, self.value).await {
            Ok(Some(value)) => Ok(value),
            _ => Ok(Prop::default_value(ctx, prop_id)
                .await?
                .unwrap_or(Value::Null)),
        }
    }

    pub async fn unprocessed_value(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        Self::fetch_value_from_store(ctx, self.unprocessed_value).await
    }

    pub async fn get_parent_av_id_for_ordered_child(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributeValueId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let ordering_node_id = match workspace_snapshot
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Ordinal)
            .await?
            .first()
            .copied()
        {
            Some(ordering_idx) => workspace_snapshot.get_node_weight(ordering_idx).await?.id(),
            None => return Ok(None),
        };

        let parent_av_id = if let Some(parent_av_idx) = workspace_snapshot
            .incoming_sources_for_edge_weight_kind(
                ordering_node_id,
                EdgeWeightKindDiscriminants::Ordering,
            )
            .await?
            .first()
            .copied()
        {
            let parent_av_id: AttributeValueId = workspace_snapshot
                .get_node_weight(parent_av_idx)
                .await?
                .id()
                .into();

            let prop_id = AttributeValue::prop_id(ctx, parent_av_id).await?;

            let parent_prop = Prop::get_by_id(ctx, prop_id).await?;

            if ![PropKind::Map, PropKind::Array].contains(&parent_prop.kind) {
                return Ok(None);
            }

            parent_av_id
        } else {
            return Ok(None);
        };

        Ok(Some(parent_av_id))
    }

    async fn get_child_av_ids_from_ordering_node(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let ordered_ulids = ctx
            .workspace_snapshot()?
            .ordered_children_for_node(id)
            .await?
            .ok_or(AttributeValueError::NoOrderingNodeForAttributeValue(id))?;
        Ok(ordered_ulids.iter().map(|&id| id.into()).collect())
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
            PropKind::Array | PropKind::Map => {
                Self::get_child_av_ids_from_ordering_node(ctx, id).await
            }
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
                let child_ids = Self::get_child_av_ids_from_ordering_node(ctx, id).await?;
                let child_prop_ids = Prop::direct_child_prop_ids_ordered(ctx, prop.id).await?;

                // Get the mapping from PropId -> AttributeValueId
                let mut av_prop_map = HashMap::with_capacity(child_ids.len());
                for &child_id in &child_ids {
                    let child_prop_id = Self::prop_id(ctx, child_id).await?;
                    if av_prop_map.insert(child_prop_id, child_id).is_some() {
                        // If the prop showed up in more than one AV, something is wrong.
                        // Due to a bug, this sometimes happens in the wild; we're investigating.
                        let component =
                            Component::get_by_id(ctx, Self::component_id(ctx, child_id).await?)
                                .await?;
                        warn!(
                            "Multiple AVs for prop {} in component {}, schema {}",
                            Prop::path_by_id(ctx, child_prop_id).await?,
                            component.name(ctx).await?,
                            component.schema(ctx).await?.name
                        );
                        // TODO error instead when this bug is fixed
                        // return Err(AttributeValueError::MultipleAttributeValuesSameProp(
                        //     old_child_id,
                        //     child_id,
                        //     child_prop_id,
                        // ));
                    }
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
                            let child_prop_id = Self::prop_id(ctx, child_id).await?;
                            // If the prop wasn't a duplicate (caught earlier)
                            // TODO this appears when the above bug happens; reenable this
                            // when said bug is fixed
                            // return Err(AttributeValueError::FieldNotChildOfObject(child_id));
                            let component =
                                Component::get_by_id(ctx, Self::component_id(ctx, child_id).await?)
                                    .await?;
                            warn!(
                                    "Child AV with prop {} (parent ID {:?}) not found in corresponding parent prop {} (ID {}) in component {}, schema {}",
                                    Prop::path_by_id(ctx, child_prop_id).await?,
                                    Prop::parent_prop_id_by_id(ctx, child_prop_id).await?,
                                    Prop::path_by_id(ctx, prop.id).await?,
                                    prop.id,
                                    component.name(ctx).await?,
                                    component.schema(ctx).await?.name
                                );
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
        // The resulting pairs
        let mut pairs: Vec<ChildAttributeValuePair> = Vec::new();

        // The index in `pairs` for a given key
        let mut pair_index = HashMap::<KeyOrIndex, usize>::new();

        // Add the children of the first parent first, in order.
        let first_children = Self::get_child_av_ids_in_order(ctx, first_parent).await?;
        pairs.reserve(first_children.len());

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

    pub async fn remove_by_id(ctx: &DalContext, id: AttributeValueId) -> AttributeValueResult<()> {
        let parent_av_id = Self::get_parent_av_id_for_ordered_child(ctx, id)
            .await?
            .ok_or(AttributeValueError::RemovingWhenNotChildOrMapOrArray(id))?;

        ctx.workspace_snapshot()?.remove_node_by_id(id).await?;
        ctx.add_dependent_values_and_enqueue(vec![parent_av_id])
            .await?;
        Ok(())
    }

    pub async fn list_input_socket_sources_for_id(
        ctx: &DalContext,
        av_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<InputSocketId>> {
        let prototype_id = Self::prototype_id(ctx, av_id).await?;
        Ok(AttributePrototype::list_input_socket_sources_for_id(ctx, prototype_id).await?)
    }

    /// Get the moral equivalent of the [`PropPath`]for a given [`AttributeValueId`].
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
                        Self::parent_attribute_value_id(ctx, attribute_value_id).await?
                    {
                        let key_or_index =
                            Self::get_index_or_key_of_child_entry(ctx, attribute_value_id).await?;

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

    /// If the child attribute value is the child of a map, return its map key. Otherwise return None
    pub async fn get_key_of_child_entry(
        ctx: &DalContext,
        parent_attribute_value_id: AttributeValueId,
        child_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<String>> {
        Ok(ctx
            .workspace_snapshot()?
            .find_edge(
                parent_attribute_value_id,
                child_attribute_value_id,
                EdgeWeightKindDiscriminants::Contain,
            )
            .await
            .and_then(|weight| match weight.kind() {
                EdgeWeightKind::Contain(key) => key.to_owned(),
                _ => None,
            }))
    }

    pub async fn get_index_or_key_of_child_entry(
        ctx: &DalContext,
        child_id: AttributeValueId,
    ) -> AttributeValueResult<Option<KeyOrIndex>> {
        Ok(
            match Self::parent_attribute_value_id(ctx, child_id).await? {
                Some(pav_id) => match Self::is_for(ctx, pav_id).await? {
                    ValueIsFor::Prop(prop_id) => match Prop::get_by_id(ctx, prop_id).await?.kind {
                        PropKind::Array => {
                            match ctx
                                .workspace_snapshot()?
                                .ordering_node_for_container(pav_id)
                                .await?
                            {
                                Some(ordering_node) => {
                                    let index = ordering_node.get_index_for_id(child_id.into())?;
                                    Some(KeyOrIndex::Index(index))
                                }
                                None => None,
                            }
                        }
                        PropKind::Map => Self::get_key_of_child_entry(ctx, pav_id, child_id)
                            .await?
                            .map(KeyOrIndex::Key),
                        _ => None,
                    },
                    _ => None,
                },
                None => None,
            },
        )
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
            if let ValueIsFor::Prop(prop_id) =
                AttributeValue::is_for(ctx, attribute_value_id).await?
            {
                let prop = Prop::get_by_id(ctx, prop_id).await?;
                if prop.kind == PropKind::Object {
                    for child_value_id in
                        AttributeValue::get_child_av_ids_in_order(ctx, attribute_value_id).await?
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
