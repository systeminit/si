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

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
use ulid::Ulid;

pub use dependent_value_graph::DependentValueGraph;

use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::argument::{FuncArgument, FuncArgumentError};
use crate::func::before::before_funcs_for_component;
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::execution::{FuncExecution, FuncExecutionError, FuncExecutionPk};
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::FuncError;
use crate::prop::PropError;
use crate::socket::input::InputSocketError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{
    AttributeValueNodeWeight, NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::{serde_value_to_string_type, WorkspaceSnapshotError};
use crate::{
    pk, AttributePrototype, AttributePrototypeId, Component, ComponentId, DalContext, Func, FuncId,
    InputSocketId, OutputSocketId, Prop, PropId, PropKind, TransactionsError,
};

use super::prototype::argument::static_value::StaticArgumentValue;
use super::prototype::argument::value_source::ValueSourceError;
use super::prototype::argument::{
    value_source::ValueSource, AttributePrototypeArgument, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId,
};

pub mod dependent_value_graph;
pub mod view;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AttributeValueError {
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
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("component error: {0}")]
    Component(String),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("empty attribute prototype arguments for group name: {0}")]
    EmptyAttributePrototypeArgumentsForGroup(String),
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
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func execution error: {0}")]
    FuncExecution(#[from] FuncExecutionError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("cannot insert for prop kind: {0}")]
    InsertionForInvalidPropKind(PropKind),
    #[error("layer db error: {0}")]
    LayerDb(#[from] si_layer_cache::LayerDbError),
    #[error("missing attribute value with id: {0}")]
    MissingForId(AttributeValueId),
    #[error("attribute value {0} missing prop edge when one was expected")]
    MissingPropEdge(AttributeValueId),
    #[error("missing prototype for attribute value {0}")]
    MissingPrototype(AttributeValueId),
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
    #[error("value source error: {0}")]
    ValueSource(#[from] ValueSourceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributeValueResult<T> = Result<T, AttributeValueError>;

pk!(AttributeValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    id: AttributeValueId,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    value: Option<ContentAddress>,
    materialized_view: Option<ContentAddress>,
    func_execution_pk: Option<FuncExecutionPk>,
}

/// What "thing" on the schema variant, (either a prop, input socket, or output socket),
/// is a particular value the value of/for?
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ValueIsFor {
    Prop(PropId),
    InputSocket(InputSocketId),
    OutputSocket(OutputSocketId),
}

impl ValueIsFor {
    pub fn prop_id(&self) -> Option<PropId> {
        match self {
            ValueIsFor::Prop(prop_id) => Some(*prop_id),
            _ => None,
        }
    }

    pub fn output_socket_id(&self) -> Option<OutputSocketId> {
        match self {
            ValueIsFor::OutputSocket(id) => Some(*id),
            _ => None,
        }
    }

    pub fn input_socket_id(&self) -> Option<InputSocketId> {
        match self {
            ValueIsFor::InputSocket(id) => Some(*id),
            _ => None,
        }
    }
}

impl From<ValueIsFor> for Ulid {
    fn from(value: ValueIsFor) -> Self {
        match value {
            ValueIsFor::OutputSocket(output_socket_id) => output_socket_id.into(),
            ValueIsFor::InputSocket(input_socket_id) => input_socket_id.into(),
            ValueIsFor::Prop(prop_id) => prop_id.into(),
        }
    }
}

impl From<PropId> for ValueIsFor {
    fn from(value: PropId) -> Self {
        Self::Prop(value)
    }
}

impl From<OutputSocketId> for ValueIsFor {
    fn from(value: OutputSocketId) -> Self {
        Self::OutputSocket(value)
    }
}

impl From<InputSocketId> for ValueIsFor {
    fn from(value: InputSocketId) -> Self {
        Self::InputSocket(value)
    }
}

#[derive(Clone, Debug)]
pub struct PrototypeExecutionResult {
    value: Option<Value>,
    unprocessed_value: Option<Value>,
    func_execution_pk: FuncExecutionPk,
}

impl From<AttributeValueNodeWeight> for AttributeValue {
    fn from(value: AttributeValueNodeWeight) -> Self {
        Self {
            id: value.id().into(),
            unprocessed_value: value.unprocessed_value(),
            value: value.value(),
            materialized_view: value.materialized_view(),
            func_execution_pk: value.func_execution_pk(),
        }
    }
}

impl AttributeValue {
    pub fn id(&self) -> AttributeValueId {
        self.id
    }

    pub async fn new(
        ctx: &DalContext,
        is_for: impl Into<ValueIsFor>,
        component_id: Option<ComponentId>,
        maybe_parent_attribute_value: Option<AttributeValueId>,
        key: Option<String>,
    ) -> AttributeValueResult<Self> {
        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight = NodeWeight::new_attribute_value(change_set, id, None, None, None, None)?;
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
                .add_ordered_node(change_set, node_weight.clone())
                .await?;
        } else {
            ctx.workspace_snapshot()?
                .add_node(node_weight.clone())
                .await?;
        };

        match is_for {
            ValueIsFor::Prop(prop_id) => {
                ctx.workspace_snapshot()?
                    .add_edge(
                        id,
                        EdgeWeight::new(change_set, EdgeWeightKind::Prop)?,
                        prop_id,
                    )
                    .await?;

                // Attach value to parent prop (or root to component)
                match maybe_parent_attribute_value {
                    Some(pav_id) => {
                        ctx.workspace_snapshot()?
                            .add_ordered_edge(
                                change_set,
                                pav_id,
                                EdgeWeight::new(change_set, EdgeWeightKind::Contain(key))?,
                                id,
                            )
                            .await?;
                    }
                    None => {
                        // Component --Use--> AttributeValue
                        ctx.workspace_snapshot()?.add_edge(
                            component_id.ok_or(
                                AttributeValueError::CannotCreateRootPropValueWithoutComponentId,
                            )?,
                            EdgeWeight::new(change_set, EdgeWeightKind::Root)?,
                            id,
                        ).await?;
                    }
                }
            }
            is_for_socket => {
                // Attach value to component via SocketValue edge and to Socket
                let socket_id: Ulid = is_for_socket
                    .output_socket_id()
                    .map(Into::into)
                    .or_else(|| is_for_socket.input_socket_id().map(Into::into))
                    .ok_or(AttributeValueError::UnexpectedGraphLayout(
                        "we expected a ValueIsFor for a socket type here but did not get one",
                    ))?;

                ctx.workspace_snapshot()?
                    .add_edge(
                        component_id.ok_or(
                            AttributeValueError::CannotCreateSocketValueWithoutComponentId,
                        )?,
                        EdgeWeight::new(change_set, EdgeWeightKind::SocketValue)?,
                        id,
                    )
                    .await?;

                ctx.workspace_snapshot()?
                    .add_edge(
                        id,
                        EdgeWeight::new(change_set, EdgeWeightKind::Socket)?,
                        socket_id,
                    )
                    .await?;
            }
        }

        Ok(node_weight.get_attribute_value_node_weight()?.into())
    }

    async fn update_inner(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
        spawn_dependent_values_update: bool,
    ) -> AttributeValueResult<()> {
        Self::vivify_value_and_parent_values(ctx, attribute_value_id).await?;
        Self::set_value(ctx, attribute_value_id, value.clone()).await?;
        Self::populate_nested_values(ctx, attribute_value_id, value).await?;

        if spawn_dependent_values_update {
            ctx.enqueue_dependent_values_update(vec![attribute_value_id])
                .await?;
        }

        Ok(())
    }

    pub async fn update(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) -> AttributeValueResult<()> {
        Self::update_inner(ctx, attribute_value_id, value, true).await
    }

    /// Directly update an attribute value but do not trigger a dependent values update. Used
    /// during component creation so that we can ensure only one job is necessary for the many
    /// values updated when a component is created. Use only when you understand why you don't want
    /// to trigger a job, because if you don't run a dependent values job update, the materialized
    /// views for the component will *not* be updated to reflect the new value, nor will any values
    /// that depend on this value be updated.
    pub async fn update_no_dependent_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) -> AttributeValueResult<()> {
        Self::update_inner(ctx, attribute_value_id, value, false).await
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

        if let Some(input_socket) = socket_node_weight
            .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::InputSocket)
        {
            return Ok(ValueIsFor::InputSocket(input_socket.id().into()));
        }

        if let Some(output_socket) = socket_node_weight
            .get_option_content_node_weight_of_kind(ContentAddressDiscriminants::OutputSocket)
        {
            return Ok(ValueIsFor::OutputSocket(output_socket.id().into()));
        }

        Err(WorkspaceSnapshotError::UnexpectedEdgeTarget(
            socket_node_weight.id(),
            value_id.into(),
            EdgeWeightKindDiscriminants::Socket,
        )
        .into())
    }

    pub async fn execute_prototype_function(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<PrototypeExecutionResult> {
        let prototype_id = AttributeValue::prototype_id(ctx, attribute_value_id).await?;
        let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;
        let destination_component_id =
            AttributeValue::component_id(ctx, attribute_value_id).await?;
        let value_is_for = AttributeValue::is_for(ctx, attribute_value_id).await?;
        let apa_ids = AttributePrototypeArgument::list_ids_for_prototype(ctx, prototype_id).await?;
        let mut func_binding_args: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

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
                let destination_component = Component::get_by_id(ctx, destination_component_id)
                    .await
                    .map_err(|e| AttributeValueError::Component(e.to_string()))?;
                let source_component = Component::get_by_id(ctx, expected_source_component_id)
                    .await
                    .map_err(|e| AttributeValueError::Component(e.to_string()))?;
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
                        other_source => {
                            let mut values = vec![];

                            for av_id in other_source
                                .attribute_values_for_component_id(
                                    ctx,
                                    expected_source_component_id,
                                )
                                .await?
                            {
                                let attribute_value = AttributeValue::get_by_id(ctx, av_id).await?;
                                // XXX: We need to properly handle the difference between "there is
                                // XXX: no value" vs "the value is null", but right now we collapse
                                // XXX: the two to just be "null" when passing these to a function.
                                values.push(
                                    attribute_value
                                        .materialized_view(ctx)
                                        .await?
                                        .unwrap_or(Value::Null),
                                );
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
                    )
                }
            }
        }
        let prepared_func_binding_args = serde_json::to_value(args_map)?;

        // We need the associated [`ComponentId`] for this function--this is how we resolve and
        // prepare before functions
        let associated_component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
        let before = before_funcs_for_component(ctx, &associated_component_id)
            .await
            .map_err(|e| AttributeValueError::BeforeFunc(e.to_string()))?;

        let (_, func_binding_return_value) = match FuncBinding::create_and_execute(
            ctx,
            prepared_func_binding_args.clone(),
            prototype_func_id,
            before,
        )
        .instrument(debug_span!(
            "Func execution",
            "func.id" = %prototype_func_id,
            ?prepared_func_binding_args,
        ))
        .await
        {
            Ok(function_return_value) => function_return_value,
            Err(FuncBindingError::FuncBackendResultFailure {
                kind,
                message,
                backend,
            }) => {
                return Err(AttributeValueError::FuncBackendResultFailure {
                    kind,
                    message,
                    backend,
                });
            }
            Err(err) => Err(err)?,
        };

        let unprocessed_value = func_binding_return_value.unprocessed_value().cloned();
        let processed_value = match value_is_for {
            ValueIsFor::Prop(prop_id) => match &unprocessed_value {
                Some(unprocessed_value) => {
                    let prop = Prop::get_by_id(ctx, prop_id).await?;
                    match prop.kind {
                        PropKind::Object | PropKind::Map => Some(serde_json::json!({})),
                        PropKind::Array => Some(serde_json::json!([])),
                        _ => Some(unprocessed_value.to_owned()),
                    }
                }
                None => None,
            },
            _ => func_binding_return_value.value().cloned(),
        };

        Ok(PrototypeExecutionResult {
            value: processed_value,
            unprocessed_value,
            func_execution_pk: func_binding_return_value.func_execution_pk(),
        })
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn set_values_from_execution_result(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        PrototypeExecutionResult {
            value,
            unprocessed_value,
            func_execution_pk,
        }: PrototypeExecutionResult,
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

        let values_are_different = value != unprocessed_value;

        Self::set_real_values(
            ctx,
            attribute_value_id,
            value,
            unprocessed_value.clone(),
            func_execution_pk,
        )
        .await?;

        if values_are_different {
            Self::populate_nested_values(ctx, attribute_value_id, unprocessed_value).await?;
        } else {
            let materialized_view =
                AttributeValue::create_materialized_view(ctx, attribute_value_id).await?;
            Self::set_materialized_view(ctx, attribute_value_id, materialized_view).await?;
        }

        Ok(())
    }

    pub async fn update_from_prototype_function(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let execution_result =
            AttributeValue::execute_prototype_function(ctx, attribute_value_id).await?;

        AttributeValue::set_values_from_execution_result(ctx, attribute_value_id, execution_result)
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

    pub async fn insert(
        ctx: &DalContext,
        parent_attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        key: Option<String>,
    ) -> AttributeValueResult<AttributeValueId> {
        let element_prop_id: PropId = {
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
            if prop_node_weight.kind() != PropKind::Array
                && prop_node_weight.kind() != PropKind::Map
            {
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

            workspace_snapshot
                .get_node_weight(element_prop_index)
                .await?
                .get_prop_node_weight()?
                .clone()
                .id()
                .into()
        };

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
            let empty_value = {
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

                prop_node.kind().empty_value()
            };

            let attribute_value = Self::get_by_id(ctx, attribute_value_id).await?;

            // If we have a set value, we don't need to vivify
            if attribute_value.value.is_some() {
                return Ok(());
            } else {
                Self::set_value(ctx, attribute_value_id, empty_value).await?;

                current_attribute_value_id =
                    AttributeValue::parent_attribute_value_id(ctx, attribute_value_id).await?;
            }
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
            let current_node_index = workspace_snapshot
                .get_node_index_by_id(attribute_value_id)
                .await?;
            let current_target_idx = workspace_snapshot
                .get_latest_node_index(attribute_value_target)
                .await?;

            workspace_snapshot
                .remove_edge(
                    ctx.change_set_pointer()?,
                    current_node_index,
                    current_target_idx,
                    EdgeWeightKindDiscriminants::Contain,
                )
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

        // walk up the tree that we touched, creating materialized views
        while let Some(attribute_value_id) = view_stack.pop() {
            let materialized_view =
                AttributeValue::create_materialized_view(ctx, attribute_value_id).await?;
            Self::set_materialized_view(ctx, attribute_value_id, materialized_view).await?;
        }

        Ok(())
    }

    pub async fn create_materialized_view(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        let av = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        if av.value(ctx).await?.is_none() {
            return Ok(None);
        }
        let workspace_snapshot = ctx.workspace_snapshot()?;

        match AttributeValue::is_for(ctx, attribute_value_id).await? {
            ValueIsFor::Prop(prop_id) => {
                let prop_kind = workspace_snapshot
                    .get_node_weight_by_id(prop_id)
                    .await?
                    .get_prop_node_weight()?
                    .kind();

                match prop_kind {
                    PropKind::Object => {
                        let mut object_view: HashMap<String, serde_json::Value> = HashMap::new();
                        let mut child_av_ids = vec![];

                        {
                            for child_target in workspace_snapshot
                                .outgoing_targets_for_edge_weight_kind(
                                    attribute_value_id,
                                    EdgeWeightKindDiscriminants::Contain,
                                )
                                .await?
                            {
                                let av_id =
                                    workspace_snapshot.get_node_weight(child_target).await?.id();
                                child_av_ids.push(av_id.into());
                            }
                        }

                        for child_av_id in child_av_ids {
                            let child_av = AttributeValue::get_by_id(ctx, child_av_id).await?;

                            if let ValueIsFor::Prop(child_prop_id) =
                                AttributeValue::is_for(ctx, child_av.id()).await?
                            {
                                let child_prop_name = {
                                    workspace_snapshot
                                        .get_node_weight_by_id(child_prop_id)
                                        .await?
                                        .get_prop_node_weight()?
                                        .name()
                                        .to_owned()
                                };

                                let child_materialized_view =
                                    child_av.materialized_view(ctx).await?;
                                if let Some(view) = child_materialized_view {
                                    object_view.insert(child_prop_name, view);
                                }
                            } else {
                                return Err(AttributeValueError::UnexpectedGraphLayout("a child attribute value of an object has no outgoing Prop edge but has an outgoing Socket edge"));
                            }
                        }

                        Ok(Some(serde_json::to_value(object_view)?))
                    }
                    PropKind::Map => {
                        let mut map_view: HashMap<String, serde_json::Value> = HashMap::new();

                        let child_av_idxs_and_keys: HashMap<String, NodeIndex> = {
                            workspace_snapshot
                                .edges_directed_for_edge_weight_kind(
                                    attribute_value_id,
                                    Outgoing,
                                    EdgeWeightKindDiscriminants::Contain,
                                )
                                .await?
                                .iter()
                                .filter_map(|(edge_weight, _, target_idx)| {
                                    if let EdgeWeightKind::Contain(Some(key)) = edge_weight.kind() {
                                        Some((key.to_owned(), *target_idx))
                                    } else {
                                        None
                                    }
                                })
                                .collect()
                        };

                        for (key, node_index) in child_av_idxs_and_keys {
                            let child_av_id: AttributeValueId = {
                                workspace_snapshot
                                    .get_node_weight(node_index)
                                    .await?
                                    .id()
                                    .into()
                            };

                            let child_av = AttributeValue::get_by_id(ctx, child_av_id).await?;
                            if let Some(view) = child_av.materialized_view(ctx).await? {
                                map_view.insert(key, view);
                            }
                        }

                        Ok(Some(serde_json::to_value(map_view)?))
                    }
                    PropKind::Array => {
                        let mut array_view = vec![];

                        let element_av_ids = {
                            workspace_snapshot
                                .ordered_children_for_node(attribute_value_id)
                                .await?
                                .ok_or(AttributeValueError::UnexpectedGraphLayout(
                                    "array attribute value has no ordering node",
                                ))?
                        };

                        for element_av_id in element_av_ids {
                            let av = AttributeValue::get_by_id(ctx, element_av_id.into()).await?;
                            if let Some(view) = av.materialized_view(ctx).await? {
                                array_view.push(view);
                            }
                        }

                        Ok(Some(serde_json::to_value(array_view)?))
                    }
                    _ => Ok(av.value(ctx).await?),
                }
            }
            ValueIsFor::OutputSocket(_) | ValueIsFor::InputSocket(_) => Ok(av.value(ctx).await?),
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
                key = contain_key.to_owned();
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
            .edges_directed(attribute_value_id, Incoming)
            .await?
            .iter()
            .find(|(edge_weight, _, _)| {
                matches!(edge_weight.kind(), EdgeWeightKind::Contain(Some(_)))
            })
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
    pub async fn set_component_prototype_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        attribute_prototype_id: AttributePrototypeId,
    ) -> AttributeValueResult<()> {
        let maybe_existing_prototype_id =
            Self::component_prototype_id(ctx, attribute_value_id).await?;

        if let Some(exsiting_prototype_id) = maybe_existing_prototype_id {
            AttributePrototype::remove(ctx, exsiting_prototype_id).await?;
        }

        ctx.workspace_snapshot()?
            .add_edge(
                attribute_value_id,
                EdgeWeight::new(ctx.change_set_pointer()?, EdgeWeightKind::Prototype(None))?,
                attribute_prototype_id,
            )
            .await?;

        Ok(())
    }

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
                ctx.change_set_pointer()?,
                attribute_value_id,
                prototype_id,
                EdgeWeightKindDiscriminants::Prototype,
            )
            .await?;

        AttributeValue::update_from_prototype_function(ctx, attribute_value_id).await?;
        ctx.enqueue_dependent_values_update(vec![attribute_value_id])
            .await?;

        Ok(())
    }
    async fn set_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) -> AttributeValueResult<()> {
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

            // None for the value means there is no value, so we use unset, but if it's a
            // literal serde_json::Value::Null it means the value is set, but to null
            if value.is_none() {
                IntrinsicFunc::Unset
            } else {
                IntrinsicFunc::from(prop_node.kind())
            }
        };

        let func_id = Func::find_intrinsic(ctx, intrinsic_func).await?;
        let prototype = AttributePrototype::new(ctx, func_id).await?;

        Self::set_component_prototype_id(ctx, attribute_value_id, prototype.id()).await?;

        let func_binding_args = match value.to_owned() {
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

        let associated_component_id = AttributeValue::component_id(ctx, attribute_value_id).await?;
        let before = before_funcs_for_component(ctx, &associated_component_id)
            .await
            .map_err(|e| AttributeValueError::BeforeFunc(e.to_string()))?;

        let (_, func_binding_return_value) =
            match FuncBinding::create_and_execute(ctx, func_binding_args.clone(), func_id, before)
                .instrument(debug_span!(
                    "Func execution",
                    "func.id" = %func_id,
                    ?func_binding_args,
                ))
                .await
            {
                Ok(function_return_value) => function_return_value,
                Err(FuncBindingError::FuncBackendResultFailure {
                    kind,
                    message,
                    backend,
                }) => {
                    return Err(AttributeValueError::FuncBackendResultFailure {
                        kind,
                        message,
                        backend,
                    });
                }
                Err(err) => Err(err)?,
            };

        Self::set_real_values(
            ctx,
            attribute_value_id,
            func_binding_return_value.value().cloned(),
            func_binding_return_value.unprocessed_value().cloned(),
            func_binding_return_value.func_execution_pk(),
        )
        .await?;
        Ok(())
    }

    #[instrument(level = "debug", skip_all)]
    async fn set_materialized_view(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        view: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let (av_idx, av_node_weight) = {
            let av_idx = workspace_snapshot
                .get_node_index_by_id(attribute_value_id)
                .await?;

            (
                av_idx,
                workspace_snapshot
                    .get_node_weight(av_idx)
                    .await?
                    .get_attribute_value_node_weight()?,
            )
        };

        let content_view: Option<si_events::CasValue> = view.clone().map(Into::into);

        let view_address = match content_view {
            Some(view) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(view.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )
                    .await?
                    .0,
            ),
            None => None,
        };

        debug!(
            "set_materialized_view: {:?}, {:?}, {}",
            &view, &view_address, attribute_value_id
        );

        let mut new_av_node_weight =
            av_node_weight.new_with_incremented_vector_clock(ctx.change_set_pointer()?)?;

        new_av_node_weight.set_materialized_view(view_address.map(ContentAddress::JsonValue));

        workspace_snapshot
            .add_node(NodeWeight::AttributeValue(new_av_node_weight))
            .await?;
        workspace_snapshot.replace_references(av_idx).await?;

        Ok(())
    }

    // todo: add func binding id and func binding return value id here to store on the attribute
    // value, this will also mean creating those rows for "intrinsic" execution in set_value
    async fn set_real_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        unprocessed_value: Option<serde_json::Value>,
        func_execution_pk: FuncExecutionPk,
    ) -> AttributeValueResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let (av_idx, av_node_weight) = {
            let av_idx = workspace_snapshot
                .get_node_index_by_id(attribute_value_id)
                .await?;

            (
                av_idx,
                workspace_snapshot
                    .get_node_weight(av_idx)
                    .await?
                    .get_attribute_value_node_weight()?,
            )
        };

        let content_value: Option<si_events::CasValue> = value.map(Into::into);
        let content_unprocessed_value: Option<si_events::CasValue> =
            unprocessed_value.map(Into::into);

        let value_address = match content_value {
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

        let unprocessed_value_address = match content_unprocessed_value {
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

        let mut new_av_node_weight =
            av_node_weight.new_with_incremented_vector_clock(ctx.change_set_pointer()?)?;

        new_av_node_weight.set_value(value_address.map(ContentAddress::JsonValue));
        new_av_node_weight
            .set_unprocessed_value(unprocessed_value_address.map(ContentAddress::JsonValue));
        new_av_node_weight.set_func_execution_pk(Some(func_execution_pk));

        workspace_snapshot
            .add_node(NodeWeight::AttributeValue(new_av_node_weight))
            .await?;
        workspace_snapshot.replace_references(av_idx).await?;

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

    pub async fn prop_id_for_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<PropId> {
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
                maybe_prop_id = match maybe_prop_id {
                    Some(already_found_prop_id) => {
                        return Err(AttributeValueError::MultiplePropsFound(
                            prop_node_weight.id().into(),
                            already_found_prop_id,
                            attribute_value_id,
                        ));
                    }
                    None => Some(target_node_weight.id().into()),
                };
            }
        }

        maybe_prop_id.ok_or(AttributeValueError::PropNotFound(attribute_value_id))
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

    pub async fn unprocessed_value(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        Self::fetch_value_from_store(ctx, self.unprocessed_value).await
    }

    pub async fn materialized_view(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        Self::fetch_value_from_store(ctx, self.materialized_view).await
    }

    pub async fn func_execution(
        &self,
        ctx: &DalContext,
    ) -> AttributeValueResult<Option<FuncExecution>> {
        Ok(match self.func_execution_pk {
            Some(pk) => Some(FuncExecution::get_by_pk(ctx, &pk).await?),
            None => None,
        })
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

            let prop_id = AttributeValue::prop_id_for_id(ctx, parent_av_id).await?;

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

    pub async fn get_child_av_ids_for_ordered_parent(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        if let Some(ordering) = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Ordering)
            .await?
            .pop()
        {
            let node_weight = workspace_snapshot.get_node_weight(ordering).await?;
            if let NodeWeight::Ordering(ordering_weight) = node_weight {
                Ok(ordering_weight
                    .order()
                    .clone()
                    .into_iter()
                    .map(|ulid| ulid.into())
                    .collect())
            } else {
                Err(AttributeValueError::NodeWeightMismatch(
                    ordering,
                    NodeWeightDiscriminants::Ordering,
                ))
            }
        } else {
            // Leaves don't have ordering nodes
            Ok(vec![])
        }
    }

    pub async fn remove_by_id(ctx: &DalContext, id: AttributeValueId) -> AttributeValueResult<()> {
        let parent_av_id = Self::get_parent_av_id_for_ordered_child(ctx, id)
            .await?
            .ok_or(AttributeValueError::RemovingWhenNotChildOrMapOrArray(id))?;

        let av = Self::get_by_id(ctx, id).await?;

        ctx.workspace_snapshot()?
            .remove_node_by_id(ctx.change_set_pointer()?, av.id)
            .await?;

        ctx.enqueue_dependent_values_update(vec![parent_av_id])
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
}
