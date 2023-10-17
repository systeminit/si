use std::collections::{HashMap, VecDeque};

use content_store::{ContentHash, Store};
use petgraph::prelude::*;
use ulid::Ulid;

use crate::attribute::value::{AttributeValueContent, AttributeValueContentV1};
use crate::change_set_pointer::ChangeSetPointer;
use crate::func::intrinsics::IntrinsicFunc;

use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphError;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::workspace_snapshot::{
    serde_value_to_string_type, WorkspaceSnapshotError, WorkspaceSnapshotResult,
};
use crate::{
    AttributePrototypeId, AttributeValue, AttributeValueId, DalContext, FuncId, PropId, PropKind,
    Timestamp, WorkspaceSnapshot,
};

// pub enum AttributeValueParent {
//     // "More specific"
//     Component(ComponentId),

//     // "Least specific"
//     ExternalProvider(ExternalProviderId),
//     InternalProvider(InternalProviderId),
//     Prop(PropId),

//     // "I don't care, eventually my parent knows who they belong to"
//     AttributeValue(AttributeValueId),
// }

impl WorkspaceSnapshot {
    pub async fn attribute_value_create(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        ordered: bool,
    ) -> WorkspaceSnapshotResult<(AttributeValue, NodeIndex)> {
        let timestamp = Timestamp::now();

        let content = AttributeValueContentV1 {
            timestamp,
            unprocessed_value: None,
            value: None,
        };
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&AttributeValueContent::V1(content.clone()))?;

        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::AttributeValue(hash))?;
        let node_index = if ordered {
            self.working_copy()?
                .add_ordered_node(change_set, node_weight)?
        } else {
            self.working_copy()?.add_node(node_weight)?
        };

        Ok((
            AttributeValue::assemble(AttributeValueId::from(id), &content),
            node_index,
        ))
    }

    pub async fn attribute_value_update(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> WorkspaceSnapshotResult<()> {
        self.attribute_value_vivify_value_and_parent_values(ctx, change_set, attribute_value_id)
            .await?;
        self.attribute_value_set_value(ctx, change_set, attribute_value_id, value.clone())
            .await?;
        self.attribute_value_populate_nested_values(ctx, change_set, attribute_value_id, value)
            .await?;
        Ok(())
    }

    pub async fn attribute_value_insert(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        parent_attribute_value_id: AttributeValueId,
        key: Option<String>,
        value: Option<serde_json::Value>,
    ) -> WorkspaceSnapshotResult<()> {
        // Find the array or map prop.
        let prop_index = self
            .outgoing_targets_for_edge_weight_kind(
                parent_attribute_value_id.into(),
                EdgeWeightKindDiscriminants::Prop,
            )?
            .get(0)
            .copied()
            .ok_or(WorkspaceSnapshotError::AttributeValueMissingPropEdge(
                parent_attribute_value_id,
            ))?;
        let prop_node_weight = match self.get_node_weight(prop_index)?.clone() {
            NodeWeight::Prop(inner) => inner,
            _ => {
                return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                    prop_index,
                    "NodeWeight::Prop".into(),
                ))
            }
        };

        // Ensure it actually is an array or map prop.
        if prop_node_weight.kind() != PropKind::Array || prop_node_weight.kind() != PropKind::Map {
            return Err(WorkspaceSnapshotError::InsertionForInvalidPropKind(
                prop_node_weight.kind(),
            ));
        }

        // Find a singlular child prop for the map or an array prop (i.e. the "element" or "entry" prop").
        let prop_id = PropId::from(prop_node_weight.id());
        let child_prop_indices = self.outgoing_targets_for_edge_weight_kind(
            prop_node_weight.id(),
            EdgeWeightKindDiscriminants::Use,
        )?;
        if child_prop_indices.len() > 1 {
            return Err(WorkspaceSnapshotError::PropMoreThanOneChild(prop_id));
        }
        let element_prop_index = child_prop_indices
            .get(0)
            .ok_or(WorkspaceSnapshotError::PropMissingElementProp(prop_id))?
            .to_owned();
        let element_prop_node_weight = match self.get_node_weight(element_prop_index)?.clone() {
            NodeWeight::Prop(inner) => inner,
            _ => {
                return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                    element_prop_index,
                    "NodeWeight::Prop".into(),
                ))
            }
        };

        // Create the "element" attribute value in the array or map alongside an attribute prototype for it.
        let (new_attribute_value_node, new_attribute_value_index) = self
            .attribute_value_create(
                ctx,
                change_set,
                matches!(
                    element_prop_node_weight.kind(),
                    PropKind::Map | PropKind::Object | PropKind::Array
                ),
            )
            .await?;
        let parent_av_node_index = self.get_node_index_by_id(parent_attribute_value_id.into())?;
        self.working_copy()?.add_ordered_edge(
            change_set,
            parent_av_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Contain(key))?,
            new_attribute_value_index,
        )?;
        self.working_copy()?.add_edge(
            new_attribute_value_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Prop)?,
            element_prop_index,
        )?;
        let func_id = self.func_find_intrinsic(IntrinsicFunc::Unset)?;
        self.attribute_prototype_create(ctx, change_set, func_id)
            .await?;

        // The element has been created an inserted. Now, we can update it with the provided value.
        self.attribute_value_update(ctx, change_set, new_attribute_value_node.id, value)
            .await
    }

    async fn attribute_value_vivify_value_and_parent_values(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<()> {
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

            for edge_ref in self.edges_directed(attribute_value_id.into(), Outgoing)? {
                if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                    maybe_prop_node_index = Some(edge_ref.target());
                }

                if edge_ref.weight().kind() == &EdgeWeightKind::Provider {
                    maybe_provider_node_index = Some(edge_ref.target());
                }
            }

            if maybe_provider_node_index.is_none() || maybe_prop_node_index.is_none() {
                for edge_ref in self.edges_directed(attribute_value_id.into(), Incoming)? {
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
                return Err(WorkspaceSnapshotError::UnexpectedGraphLayout(
                    "found both an provider edge and an prop edge",
                ));
            }

            // We're set on a provider, so we should look up the prop (if any)
            if let Some(provider_node_index) = maybe_provider_node_index {
                let provider_id = self
                    .working_copy()?
                    .get_node_weight(provider_node_index)?
                    .id();

                maybe_prop_node_index = self
                    .incoming_sources_for_edge_weight_kind(
                        provider_id,
                        EdgeWeightKindDiscriminants::Prop,
                    )?
                    .get(0)
                    .copied();
            }

            let empty_value = match maybe_prop_node_index {
                Some(prop_node_index) => {
                    match self.working_copy()?.get_node_weight(prop_node_index).map(
                        |node_weight| {
                            if let NodeWeight::Prop(inner) = node_weight {
                                Some(inner.kind())
                            } else {
                                None
                            }
                        },
                    )? {
                        Some(PropKind::Array) => Some(serde_json::json!([])),
                        Some(PropKind::Map) | Some(PropKind::Object) => Some(serde_json::json!({})),

                        // This means we did not get a prop node weight despite the node index coming
                        // from a prop edge
                        None => {
                            return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                                prop_node_index,
                                "NodeWeight::Prop".into(),
                            ))
                        }
                        _ => None,
                    }
                }
                None => Some(serde_json::json!({})),
            };

            let (_, inner) = self
                .attribute_value_get_content(ctx, attribute_value_id)
                .await?;

            // If we have a set value, we don't need to vivify
            if inner.value.is_some() {
                return Ok(());
            } else {
                self.attribute_value_set_value(ctx, change_set, attribute_value_id, empty_value)
                    .await?;

                // This assumes the only incoming contain edge from an attribute value is from
                // another attribute value
                let maybe_parent_attribute_node_index = self
                    .incoming_sources_for_edge_weight_kind(
                        attribute_value_id.into(),
                        EdgeWeightKindDiscriminants::Contain,
                    )?
                    .get(0)
                    .copied();

                if let Some(node_index) = maybe_parent_attribute_node_index {
                    current_attribute_value_id = Some(AttributeValueId::from(
                        self.get_node_weight(node_index)?.id(),
                    ));
                } else {
                    current_attribute_value_id = None;
                }
            }
        }

        Ok(())
    }

    async fn create_nested_value(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        func_id: FuncId,
        prop_id: PropId,
        key: Option<String>,
    ) -> WorkspaceSnapshotResult<AttributeValueId> {
        let prop_node_index = self.get_node_index_by_id(prop_id.into())?;
        let prop_kind =
            if let NodeWeight::Prop(prop_inner) = self.get_node_weight(prop_node_index)? {
                prop_inner.kind()
            } else {
                return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                    prop_node_index,
                    "NodeWeight::Prop".into(),
                ));
            };

        let (new_attribute_value_node, new_attribute_value_index) =
            self.attribute_value_create(ctx, change_set, true).await?;

        let parent_av_node_index = self.get_node_index_by_id(attribute_value_id.into())?;
        self.working_copy()?.add_ordered_edge(
            change_set,
            parent_av_node_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Contain(key))?,
            new_attribute_value_index,
        )?;

        self.working_copy()?.add_edge(
            new_attribute_value_index,
            EdgeWeight::new(change_set, EdgeWeightKind::Prop)?,
            prop_node_index,
        )?;

        self.attribute_prototype_create(ctx, change_set, func_id)
            .await?;

        match prop_kind {
            PropKind::Object | PropKind::Map => {
                self.attribute_value_set_value(
                    ctx,
                    change_set,
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
                self.attribute_value_set_value(
                    ctx,
                    change_set,
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
                self.attribute_value_set_value(ctx, change_set, attribute_value_id, value)
                    .await?;
            }
        }

        Ok(new_attribute_value_node.id)
    }

    pub async fn attribute_value_populate_nested_values(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> WorkspaceSnapshotResult<()> {
        // Remove child attribute value edges
        for attribute_value_target in self.outgoing_targets_for_edge_weight_kind(
            attribute_value_id.into(),
            EdgeWeightKindDiscriminants::Contain,
        )? {
            let current_node_index = self.get_node_index_by_id(attribute_value_id.into())?;
            self.working_copy()?.remove_edge(
                change_set,
                current_node_index,
                attribute_value_target,
                EdgeWeightKindDiscriminants::Contain,
            )?;
        }

        let mut work_queue = VecDeque::from([(attribute_value_id, value)]);

        let unset_func_id = self.func_find_intrinsic(IntrinsicFunc::Unset)?;

        while let Some((attribute_value_id, maybe_value)) = work_queue.pop_front() {
            // We're only looking for props on outgoing edges because we're assuming this will only be used for
            // attribute values on components. For default values at the schema variant level, we're
            // planning to add a "const arg" node that contains the default input for the function that
            // sets the value on the prototype
            let prop_node_index = self
                .outgoing_targets_for_edge_weight_kind(
                    attribute_value_id.into(),
                    EdgeWeightKindDiscriminants::Prop,
                )?
                .get(0)
                .copied()
                .ok_or(WorkspaceSnapshotError::AttributeValueMissingPropEdge(
                    attribute_value_id,
                ))?;

            let (prop_kind, prop_id) =
                if let NodeWeight::Prop(prop_inner) = self.get_node_weight(prop_node_index)? {
                    (prop_inner.kind(), PropId::from(prop_inner.id()))
                } else {
                    return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                        prop_node_index,
                        "NodeWeight::Prop".into(),
                    ));
                };

            match prop_kind {
                PropKind::Object => {
                    let maybe_object_map = match maybe_value {
                        Some(serde_json::Value::Object(map)) => Some(map),
                        Some(value) => {
                            return Err(WorkspaceSnapshotError::TypeMismatch(
                                prop_kind,
                                serde_value_to_string_type(&value),
                            ));
                        }
                        None => None,
                    };

                    let child_prop_indexes = self.outgoing_targets_for_edge_weight_kind(
                        prop_id.into(),
                        EdgeWeightKindDiscriminants::Use,
                    )?;

                    let mut prop_map = HashMap::new();
                    for node_index in child_prop_indexes {
                        if let NodeWeight::Prop(prop_inner) = self.get_node_weight(node_index)? {
                            prop_map.insert(
                                prop_inner.name().to_string(),
                                (prop_inner.id(), prop_inner.kind()),
                            );
                        }
                    }

                    // Remove keys from our value if there is no corresponding child prop
                    let maybe_object_map = maybe_object_map.map(|mut map| {
                        map.retain(|k, _| prop_map.contains_key(k));
                        map
                    });

                    for (key, (prop_id, prop_kind)) in prop_map.into_iter() {
                        let field_value = maybe_object_map
                            .as_ref()
                            .and_then(|map| map.get(&key).cloned());

                        let new_attribute_value_id = self
                            .create_nested_value(
                                ctx,
                                change_set,
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
                                    work_queue.push_back((new_attribute_value_id, field_value));
                                }
                            }
                            PropKind::Object => {
                                work_queue.push_back((new_attribute_value_id, field_value));
                            }
                            _ => {}
                        }
                    }
                }
                PropKind::Array => {
                    let array_items = match maybe_value {
                        Some(serde_json::Value::Array(array)) => {
                            if array.is_empty() {
                                continue;
                            }
                            array
                        }
                        Some(value) => {
                            return Err(WorkspaceSnapshotError::TypeMismatch(
                                prop_kind,
                                serde_value_to_string_type(&value),
                            ));
                        }
                        None => continue,
                    };

                    // find the child element prop
                    let child_props = self.outgoing_targets_for_edge_weight_kind(
                        prop_id.into(),
                        EdgeWeightKindDiscriminants::Use,
                    )?;

                    if child_props.len() > 1 {
                        return Err(WorkspaceSnapshotError::PropMoreThanOneChild(prop_id));
                    }

                    let element_prop_index = child_props
                        .get(0)
                        .ok_or(WorkspaceSnapshotError::PropMissingElementProp(prop_id))?
                        .to_owned();

                    let (element_prop_id, element_prop_kind) =
                        match self.get_node_weight(element_prop_index)? {
                            NodeWeight::Prop(prop_inner) => (prop_inner.id(), prop_inner.kind()),
                            _ => {
                                return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                                    element_prop_index,
                                    "NodeWeight::Prop".into(),
                                ))
                            }
                        };

                    for array_item in array_items {
                        // TODO: should we type check the values here against the element prop?
                        let array_item_value = Some(array_item);
                        let new_attribute_value_id = self
                            .create_nested_value(
                                ctx,
                                change_set,
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
                                    work_queue
                                        .push_back((new_attribute_value_id, array_item_value));
                                }
                            }
                            PropKind::Object => {
                                work_queue.push_back((new_attribute_value_id, array_item_value));
                            }
                            _ => {}
                        }
                    }
                }
                PropKind::Map => {
                    let map_map = match maybe_value {
                        Some(serde_json::Value::Object(map)) => {
                            if map.is_empty() {
                                continue;
                            }
                            map
                        }
                        Some(value) => {
                            return Err(WorkspaceSnapshotError::TypeMismatch(
                                prop_kind,
                                serde_value_to_string_type(&value),
                            ));
                        }
                        None => continue,
                    };

                    // find the child element prop
                    let child_props = self.outgoing_targets_for_edge_weight_kind(
                        prop_id.into(),
                        EdgeWeightKindDiscriminants::Use,
                    )?;

                    if child_props.len() > 1 {
                        return Err(WorkspaceSnapshotError::PropMoreThanOneChild(prop_id));
                    }

                    let element_prop_index = child_props
                        .get(0)
                        .ok_or(WorkspaceSnapshotError::PropMissingElementProp(prop_id))?
                        .to_owned();

                    let (element_prop_id, element_prop_kind) =
                        match self.get_node_weight(element_prop_index)? {
                            NodeWeight::Prop(prop_inner) => (prop_inner.id(), prop_inner.kind()),
                            _ => {
                                return Err(WorkspaceSnapshotError::NodeWeightMismatch(
                                    element_prop_index,
                                    "NodeWeight::Prop".into(),
                                ))
                            }
                        };

                    for (key, value) in map_map.into_iter() {
                        let value = Some(value);
                        let new_attribute_value_id = self
                            .create_nested_value(
                                ctx,
                                change_set,
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
                                    work_queue.push_back((new_attribute_value_id, value));
                                }
                            }
                            PropKind::Object => {
                                work_queue.push_back((new_attribute_value_id, value));
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn attribute_value_set_value(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> WorkspaceSnapshotResult<()> {
        let mut maybe_prop_node_index = None;
        let mut maybe_prototype_node_index = None;
        let mut prop_direction = Outgoing;
        for edge_ref in self.edges_directed(attribute_value_id.into(), Outgoing)? {
            if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                maybe_prop_node_index = Some(edge_ref.target());
                prop_direction = Outgoing;
            }
            if edge_ref.weight().kind() == &EdgeWeightKind::Prototype {
                maybe_prototype_node_index = Some(edge_ref.target());
            }
        }

        let prototype_node_index = maybe_prototype_node_index.ok_or(
            WorkspaceSnapshotError::AttributeValueMissingPrototype(attribute_value_id),
        )?;

        let prototype_id = AttributePrototypeId::from(
            self.working_copy()?
                .get_node_weight(prototype_node_index)?
                .id(),
        );

        if maybe_prop_node_index.is_none() {
            for edge_ref in self.edges_directed(attribute_value_id.into(), Incoming)? {
                if edge_ref.weight().kind() == &EdgeWeightKind::Prop {
                    maybe_prop_node_index = Some(edge_ref.target());
                    prop_direction = Incoming;
                }
            }
        }

        let intrinsic_func = match maybe_prop_node_index {
            Some(prop_node_index) => {
                if let NodeWeight::Prop(prop_inner) =
                    self.working_copy()?.get_node_weight(prop_node_index)?
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
                    Err(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
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

        let func_id = self.func_find_intrinsic(intrinsic_func)?;

        // If we have a prop, then we need to know if the edge to it was incoming or outgoing (found
        // above). If the edge is outgoing, we need to break the link from the value to the prototype
        // and create a new one. If the edge is incoming, we need to update the prototype directly.
        if maybe_prop_node_index.is_some() {
            match prop_direction {
                Direction::Outgoing => {
                    let attribute_value_node_idx = self
                        .working_copy()?
                        .get_node_index_by_id(attribute_value_id.into())?;

                    self.working_copy()?.remove_edge(
                        change_set,
                        attribute_value_node_idx,
                        prototype_node_index,
                        EdgeWeightKindDiscriminants::Use,
                    )?;

                    self.attribute_prototype_create(ctx, change_set, func_id)
                        .await?;
                }
                Direction::Incoming => {
                    self.attribute_prototype_update_func(change_set, prototype_id, func_id)?;
                }
            }
        }

        let processed = match &value {
            Some(serde_json::Value::Object(_)) => Some(serde_json::json![{}]),
            Some(serde_json::Value::Array(_)) => Some(serde_json::json![[]]),
            value => value.to_owned(),
        };
        self.attribute_value_set_real_values(ctx, change_set, attribute_value_id, processed, value)
            .await?;
        Ok(())
    }

    async fn attribute_value_get_content(
        &mut self,
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotResult<(ContentHash, AttributeValueContentV1)> {
        let id: Ulid = attribute_value_id.into();
        let node_index = self.working_copy()?.get_node_index_by_id(id)?;
        let node_weight = self.working_copy()?.get_node_weight(node_index)?;
        let hash = node_weight.content_hash();

        let content: AttributeValueContent = ctx
            .content_store()
            .lock()
            .await
            .get(&hash)
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let AttributeValueContent::V1(inner) = content;

        Ok((hash, inner))
    }

    async fn attribute_value_set_real_values(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
        unprocessed_value: Option<serde_json::Value>,
    ) -> WorkspaceSnapshotResult<AttributeValue> {
        let (_, inner) = self
            .attribute_value_get_content(ctx, attribute_value_id)
            .await?;
        let mut attribute_value = AttributeValue::assemble(attribute_value_id, &inner);

        attribute_value.value = value;
        attribute_value.unprocessed_value = unprocessed_value;

        let updated = AttributeValueContentV1::from(attribute_value.to_owned());
        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&AttributeValueContent::V1(updated))?;

        self.working_copy()?
            .update_content(change_set, attribute_value_id.into(), hash)?;

        Ok(attribute_value)
    }
}
