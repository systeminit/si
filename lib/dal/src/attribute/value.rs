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

use content_store::{ContentHash, Store, StoreError};
use petgraph::graph::NodeIndex;
use petgraph::prelude::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use strum::EnumDiscriminants;
use telemetry::prelude::*;
use thiserror::Error;
use tokio::sync::TryLockError;
use ulid::Ulid;

use crate::attribute::prototype::AttributePrototypeError;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::intrinsics::IntrinsicFunc;
use crate::func::FuncError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{
    NodeWeight, NodeWeightDiscriminants, NodeWeightError,
};
use crate::workspace_snapshot::{serde_value_to_string_type, WorkspaceSnapshotError};
use crate::{
    pk, AttributePrototype, AttributePrototypeId, DalContext, Func, FuncId, PropId, PropKind,
    Timestamp, TransactionsError,
};

pub mod view;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum AttributeValueError {
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetPointerError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("cannot insert for prop kind: {0}")]
    InsertionForInvalidPropKind(PropKind),
    #[error("attribute value {0} missing prop edge when one was expected")]
    MissingPropEdge(AttributeValueId),
    #[error("missing prototype for attribute value {0}")]
    MissingPrototype(AttributeValueId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight mismatch, expected {0:?} to be {1:?}")]
    NodeWeightMismatch(NodeIndex, NodeWeightDiscriminants),
    #[error("array or map prop missing element prop: {0}")]
    PropMissingElementProp(PropId),
    #[error("array or map prop has more than one child prop: {0}")]
    PropMoreThanOneChild(PropId),
    #[error("store error: {0}")]
    Store(#[from] StoreError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("try lock error: {0}")]
    TryLock(#[from] TryLockError),
    #[error("type mismatch: expected prop kind {0}, got {1}")]
    TypeMismatch(PropKind, String),
    #[error("unexpected graph layout: {0}")]
    UnexpectedGraphLayout(&'static str),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type AttributeValueResult<T> = Result<T, AttributeValueError>;

pk!(AttributeValueId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeValue {
    pub id: AttributeValueId,
    #[serde(flatten)]
    pub timestamp: Timestamp,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<serde_json::Value>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<serde_json::Value>,
}

#[derive(EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum AttributeValueContent {
    V1(AttributeValueContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributeValueContentV1 {
    pub timestamp: Timestamp,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    pub unprocessed_value: Option<serde_json::Value>,
    /// The processed return value.
    /// Example: empty array.
    pub value: Option<serde_json::Value>,
}

impl From<AttributeValue> for AttributeValueContentV1 {
    fn from(value: AttributeValue) -> Self {
        Self {
            timestamp: value.timestamp,
            unprocessed_value: value.unprocessed_value,
            value: value.value,
        }
    }
}

impl AttributeValue {
    pub fn assemble(id: AttributeValueId, inner: AttributeValueContentV1) -> Self {
        Self {
            id,
            timestamp: inner.timestamp,
            value: inner.value,
            unprocessed_value: inner.unprocessed_value,
        }
    }

    pub fn id(&self) -> AttributeValueId {
        self.id
    }

    pub fn new(ctx: &DalContext, ordered: bool) -> AttributeValueResult<Self> {
        let content = AttributeValueContentV1 {
            timestamp: Timestamp::now(),
            unprocessed_value: None,
            value: None,
        };
        let hash = ctx
            .content_store()
            .try_lock()?
            .add(&AttributeValueContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributeValue(hash))?;
        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            if ordered {
                workspace_snapshot.add_ordered_node(change_set, node_weight)?;
            } else {
                workspace_snapshot.add_node(node_weight)?;
            };
        }

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn update(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        Self::vivify_value_and_parent_values(ctx, attribute_value_id).await?;
        Self::set_value(ctx, attribute_value_id, value.clone()).await?;
        Self::populate_nested_values(ctx, attribute_value_id, value).await?;
        Ok(())
    }

    pub async fn insert(
        ctx: &DalContext,
        parent_attribute_value_id: AttributeValueId,
        key: Option<String>,
        value: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        let element_prop_node_weight = {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            // Find the array or map prop.
            let prop_index = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(
                    parent_attribute_value_id,
                    EdgeWeightKindDiscriminants::Prop,
                )?
                .get(0)
                .copied()
                .ok_or(AttributeValueError::MissingPropEdge(
                    parent_attribute_value_id,
                ))?;
            let prop_node_weight = match workspace_snapshot.get_node_weight(prop_index)?.clone() {
                NodeWeight::Prop(inner) => inner,
                _ => {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        prop_index,
                        NodeWeightDiscriminants::Prop,
                    ))
                }
            };

            // Ensure it actually is an array or map prop.
            if prop_node_weight.kind() != PropKind::Array
                || prop_node_weight.kind() != PropKind::Map
            {
                return Err(AttributeValueError::InsertionForInvalidPropKind(
                    prop_node_weight.kind(),
                ));
            }

            // Find a singular child prop for the map or an array prop (i.e. the "element" or "entry" prop").
            let prop_id = PropId::from(prop_node_weight.id());
            let child_prop_indices = workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                prop_node_weight.id(),
                EdgeWeightKindDiscriminants::Use,
            )?;
            if child_prop_indices.len() > 1 {
                return Err(AttributeValueError::PropMoreThanOneChild(prop_id));
            }
            let element_prop_index = child_prop_indices
                .get(0)
                .ok_or(AttributeValueError::PropMissingElementProp(prop_id))?
                .to_owned();
            match workspace_snapshot
                .get_node_weight(element_prop_index)?
                .clone()
            {
                NodeWeight::Prop(inner) => inner,
                _ => {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        element_prop_index,
                        NodeWeightDiscriminants::Prop,
                    ))
                }
            }
        };

        // Create the "element" attribute value in the array or map alongside an attribute prototype for it.
        let new_attribute_value = Self::new(
            ctx,
            matches!(
                element_prop_node_weight.kind(),
                PropKind::Map | PropKind::Object | PropKind::Array
            ),
        )?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            let change_set = ctx.change_set_pointer()?;
            workspace_snapshot.add_ordered_edge(
                change_set,
                parent_attribute_value_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(key))?,
                new_attribute_value.id,
            )?;

            workspace_snapshot.add_edge(
                new_attribute_value.id,
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)?,
                element_prop_node_weight.id(),
            )?;
        }

        let func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset)?;
        AttributePrototype::new(ctx, func_id)?;

        // The element has been created an inserted. Now, we can update it with the provided value.
        Self::update(ctx, new_attribute_value.id, value).await
    }

    async fn vivify_value_and_parent_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        // determine if the value is for a prop, or for an internal provider. if it is for an
        // internal provider we want to find if it is an internal provider for a prop (since we
        // want to use the function for that prop kind), or if it is an explicit internal  or
        // external provider (and has no prop)
        // Values on components have outgoing edges to props or outgoing edges to a provider. Values
        // on a schema variant have incoming edges from props or incoming edges from providers
        let mut current_attribute_value_id = Some(attribute_value_id);

        while let Some(attribute_value_id) = current_attribute_value_id {
            let mut maybe_prop_node_index = None;
            let mut maybe_provider_node_index = None;
            let empty_value = {
                let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                for edge_ref in
                    workspace_snapshot.edges_directed(attribute_value_id, Direction::Outgoing)?
                {
                    if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                        maybe_prop_node_index = Some(edge_ref.target());
                    }

                    if edge_ref.weight().kind() == &EdgeWeightKind::Provider {
                        maybe_provider_node_index = Some(edge_ref.target());
                    }
                }

                if maybe_provider_node_index.is_none() || maybe_prop_node_index.is_none() {
                    for edge_ref in workspace_snapshot
                        .edges_directed(attribute_value_id, Direction::Incoming)?
                    {
                        if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                            maybe_prop_node_index = Some(edge_ref.source());
                        }

                        if edge_ref.weight().kind() == &EdgeWeightKind::Provider {
                            maybe_provider_node_index = Some(edge_ref.source());
                        }
                    }
                }

                // This should not be possible.
                if maybe_prop_node_index.is_some() && maybe_provider_node_index.is_some() {
                    return Err(AttributeValueError::UnexpectedGraphLayout(
                        "found both an provider edge and an prop edge",
                    ));
                }

                // We're set on a provider, so we should look up the prop (if any)
                if let Some(provider_node_index) = maybe_provider_node_index {
                    let provider_id = workspace_snapshot
                        .get_node_weight(provider_node_index)?
                        .id();

                    maybe_prop_node_index = workspace_snapshot
                        .incoming_sources_for_edge_weight_kind(
                            provider_id,
                            EdgeWeightKindDiscriminants::Prop,
                        )?
                        .get(0)
                        .copied();
                }

                match maybe_prop_node_index {
                    Some(prop_node_index) => {
                        match workspace_snapshot.get_node_weight(prop_node_index).map(
                            |node_weight| {
                                if let NodeWeight::Prop(inner) = node_weight {
                                    Some(inner.kind())
                                } else {
                                    None
                                }
                            },
                        )? {
                            Some(PropKind::Array) => Some(serde_json::json!([])),
                            Some(PropKind::Map) | Some(PropKind::Object) => {
                                Some(serde_json::json!({}))
                            }

                            // This means we did not get a prop node weight despite the node index coming
                            // from a prop edge
                            None => {
                                return Err(AttributeValueError::NodeWeightMismatch(
                                    prop_node_index,
                                    NodeWeightDiscriminants::Prop,
                                ))
                            }
                            _ => None,
                        }
                    }
                    None => Some(serde_json::json!({})),
                }
            };

            let (_, inner) = Self::get_content(ctx, attribute_value_id).await?;

            // If we have a set value, we don't need to vivify
            if inner.value.is_some() {
                return Ok(());
            } else {
                Self::set_value(ctx, attribute_value_id, empty_value).await?;

                let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

                // This assumes the only incoming contain edge from an attribute value is from
                // another attribute value
                let maybe_parent_attribute_node_index = workspace_snapshot
                    .incoming_sources_for_edge_weight_kind(
                        attribute_value_id,
                        EdgeWeightKindDiscriminants::Contain,
                    )?
                    .get(0)
                    .copied();

                if let Some(node_index) = maybe_parent_attribute_node_index {
                    current_attribute_value_id = Some(AttributeValueId::from(
                        workspace_snapshot.get_node_weight(node_index)?.id(),
                    ));
                } else {
                    current_attribute_value_id = None;
                }
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
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            let prop_node_index = workspace_snapshot.get_node_index_by_id(prop_id)?;
            if let NodeWeight::Prop(prop_inner) =
                workspace_snapshot.get_node_weight(prop_node_index)?
            {
                prop_inner.kind()
            } else {
                return Err(AttributeValueError::NodeWeightMismatch(
                    prop_node_index,
                    NodeWeightDiscriminants::Prop,
                ));
            }
        };

        let new_attribute_value = Self::new(ctx, true)?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            let change_set = ctx.change_set_pointer()?;

            workspace_snapshot.add_ordered_edge(
                change_set,
                attribute_value_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Contain(key))?,
                new_attribute_value.id,
            )?;

            workspace_snapshot.add_edge(
                new_attribute_value.id,
                EdgeWeight::new(change_set, EdgeWeightKind::Prop)?,
                prop_id,
            )?;
        }

        AttributePrototype::new(ctx, func_id)?;

        match prop_kind {
            PropKind::Object | PropKind::Map => {
                Self::set_value(
                    ctx,
                    attribute_value_id,
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
                    attribute_value_id,
                    if value.is_some() {
                        Some(serde_json::json!([]))
                    } else {
                        None
                    },
                )
                .await?;
            }
            _ => {
                Self::set_value(ctx, attribute_value_id, value).await?;
            }
        }

        Ok(new_attribute_value.id)
    }

    async fn populate_nested_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        // Cache the unset func id before getting the workspace snapshot.
        let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset)?;

        {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            // Remove child attribute value edges
            for attribute_value_target in workspace_snapshot.outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::Contain,
            )? {
                let current_node_index =
                    workspace_snapshot.get_node_index_by_id(attribute_value_id)?;
                workspace_snapshot.remove_edge(
                    ctx.change_set_pointer()?,
                    current_node_index,
                    attribute_value_target,
                    EdgeWeightKindDiscriminants::Contain,
                )?;
            }
        }

        let mut work_queue = VecDeque::from([(attribute_value_id, value)]);

        while let Some((attribute_value_id, maybe_value)) = work_queue.pop_front() {
            let (prop_kind, prop_id) = {
                let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                // We're only looking for props on outgoing edges because we're assuming this will only be used for
                // attribute values on components. For default values at the schema variant level, we're
                // planning to add a "const arg" node that contains the default input for the function that
                // sets the value on the prototype
                let prop_node_index = workspace_snapshot
                    .outgoing_targets_for_edge_weight_kind(
                        attribute_value_id,
                        EdgeWeightKindDiscriminants::Prop,
                    )?
                    .get(0)
                    .copied()
                    .ok_or(AttributeValueError::MissingPropEdge(attribute_value_id))?;

                if let NodeWeight::Prop(prop_inner) =
                    workspace_snapshot.get_node_weight(prop_node_index)?
                {
                    (prop_inner.kind(), PropId::from(prop_inner.id()))
                } else {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        prop_node_index,
                        NodeWeightDiscriminants::Prop,
                    ));
                }
            };

            let extension = match prop_kind {
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
            work_queue.extend(extension);
        }

        Ok(())
    }

    async fn process_populate_nested_values_for_object(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_value_id: AttributeValueId,
        unset_func_id: FuncId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<VecDeque<(AttributeValueId, Option<Value>)>> {
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
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            let child_prop_indexes = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)?;

            let mut prop_map = HashMap::new();
            for node_index in child_prop_indexes {
                if let NodeWeight::Prop(prop_inner) =
                    workspace_snapshot.get_node_weight(node_index)?
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
                _ => {}
            }
        }
        Ok(work_queue_extension)
    }

    async fn process_populate_nested_values_for_array(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_value_id: AttributeValueId,
        unset_func_id: FuncId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<VecDeque<(AttributeValueId, Option<Value>)>> {
        let mut work_queue_extension = VecDeque::new();

        let array_items = match maybe_value {
            Some(serde_json::Value::Array(array)) => {
                if array.is_empty() {
                    return Ok(work_queue_extension);
                }
                array
            }
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Array,
                    serde_value_to_string_type(&value),
                ));
            }
            None => return Ok(work_queue_extension),
        };

        let (element_prop_id, element_prop_kind) = {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            // find the child element prop
            let child_props = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)?;

            if child_props.len() > 1 {
                return Err(AttributeValueError::PropMoreThanOneChild(prop_id));
            }

            let element_prop_index = child_props
                .get(0)
                .ok_or(AttributeValueError::PropMissingElementProp(prop_id))?
                .to_owned();

            match workspace_snapshot.get_node_weight(element_prop_index)? {
                NodeWeight::Prop(prop_inner) => (prop_inner.id(), prop_inner.kind()),
                _ => {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        element_prop_index,
                        NodeWeightDiscriminants::Prop,
                    ))
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
                _ => {}
            }
        }

        Ok(work_queue_extension)
    }

    async fn process_populate_nested_values_for_map(
        ctx: &DalContext,
        prop_id: PropId,
        attribute_value_id: AttributeValueId,
        unset_func_id: FuncId,
        maybe_value: Option<Value>,
    ) -> AttributeValueResult<VecDeque<(AttributeValueId, Option<Value>)>> {
        let mut work_queue_extension = VecDeque::new();

        let map_map = match maybe_value {
            Some(Value::Object(map)) => {
                if map.is_empty() {
                    return Ok(work_queue_extension);
                }
                map
            }
            Some(value) => {
                return Err(AttributeValueError::TypeMismatch(
                    PropKind::Map,
                    serde_value_to_string_type(&value),
                ));
            }
            None => return Ok(work_queue_extension),
        };

        let (element_prop_id, element_prop_kind) = {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            // find the child element prop
            let child_props = workspace_snapshot
                .outgoing_targets_for_edge_weight_kind(prop_id, EdgeWeightKindDiscriminants::Use)?;

            if child_props.len() > 1 {
                return Err(AttributeValueError::PropMoreThanOneChild(prop_id));
            }

            let element_prop_index = child_props
                .get(0)
                .ok_or(AttributeValueError::PropMissingElementProp(prop_id))?
                .to_owned();

            match workspace_snapshot.get_node_weight(element_prop_index)? {
                NodeWeight::Prop(prop_inner) => (prop_inner.id(), prop_inner.kind()),
                _ => {
                    return Err(AttributeValueError::NodeWeightMismatch(
                        element_prop_index,
                        NodeWeightDiscriminants::Prop,
                    ))
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
                Some(key),
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
                _ => {}
            }
        }
        Ok(work_queue_extension)
    }

    async fn set_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> AttributeValueResult<()> {
        let mut maybe_prop_node_index = None;
        let mut maybe_prototype_node_index = None;
        let mut prop_direction = Direction::Outgoing;

        let (intrinsic_func, attribute_prototype_id) = {
            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

            for edge_ref in
                workspace_snapshot.edges_directed(attribute_value_id, Direction::Outgoing)?
            {
                if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                    maybe_prop_node_index = Some(edge_ref.target());
                    prop_direction = Direction::Outgoing;
                }
                let discrim: EdgeWeightKindDiscriminants = edge_ref.weight().kind().into();
                if discrim == EdgeWeightKindDiscriminants::Prototype {
                    maybe_prototype_node_index = Some(edge_ref.target());
                }
            }

            let prototype_node_index = maybe_prototype_node_index
                .ok_or(AttributeValueError::MissingPrototype(attribute_value_id))?;

            let prototype_id = AttributePrototypeId::from(
                workspace_snapshot
                    .get_node_weight(prototype_node_index)?
                    .id(),
            );

            if maybe_prop_node_index.is_none() {
                for edge_ref in
                    workspace_snapshot.edges_directed(attribute_value_id, Direction::Incoming)?
                {
                    if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                        maybe_prop_node_index = Some(edge_ref.target());
                        prop_direction = Direction::Incoming;
                    }
                }
            }

            let intrinsic_func = match maybe_prop_node_index {
                Some(prop_node_index) => {
                    if let NodeWeight::Prop(prop_inner) =
                        workspace_snapshot.get_node_weight(prop_node_index)?
                    {
                        // None for the value means there is no value, so we use unset, but if it's a
                        // literal serde_json::Value::Null it means the value is set, but to null
                        if value.is_none() {
                            IntrinsicFunc::Unset
                        } else {
                            match prop_inner.kind() {
                                PropKind::Array => IntrinsicFunc::SetArray,
                                PropKind::Boolean => IntrinsicFunc::SetBoolean,
                                PropKind::Integer => IntrinsicFunc::SetInteger,
                                PropKind::Map => IntrinsicFunc::SetMap,
                                PropKind::Object => IntrinsicFunc::SetObject,
                                PropKind::String => IntrinsicFunc::SetString,
                            }
                        }
                    } else {
                        Err(AttributeValueError::NodeWeightMismatch(
                            prop_node_index,
                            NodeWeightDiscriminants::Prop,
                        ))?
                    }
                }
                None => match value {
                    None | Some(serde_json::Value::Null) => IntrinsicFunc::Unset,
                    Some(serde_json::Value::Array(_)) => IntrinsicFunc::SetArray,
                    Some(serde_json::Value::Bool(_)) => IntrinsicFunc::SetBoolean,
                    Some(serde_json::Value::Number(_)) => IntrinsicFunc::SetInteger,
                    Some(serde_json::Value::Object(_)) => IntrinsicFunc::SetObject,
                    Some(serde_json::Value::String(_)) => IntrinsicFunc::SetString,
                },
            };

            (intrinsic_func, prototype_id)
        };

        let func_id = Func::find_intrinsic(ctx, intrinsic_func)?;

        // If we have a prop, then we need to know if the edge to it was incoming or outgoing (found
        // above). If the edge is outgoing, we need to break the link from the value to the prototype
        // and create a new one. If the edge is incoming, we need to update the prototype directly.
        if maybe_prop_node_index.is_some() {
            match prop_direction {
                Direction::Outgoing => {
                    {
                        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
                        let attribute_value_node_index =
                            workspace_snapshot.get_node_index_by_id(attribute_value_id)?;
                        let attribute_prototype_node_index =
                            workspace_snapshot.get_node_index_by_id(attribute_prototype_id)?;

                        workspace_snapshot.remove_edge(
                            ctx.change_set_pointer()?,
                            attribute_value_node_index,
                            attribute_prototype_node_index,
                            EdgeWeightKindDiscriminants::Use,
                        )?;
                    }

                    AttributePrototype::new(ctx, func_id)?;
                }
                Direction::Incoming => {
                    AttributePrototype::update_func_by_id(ctx, attribute_prototype_id, func_id)?;
                }
            }
        }

        let processed = match &value {
            Some(Value::Object(_)) => Some(serde_json::json![{}]),
            Some(Value::Array(_)) => Some(serde_json::json![[]]),
            value => value.to_owned(),
        };
        Self::set_real_values(ctx, attribute_value_id, processed, value).await?;
        Ok(())
    }

    async fn set_real_values(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        unprocessed_value: Option<serde_json::Value>,
    ) -> AttributeValueResult<AttributeValue> {
        let (_, inner) = Self::get_content(ctx, attribute_value_id).await?;
        let attribute_value = Self::assemble(attribute_value_id, inner).modify(ctx, |av| {
            av.value = value;
            av.unprocessed_value = unprocessed_value;
            Ok(())
        })?;
        Ok(attribute_value)
    }

    fn modify<L>(self, ctx: &DalContext, lambda: L) -> AttributeValueResult<Self>
    where
        L: FnOnce(&mut Self) -> AttributeValueResult<()>,
    {
        let mut attribute_value = self;

        let before = AttributeValueContentV1::from(attribute_value.clone());
        lambda(&mut attribute_value)?;
        let updated = AttributeValueContentV1::from(attribute_value.clone());

        if updated != before {
            let hash = ctx
                .content_store()
                .try_lock()?
                .add(&AttributeValueContent::V1(updated.clone()))?;

            let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;
            workspace_snapshot.update_content(
                ctx.change_set_pointer()?,
                attribute_value.id.into(),
                hash,
            )?;
        }
        Ok(attribute_value)
    }

    async fn get_content(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<(ContentHash, AttributeValueContentV1)> {
        let id: Ulid = attribute_value_id.into();

        let mut workspace_snapshot = ctx.workspace_snapshot()?.try_lock()?;

        let node_index = workspace_snapshot.get_node_index_by_id(id)?;
        let node_weight = workspace_snapshot.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: AttributeValueContent = ctx
            .content_store()
            .try_lock()?
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let AttributeValueContent::V1(inner) = content;

        Ok((hash, inner))
    }
}

// impl AttributeValue {
//     standard_model_accessor!(
//         proxy_for_attribute_value_id,
//         Option<Pk(AttributeValueId)>,
//         AttributeValueResult
//     );
//     standard_model_accessor!(sealed_proxy, bool, AttributeValueResult);
//     standard_model_accessor!(func_binding_id, Pk(FuncBindingId), AttributeValueResult);
//     standard_model_accessor!(
//         func_binding_return_value_id,
//         Pk(FuncBindingReturnValueId),
//         AttributeValueResult
//     );
//     standard_model_accessor!(index_map, Option<IndexMap>, AttributeValueResult);
//     standard_model_accessor!(key, Option<String>, AttributeValueResult);

//     standard_model_belongs_to!(
//         lookup_fn: parent_attribute_value,
//         set_fn: set_parent_attribute_value_unchecked,
//         unset_fn: unset_parent_attribute_value,
//         table: "attribute_value_belongs_to_attribute_value",
//         model_table: "attribute_values",
//         belongs_to_id: AttributeValueId,
//         returns: AttributeValue,
//         result: AttributeValueResult,
//     );

//     standard_model_has_many!(
//         lookup_fn: child_attribute_values,
//         table: "attribute_value_belongs_to_attribute_value",
//         model_table: "attribute_values",
//         returns: AttributeValue,
//         result: AttributeValueResult,
//     );

//     standard_model_belongs_to!(
//         lookup_fn: attribute_prototype,
//         set_fn: set_attribute_prototype,
//         unset_fn: unset_attribute_prototype,
//         table: "attribute_value_belongs_to_attribute_prototype",
//         model_table: "attribute_prototypes",
//         belongs_to_id: AttributePrototypeId,
//         returns: AttributePrototype,
//         result: AttributeValueResult,
//     );

//     pub fn index_map_mut(&mut self) -> Option<&mut IndexMap> {
//         self.index_map.as_mut()
//     }

// /// Returns the *unprocessed* [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
// /// corresponding to the field on [`Self`].
// pub async fn get_unprocessed_value(
//     &self,
//     ctx: &DalContext,
// ) -> AttributeValueResult<Option<serde_json::Value>> {
//     match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
//         Some(func_binding_return_value) => {
//             Ok(func_binding_return_value.unprocessed_value().cloned())
//         }
//         None => Err(AttributeValueError::MissingFuncBindingReturnValue),
//     }
// }

//     /// Returns the [`serde_json::Value`] within the [`FuncBindingReturnValue`](crate::FuncBindingReturnValue)
//     /// corresponding to the field on [`Self`].
//     pub async fn get_value(
//         &self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<Option<serde_json::Value>> {
//         match FuncBindingReturnValue::get_by_id(ctx, &self.func_binding_return_value_id).await? {
//             Some(func_binding_return_value) => Ok(func_binding_return_value.value().cloned()),
//             None => Err(AttributeValueError::MissingFuncBindingReturnValue),
//         }
//     }

//     pub async fn update_stored_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
//         standard_model::update(
//             ctx,
//             "attribute_values",
//             "index_map",
//             self.id(),
//             &self.index_map,
//             TypeHint::JsonB,
//         )
//         .await?;
//         Ok(())
//     }

//     /// Returns a list of child [`AttributeValues`](crate::AttributeValue) for a given
//     /// [`AttributeValue`] and [`AttributeReadContext`](crate::AttributeReadContext).
//     pub async fn child_attribute_values_for_context(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         attribute_read_context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 CHILD_ATTRIBUTE_VALUES_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &attribute_value_id,
//                     &attribute_read_context,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     pub async fn find_with_parent_and_prototype_for_context(
//         ctx: &DalContext,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         attribute_prototype_id: AttributePrototypeId,
//         context: AttributeContext,
//     ) -> AttributeValueResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_WITH_PARENT_AND_PROTOTYPE_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &attribute_prototype_id,
//                     &parent_attribute_value_id,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::option_object_from_row(row)?)
//     }

//     /// Find [`Self`] with a given parent value and key.
//     pub async fn find_with_parent_and_key_for_context(
//         ctx: &DalContext,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         key: Option<String>,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Option<Self>> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 FIND_WITH_PARENT_AND_KEY_FOR_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &parent_attribute_value_id,
//                     &key,
//                 ],
//             )
//             .await?;

//         Ok(standard_model::option_object_from_row(row)?)
//     }

//     /// List [`AttributeValues`](crate::AttributeValue) for a provided
//     /// [`AttributeReadContext`](crate::AttributeReadContext).
//     ///
//     /// If you only anticipate one result to be returned and have an
//     /// [`AttributeReadContext`](crate::AttributeReadContext)
//     /// that is also a valid [`AttributeContext`](crate::AttributeContext), then you should use
//     /// [`Self::find_for_context()`] instead of this method.
//     ///
//     /// This does _not_ work for maps and arrays, barring the _first_ instance of the array or map
//     /// object themselves! For those objects, please use
//     /// [`Self::find_with_parent_and_key_for_context()`].
//     pub async fn list_for_context(
//         ctx: &DalContext,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<Self>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_CONTEXT,
//                 &[ctx.tenancy(), ctx.visibility(), &context],
//             )
//             .await?;
//         Ok(standard_model::objects_from_rows(rows)?)
//     }

//     /// Find one [`AttributeValue`](crate::AttributeValue) for a provided
//     /// [`AttributeReadContext`](crate::AttributeReadContext).
//     ///
//     /// This is a modified version of [`Self::list_for_context()`] that requires an
//     /// [`AttributeReadContext`](crate::AttributeReadContext)
//     /// that is also a valid [`AttributeContext`](crate::AttributeContext) _and_ "pops" the first
//     /// row off the rows found (which are sorted from most to least specific). Thus, the "popped"
//     /// row will corresponding to the most specific [`AttributeValue`] found.
//     ///
//     /// This does _not_ work for maps and arrays, barring the _first_ instance of the array or map
//     /// object themselves! For those objects, please use
//     /// [`Self::find_with_parent_and_key_for_context()`].
//     pub async fn find_for_context(
//         ctx: &DalContext,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Option<Self>> {
//         AttributeContextBuilder::from(context).to_context()?;
//         let mut rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_FOR_CONTEXT,
//                 &[ctx.tenancy(), ctx.visibility(), &context],
//             )
//             .await?;
//         let maybe_row = rows.pop();
//         Ok(standard_model::option_object_from_row(maybe_row)?)
//     }

//     /// Return the [`Prop`] that the [`AttributeValueId`] belongs to,
//     /// following the relationship through [`AttributePrototype`].
//     pub async fn find_prop_for_value(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//     ) -> AttributeValueResult<Prop> {
//         let row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_one(
//                 FIND_PROP_FOR_VALUE,
//                 &[ctx.tenancy(), ctx.visibility(), &attribute_value_id],
//             )
//             .await?;

//         Ok(standard_model::object_from_row(row)?)
//     }

//     /// List [`AttributeValuePayloads`](AttributeValuePayload) for a given
//     /// [`context`](crate::AttributeReadContext), which must specify a
//     /// [`ComponentId`](crate::Component).
//     pub async fn list_payload_for_read_context(
//         ctx: &DalContext,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
//         let schema_variant_id = match context.component_id {
//             Some(component_id) if component_id != ComponentId::NONE => {
//                 let component = Component::get_by_id(ctx, &component_id)
//                     .await?
//                     .ok_or(AttributeValueError::ComponentNotFoundById(component_id))?;
//                 let schema_variant = component
//                     .schema_variant(ctx)
//                     .await
//                     .map_err(|e| AttributeValueError::Component(e.to_string()))?
//                     .ok_or(AttributeValueError::SchemaVariantNotFoundForComponent(
//                         component_id,
//                     ))?;
//                 *schema_variant.id()
//             }
//             _ => {
//                 return Err(AttributeValueError::MissingComponentInReadContext(context));
//             }
//         };

//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_PAYLOAD_FOR_READ_CONTEXT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &schema_variant_id,
//                 ],
//             )
//             .await?;
//         let mut result = Vec::new();
//         for row in rows.into_iter() {
//             let func_binding_return_value_json: serde_json::Value = row.try_get("object")?;
//             let func_binding_return_value: Option<FuncBindingReturnValue> =
//                 serde_json::from_value(func_binding_return_value_json)?;

//             let prop_json: serde_json::Value = row.try_get("prop_object")?;
//             let prop: Prop = serde_json::from_value(prop_json)?;

//             let attribute_value_json: serde_json::Value = row.try_get("attribute_value_object")?;
//             let attribute_value: AttributeValue = serde_json::from_value(attribute_value_json)?;

//             let parent_attribute_value_id: Option<AttributeValueId> =
//                 row.try_get("parent_attribute_value_id")?;

//             result.push(AttributeValuePayload::new(
//                 prop,
//                 func_binding_return_value,
//                 attribute_value,
//                 parent_attribute_value_id,
//             ));
//         }
//         Ok(result)
//     }

//     /// This method is similar to [`Self::list_payload_for_read_context()`], but it leverages a
//     /// root [`AttributeValueId`](crate::AttributeValue) in order to find payloads at any
//     /// root [`Prop`](crate::Prop) corresponding to the provided context and root value.
//     ///
//     /// Requirements for the [`AttributeReadContext`](crate::AttributeReadContext):
//     /// - [`PropId`](crate::Prop) must be set to [`None`]
//     /// - Both providers fields must be unset
//     pub async fn list_payload_for_read_context_and_root(
//         ctx: &DalContext,
//         root_attribute_value_id: AttributeValueId,
//         context: AttributeReadContext,
//     ) -> AttributeValueResult<Vec<AttributeValuePayload>> {
//         if context.has_prop_id()
//             || !context.has_unset_internal_provider()
//             || !context.has_unset_external_provider()
//         {
//             return Err(AttributeValueError::IncompatibleAttributeReadContext("incompatible attribute read context for query: prop must be empty and providers must be unset"));
//         }

//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 LIST_PAYLOAD_FOR_READ_CONTEXT_AND_ROOT,
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &context,
//                     &root_attribute_value_id,
//                 ],
//             )
//             .await?;

//         let mut result = Vec::new();
//         for row in rows.into_iter() {
//             let func_binding_return_value_json: serde_json::Value = row.try_get("object")?;
//             let func_binding_return_value: Option<FuncBindingReturnValue> =
//                 serde_json::from_value(func_binding_return_value_json)?;

//             let prop_json: serde_json::Value = row.try_get("prop_object")?;
//             let prop: Prop = serde_json::from_value(prop_json)?;

//             let attribute_value_json: serde_json::Value = row.try_get("attribute_value_object")?;
//             let attribute_value: AttributeValue = serde_json::from_value(attribute_value_json)?;

//             let parent_attribute_value_id: Option<AttributeValueId> =
//                 row.try_get("parent_attribute_value_id")?;

//             result.push(AttributeValuePayload::new(
//                 prop,
//                 func_binding_return_value,
//                 attribute_value,
//                 parent_attribute_value_id,
//             ));
//         }
//         Ok(result)
//     }

//     /// Update the [`AttributeValue`] for a specific [`AttributeContext`] to the given value. If the
//     /// given [`AttributeValue`] is for a different [`AttributeContext`] than the one provided, a
//     /// new [`AttributeValue`] will be created for the given [`AttributeContext`].
//     ///
//     /// By passing in [`None`] as the `value`, the caller is explicitly saying "this value does not
//     /// exist here". This is potentially useful for "tombstoning" values that have been inherited
//     /// from a less-specific [`AttributeContext`]. For example, if a value has been set for a
//     /// [`SchemaVariant`](crate::SchemaVariant), but we do not want that value to exist for a
//     /// specific [`Component`](crate::Component), we can update the variant's value to [`None`] in
//     /// an [`AttributeContext`] specific to that component.
//     ///
//     /// This method returns the following:
//     /// - the [`Option<serde_json::Value>`] that was passed in
//     /// - the updated [`AttributeValueId`](Self)
//     pub async fn update_for_context(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         Self::update_for_context_raw(
//             ctx,
//             attribute_value_id,
//             parent_attribute_value_id,
//             context,
//             value,
//             key,
//             true,
//             true,
//         )
//         .await
//     }

//     pub async fn update_for_context_without_propagating_dependent_values(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         Self::update_for_context_raw(
//             ctx,
//             attribute_value_id,
//             parent_attribute_value_id,
//             context,
//             value,
//             key,
//             true,
//             false,
//         )
//         .await
//     }

//     pub async fn update_for_context_without_creating_proxies(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         Self::update_for_context_raw(
//             ctx,
//             attribute_value_id,
//             parent_attribute_value_id,
//             context,
//             value,
//             key,
//             false,
//             true,
//         )
//         .await
//     }

//     #[allow(clippy::too_many_arguments)]
//     async fn update_for_context_raw(
//         ctx: &DalContext,
//         attribute_value_id: AttributeValueId,
//         parent_attribute_value_id: Option<AttributeValueId>,
//         context: AttributeContext,
//         value: Option<serde_json::Value>,
//         // TODO: Allow updating the key
//         key: Option<String>,
//         create_child_proxies: bool,
//         propagate_dependent_values: bool,
//     ) -> AttributeValueResult<(Option<serde_json::Value>, AttributeValueId)> {
//         // TODO(nick,paulo,zack,jacob): ensure we do not _have_ to do this in the future.
//         let ctx = &ctx.clone_without_deleted_visibility();

// let row = ctx.txns()
//     .await?
//     .pg()
//     .query_one(
//         "SELECT new_attribute_value_id FROM attribute_value_update_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7, $8)",
//     &[
//         ctx.tenancy(),
//         ctx.visibility(),
//         &attribute_value_id,
//         &parent_attribute_value_id,
//         &context,
//         &value,
//         &key,
//         &create_child_proxies,
//     ],
//     ).await?;

//         let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

//         // TODO(fnichol): we might want to fire off a status even at this point, however we've
//         // already updated the initial attribute value, so is there much value?

//         if propagate_dependent_values && !ctx.no_dependent_values() {
//             ctx.enqueue_job(DependentValuesUpdate::new(
//                 ctx.access_builder(),
//                 *ctx.visibility(),
//                 vec![new_attribute_value_id],
//             ))
//             .await?;
//         }

//         Ok((value, new_attribute_value_id))
//     }

//     /// Insert a new value under the parent [`AttributeValue`] in the given [`AttributeContext`]. This is mostly only
//     /// useful for adding elements to a [`PropKind::Array`], or to a [`PropKind::Map`]. Updating existing values in an
//     /// [`Array`](PropKind::Array), or [`Map`](PropKind::Map), and setting/updating all other [`PropKind`] should be
//     /// able to directly use [`update_for_context()`](AttributeValue::update_for_context()), as there will already be an
//     /// appropriate [`AttributeValue`] to use. By using this function,
//     /// [`update_for_context()`](AttributeValue::update_for_context()) is called after we have created an appropriate
//     /// [`AttributeValue`] to use.
//     #[instrument(skip_all, level = "debug")]
//     pub async fn insert_for_context(
//         ctx: &DalContext,
//         item_attribute_context: AttributeContext,
//         array_or_map_attribute_value_id: AttributeValueId,
//         value: Option<serde_json::Value>,
//         key: Option<String>,
//     ) -> AttributeValueResult<AttributeValueId> {
//         Self::insert_for_context_raw(
//             ctx,
//             item_attribute_context,
//             array_or_map_attribute_value_id,
//             value,
//             key,
//             true,
//         )
//         .await
//     }

//     #[instrument(skip_all, level = "debug")]
//     pub async fn insert_for_context_without_creating_proxies(
//         ctx: &DalContext,
//         parent_context: AttributeContext,
//         parent_attribute_value_id: AttributeValueId,
//         value: Option<serde_json::Value>,
//         key: Option<String>,
//     ) -> AttributeValueResult<AttributeValueId> {
//         Self::insert_for_context_raw(
//             ctx,
//             parent_context,
//             parent_attribute_value_id,
//             value,
//             key,
//             false,
//         )
//         .await
//     }

//     #[instrument(skip_all, level = "debug")]
//     async fn insert_for_context_raw(
//         ctx: &DalContext,
//         item_attribute_context: AttributeContext,
//         array_or_map_attribute_value_id: AttributeValueId,
//         value: Option<serde_json::Value>,
//         key: Option<String>,
//         create_child_proxies: bool,
//     ) -> AttributeValueResult<AttributeValueId> {
//         let row = ctx.txns().await?.pg().query_one(
//             "SELECT new_attribute_value_id FROM attribute_value_insert_for_context_raw_v1($1, $2, $3, $4, $5, $6, $7)",
//             &[
//                 ctx.tenancy(),
//                 ctx.visibility(),
//                 &item_attribute_context,
//                 &array_or_map_attribute_value_id,
//                 &value,
//                 &key,
//                 &create_child_proxies,
//             ],
//         ).await?;

//         let new_attribute_value_id: AttributeValueId = row.try_get("new_attribute_value_id")?;

//         if !ctx.no_dependent_values() {
//             ctx.enqueue_job(DependentValuesUpdate::new(
//                 ctx.access_builder(),
//                 *ctx.visibility(),
//                 vec![new_attribute_value_id],
//             ))
//             .await?;
//         }

//         Ok(new_attribute_value_id)
//     }

//     #[instrument(skip_all, level = "debug")]
//     pub async fn update_parent_index_map(&self, ctx: &DalContext) -> AttributeValueResult<()> {
//         let _row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 "SELECT attribute_value_update_parent_index_map_v1($1, $2, $3)",
//                 &[ctx.tenancy(), ctx.visibility(), &self.id],
//             )
//             .await?;

//         Ok(())
//     }

//     async fn populate_nested_values(
//         ctx: &DalContext,
//         parent_attribute_value_id: AttributeValueId,
//         update_context: AttributeContext,
//         unprocessed_value: serde_json::Value,
//     ) -> AttributeValueResult<()> {
//         let _row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 "SELECT attribute_value_populate_nested_values_v1($1, $2, $3, $4, $5)",
//                 &[
//                     ctx.tenancy(),
//                     ctx.visibility(),
//                     &parent_attribute_value_id,
//                     &update_context,
//                     &unprocessed_value,
//                 ],
//             )
//             .await?;

//         Ok(())
//     }

//     /// Convenience method to determine if this [`AttributeValue`](Self) is for the implicit
//     /// [`InternalProvider`](crate::InternalProvider) that represents the "snapshot" of the entire
//     /// [`Component`](crate::Component). This means that the [`Prop`](crate::Prop) that the
//     /// [`InternalProvider`](crate::InternalProvider) is sourcing its data from does not have a
//     /// parent [`Prop`](crate::Prop).
//     #[allow(unused)]
//     async fn is_for_internal_provider_of_root_prop(
//         &mut self,
//         ctx: &DalContext,
//     ) -> AttributeValueResult<bool> {
//         let maybe_row = ctx
//             .txns()
//             .await?
//             .pg()
//             .query_opt(
//                 IS_FOR_INTERNAL_PROVIDER_OF_ROOT_PROP,
//                 &[&ctx.tenancy(), ctx.visibility(), &self.context],
//             )
//             .await?;
//         if let Some(row) = maybe_row {
//             // If we got a row back, that means that we are an AttributeValue for an InternalProvider,
//             // and we should have gotten a row back from the query.
//             Ok(row.try_get("is_for_root_prop")?)
//         } else {
//             // If we didn't get a row back, that means that we didn't find an InternalProvider for the
//             // InternalProviderId in our AttributeContext. Likely because it is ident_nil_v1, indicating that we're
//             // not for an InternalProvider at all.
//             Ok(false)
//         }
//     }

//     #[instrument(skip(ctx), level = "debug")]
//     pub async fn create_dependent_values(
//         ctx: &DalContext,
//         attribute_value_ids: &[AttributeValueId],
//     ) -> AttributeValueResult<()> {
//         ctx.txns()
//             .await?
//             .pg()
//             .execute(
//                 "SELECT attribute_value_create_new_affected_values_v1($1, $2, $3)",
//                 &[&ctx.tenancy(), &ctx.visibility(), &attribute_value_ids],
//             )
//             .await?;
//         Ok(())
//     }

//     /// Returns a [`HashMap`] with key [`AttributeValueId`](Self) and value
//     /// [`Vec<AttributeValueId>`](Self) where the keys correspond to [`AttributeValues`](Self) that
//     /// are affected (directly and indirectly) by at least one of the provided
//     /// [`AttributeValueIds`](Self) having a new value. The [`Vec<AttributeValueId>`](Self)
//     /// correspond to the [`AttributeValues`](Self) that the key directly depends on that are also
//     /// affected by at least one of the provided [`AttributeValueIds`](Self) having a new value.
//     ///
//     /// **NOTE**: This has the side effect of **CREATING NEW [`AttributeValues`](Self)**
//     /// if this [`AttributeValue`] affects an [`AttributeContext`](crate::AttributeContext) where an
//     /// [`AttributePrototype`](crate::AttributePrototype) that uses it didn't already have an
//     /// [`AttributeValue`].
//     #[instrument(skip(ctx), level = "debug")]
//     pub async fn dependent_value_graph(
//         ctx: &DalContext,
//         attribute_value_ids: &[AttributeValueId],
//     ) -> AttributeValueResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
//         let rows = ctx
//             .txns()
//             .await?
//             .pg()
//             .query(
//                 FETCH_UPDATE_GRAPH_DATA,
//                 &[&ctx.tenancy(), ctx.visibility(), &attribute_value_ids],
//             )
//             .instrument(debug_span!("Graph SQL query"))
//             .await?;

//         let mut result: HashMap<AttributeValueId, Vec<AttributeValueId>> = HashMap::new();
//         for row in rows.into_iter() {
//             let attr_val_id: AttributeValueId = row.try_get("attribute_value_id")?;
//             let dependencies: Vec<AttributeValueId> =
//                 row.try_get("dependent_attribute_value_ids")?;
//             result.insert(attr_val_id, dependencies);
//         }

//         Ok(result)
//     }

// pub async fn vivify_value_and_parent_values(
//     &self,
//     ctx: &DalContext,
// ) -> AttributeValueResult<AttributeValueId> {
//     let row = ctx.txns().await?.pg().query_one(
//         "SELECT new_attribute_value_id FROM attribute_value_vivify_value_and_parent_values_raw_v1($1, $2, $3, $4, $5)",
//     &[
//         ctx.tenancy(),
//         ctx.visibility(),
//         &self.context,
//         &self.id,
//         &true
//     ]).await?;

//         Ok(row.try_get("new_attribute_value_id")?)
//     }

// /// Re-evaluates the current `AttributeValue`'s `AttributePrototype` to update the
// /// `FuncBinding`, and `FuncBindingReturnValue`, reflecting the current inputs to
// /// the function.
// ///
// /// If the `AttributeValue` represents the `InternalProvider` for a `Prop` that
// /// does not have a parent `Prop` (this is typically the `InternalProvider` for
// /// the "root" `Prop` of a `SchemaVariant`), then it will also enqueue a
// /// `CodeGeneration` job for the `Component`.
// #[instrument(
//     name = "attribute_value.update_from_prototype_function",
//     skip_all,
//     level = "debug",
//     fields(
//         attribute_value.id = %self.id,
//         change_set_pk = %ctx.visibility().change_set_pk,
//     )
// )]
// pub async fn update_from_prototype_function(
//     &mut self,
//     ctx: &DalContext,
// ) -> AttributeValueResult<()> {
//     // Check if this AttributeValue is for an implicit InternalProvider as they have special behavior that doesn't involve
//     // AttributePrototype and AttributePrototypeArguments.
//     if self
//         .context
//         .is_least_specific_field_kind_internal_provider()?
//     {
//         let internal_provider =
//             InternalProvider::get_by_id(ctx, &self.context.internal_provider_id())
//                 .await?
//                 .ok_or_else(|| {
//                     AttributeValueError::InternalProviderNotFound(
//                         self.context.internal_provider_id(),
//                     )
//                 })?;
//         if internal_provider.is_internal_consumer() {
//             // We don't care about the AttributeValue that comes back from implicit_emit, since we should already be
//             // operating on an AttributeValue that has the correct AttributeContext, which means that a new one should
//             // not need to be created.
//             internal_provider
//                 .implicit_emit(ctx, self)
//                 .await
//                 .map_err(|e| AttributeValueError::InternalProvider(e.to_string()))?;

//                 debug!("InternalProvider is internal consumer");

//                 return Ok(());
//             }
//         } else if self.context.is_least_specific_field_kind_prop()? {
//             if let Some(parent_attribute_value) = self.parent_attribute_value(ctx).await? {
//                 parent_attribute_value
//                     .vivify_value_and_parent_values(ctx)
//                     .await?;
//             }
//         }

//         // The following should handle explicit "normal" Attributes, InternalProviders, and ExternalProviders already.
//         let attribute_prototype = self.attribute_prototype(ctx).await?.ok_or_else(|| {
//             AttributeValueError::AttributePrototypeNotFound(self.id, *ctx.visibility())
//         })?;

// // Note(victor): Secrets should never be passed to functions as arguments directly.
// // We detect if they're set as dependencies and later fetch before functions to execute
// // This is so secret values still trigger the dependent values system,
// // and before functions are only called when necessary
// let mut has_secrets_as_arg = false;
// let mut func_binding_args: HashMap<String, Option<serde_json::Value>> = HashMap::new();
// for mut argument_data in attribute_prototype
//     .argument_values(ctx, self.context)
//     .await
//     .map_err(|e| AttributeValueError::AttributePrototype(e.to_string()))?
// {
//     if argument_data.argument_name == "secrets" {
//         has_secrets_as_arg = true;
//         continue;
//     }

//     match argument_data.values.len() {
//         1 => {
//             let argument = argument_data.values.pop().ok_or_else(|| {
//                 AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
//                     argument_data.argument_name.clone(),
//                 )
//             })?;

//                     func_binding_args.insert(
//                         argument_data.argument_name,
//                         Some(serde_json::to_value(argument)?),
//                     );
//                 }
//                 2.. => {
//                     func_binding_args.insert(
//                         argument_data.argument_name,
//                         Some(serde_json::to_value(argument_data.values)?),
//                     );
//                 }
//                 _ => {
//                     return Err(
//                         AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
//                             argument_data.argument_name,
//                         ),
//                     );
//                 }
//             };
//         }

// let func_id = attribute_prototype.func_id();

// let before = if has_secrets_as_arg {
//     // We need the associated [`ComponentId`] for this function--this is how we resolve and
//     // prepare before functions
//     let associated_component_id = self.context.component_id();

//     before_funcs_for_component(ctx, &associated_component_id).await?
// } else {
//     vec![]
// };

// let (func_binding, mut func_binding_return_value) = match FuncBinding::create_and_execute(
//     ctx,
//     serde_json::to_value(func_binding_args.clone())?,
//     attribute_prototype.func_id(),
//     before,
// )
// .instrument(debug_span!(
//     "Func execution",
//     "func.id" = %func_id,
//     ?func_binding_args,
// ))
// .await
// {
//     Ok(function_return_value) => function_return_value,
//     Err(FuncBindingError::FuncBackendResultFailure {
//         kind,
//         message,
//         backend,
//     }) => {
//         return Err(AttributeValueError::FuncBackendResultFailure {
//             kind,
//             message,
//             backend,
//         })
//     }
//     Err(err) => Err(err)?,
// };

//         self.set_func_binding_id(ctx, *func_binding.id()).await?;
//         self.set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
//             .await?;

//         // If the value we just updated was for a Prop, we might have run a function that
//         // generates a deep data structure. If the Prop is an Array/Map/Object, then the
//         // value should be an empty Array/Map/Object, while the unprocessed value contains
//         // the deep data structure.
//         if self.context.is_least_specific_field_kind_prop()? {
//             let processed_value = match func_binding_return_value.unprocessed_value().cloned() {
//                 Some(unprocessed_value) => {
//                     let prop = Prop::get_by_id(ctx, &self.context.prop_id())
//                         .await?
//                         .ok_or_else(|| AttributeValueError::PropNotFound(self.context.prop_id()))?;

//                     match prop.kind() {
//                         PropKind::Object | PropKind::Map => Some(serde_json::json!({})),
//                         PropKind::Array => Some(serde_json::json!([])),
//                         _ => Some(unprocessed_value),
//                     }
//                 }
//                 None => None,
//             };

//             func_binding_return_value
//                 .set_value(ctx, processed_value)
//                 .await?;
//         };
//         // If they are different from each other, then we know
//         // that we need to fully process the deep data structure, populating
//         // AttributeValues for the child Props.
//         // cannot be si:setArray / si:setMap / si:setObject
//         if self.context.prop_id() != PropId::NONE {
//             let prop = Prop::get_by_id(ctx, &self.context.prop_id())
//                 .await?
//                 .ok_or_else(|| AttributeValueError::PropNotFound(self.context.prop_id()))?;

//             if *prop.kind() == PropKind::Array
//                 || *prop.kind() == PropKind::Object
//                 || *prop.kind() == PropKind::Map
//             {
//                 let func_name = match *prop.kind() {
//                     PropKind::Array => "si:setArray",
//                     PropKind::Object => "si:setObject",
//                     PropKind::Map => "si:setMap",
//                     _ => unreachable!(),
//                 };

//                 let func = Func::find_by_attr(ctx, "name", &func_name)
//                     .await?
//                     .pop()
//                     .ok_or_else(|| AttributeValueError::MissingFunc(func_name.to_owned()))?;

//                 if attribute_prototype.func_id() != *func.id() {
//                     if let Some(unprocessed_value) =
//                         func_binding_return_value.unprocessed_value().cloned()
//                     {
//                         AttributeValue::populate_nested_values(
//                             ctx,
//                             self.id,
//                             self.context,
//                             unprocessed_value,
//                         )
//                         .await?;
//                     }
//                 }
//             }
//         }

//         Ok(())
//     }

// pub async fn populate_child_proxies_for_value(
//     &self,
//     ctx: &DalContext,
//     less_specific_attribute_value_id: AttributeValueId,
//     more_specific_context: AttributeContext,
// ) -> AttributeValueResult<Option<Vec<AttributeValueId>>> {
//     let row = ctx.txns().await?.pg().query_one(
//         "SELECT new_proxy_value_ids FROM attribute_value_populate_child_proxies_for_value_v1($1, $2, $3, $4, $5)",
//         &[
//             ctx.tenancy(),
//             ctx.visibility(),
//             &less_specific_attribute_value_id,
//             &more_specific_context,
//             self.id(),
//         ]
//     ).await?;

//         // Are we part of a map or array? Be sure to update the index map
//         if self.key.is_some() {
//             ctx.txns()
//                 .await?
//                 .pg()
//                 .query_opt(
//                     "SELECT * FROM attribute_value_update_parent_index_map_v1($1, $2, $3)",
//                     &[ctx.tenancy(), ctx.visibility(), self.id()],
//                 )
//                 .await?;
//         }

//         Ok(row.try_get("new_proxy_value_ids")?)
//     }
// }

// #[derive(Debug, Clone)]
// pub struct AttributeValuePayload {
//     pub prop: Prop,
//     pub func_binding_return_value: Option<FuncBindingReturnValue>,
//     pub attribute_value: AttributeValue,
//     pub parent_attribute_value_id: Option<AttributeValueId>,
// }

// impl AttributeValuePayload {
//     pub fn new(
//         prop: Prop,
//         func_binding_return_value: Option<FuncBindingReturnValue>,
//         attribute_value: AttributeValue,
//         parent_attribute_value_id: Option<AttributeValueId>,
//     ) -> Self {
//         Self {
//             prop,
//             func_binding_return_value,
//             attribute_value,
//             parent_attribute_value_id,
//         }
//     }
// }
