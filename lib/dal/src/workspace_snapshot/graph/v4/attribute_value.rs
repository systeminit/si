use std::collections::{
    HashMap,
    VecDeque,
};

use si_id::{
    AttributePrototypeId,
    AttributeValueId,
    PropId,
};

use crate::{
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    PropKind,
    attribute::value::{
        AttributeValueError,
        AttributeValueResult,
    },
    workspace_snapshot::graph::{
        WorkspaceSnapshotGraphError,
        WorkspaceSnapshotGraphV4,
        traits::{
            attribute_value::{
                AttributeValueExt,
                AttributeValueTree,
                AttributeValueTreeEntry,
            },
            prop::PropExt as _,
        },
    },
};

impl AttributeValueExt for WorkspaceSnapshotGraphV4 {
    fn attribute_value_tree(
        &self,
        root_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueTree> {
        let root_prop_id = self
            .prop_for_attribute_value_id(root_attribute_value_id)?
            .ok_or_else(|| AttributeValueError::PropNotFound(root_attribute_value_id))?;
        let root_prop_kind = self
            .get_node_weight_by_id(root_prop_id.into())?
            .get_prop_node_weight()?
            .kind();
        let mut tree =
            AttributeValueTree::new(root_attribute_value_id, root_prop_id, root_prop_kind);
        let mut work_queue = VecDeque::from([root_attribute_value_id]);

        while let Some(attribute_value_id) = work_queue.pop_front() {
            let children = self.child_attribute_values_in_order(attribute_value_id)?;
            work_queue.reserve(children.len());
            work_queue.extend(&children);

            let mut child_tree_entries = Vec::with_capacity(children.len());
            for child_id in children {
                let child_prop_id = self
                    .prop_for_attribute_value_id(child_id)?
                    .ok_or_else(|| AttributeValueError::PropNotFound(child_id))?;
                let child_prop_kind = self
                    .get_node_weight_by_id(child_prop_id.into())?
                    .get_prop_node_weight()?
                    .kind();
                child_tree_entries.push(AttributeValueTreeEntry {
                    attribute_value_id: child_id,
                    prop_id: child_prop_id,
                    prop_kind: child_prop_kind,
                });
            }
            tree.add_children(attribute_value_id, child_tree_entries);
        }

        Ok(tree)
    }

    fn child_attribute_values(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let node_index = self.get_node_index_by_id(attribute_value_id)?;
        let child_idxs: Vec<_> = self
            .targets(node_index, EdgeWeightKindDiscriminants::Contain)
            .collect();
        let mut child_ids = Vec::with_capacity(child_idxs.len());
        for child_idx in child_idxs {
            child_ids.push(
                self.node_index_to_id(child_idx)
                    .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWithIndexNotFound(child_idx))?
                    .into(),
            );
        }

        Ok(child_ids)
    }

    fn child_attribute_values_in_order(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let node_index = self.get_node_index_by_id(attribute_value_id)?;
        let prop_id = self
            .prop_for_attribute_value_id(attribute_value_id)?
            .ok_or_else(|| AttributeValueError::PropNotFound(attribute_value_id))?;

        let prop_idx = self.get_node_index_by_id(prop_id)?;
        let prop = self
            .get_node_weight_by_id(prop_id)?
            .get_prop_node_weight()?;

        match prop.kind() {
            PropKind::Boolean
            | PropKind::Float
            | PropKind::Integer
            | PropKind::Json
            | PropKind::String => return Ok(Vec::new()),
            PropKind::Array | PropKind::Map => {
                let child_idxs = self.ordered_children_for_node(node_index)?.ok_or_else(|| {
                    AttributeValueError::NoOrderingNodeForAttributeValue(attribute_value_id)
                })?;
                let mut ordered_child_ids = Vec::with_capacity(child_idxs.len());
                for child_idx in child_idxs {
                    ordered_child_ids.push(
                        self.node_index_to_id(child_idx)
                            .ok_or_else(|| {
                                WorkspaceSnapshotGraphError::NodeWithIndexNotFound(child_idx)
                            })?
                            .into(),
                    );
                }

                Ok(ordered_child_ids)
            }
            PropKind::Object => {
                let mut child_av_ids = self.child_attribute_values(attribute_value_id)?;
                let child_props = self.ordered_child_prop_ids(prop_id)?;
                let mut av_by_prop = HashMap::with_capacity(child_props.len());

                for &child_av_id in &child_av_ids {
                    let child_prop_id = self
                        .prop_for_attribute_value_id(child_av_id)?
                        .ok_or_else(|| AttributeValueError::PropNotFound(child_av_id))?;
                    av_by_prop.insert(child_prop_id, child_av_id);
                }

                let mut ordered_child_av_ids: Vec<_> = child_props
                    .iter()
                    .filter_map(|child_prop_id| av_by_prop.get(child_prop_id).copied())
                    .collect();

                // Ensure we haven't missed any child attribute values because of having multiple
                // for the same child Prop.
                //
                // See: https://linear.app/system-initiative/issue/BUG-878/multiple-avs-for-one-prop
                if ordered_child_av_ids.len() != child_av_ids.len() {
                    child_av_ids.retain(|av_id| !ordered_child_av_ids.contains(av_id));
                    ordered_child_av_ids.append(&mut child_av_ids);
                }

                Ok(ordered_child_av_ids)
            }
        }
    }

    fn component_prototype_id(
        &self,
        id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        let node_idx = self
            .get_node_index_by_id(id)
            .map_err(|_| AttributeValueError::MissingForId(id))?;
        let maybe_prototype_idx = self
            .target_opt(node_idx, EdgeWeightKindDiscriminants::Prototype)
            .map_err(|_| AttributeValueError::MultiplePrototypesFound(id))?
            .and_then(|node_idx| self.node_index_to_id(node_idx).map(Into::into));

        Ok(maybe_prototype_idx)
    }

    fn key_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<String>> {
        let node_idx = self
            .get_node_index_by_id(attribute_value_id)
            .map_err(|_| AttributeValueError::MissingForId(attribute_value_id))?;
        let mut incoming_edges =
            self.incoming_edges(node_idx, EdgeWeightKindDiscriminants::Contain);
        match (incoming_edges.next(), incoming_edges.next()) {
            (Some(_), Some(_)) => Err(WorkspaceSnapshotGraphError::TooManyEdgesOfKind(
                node_idx,
                EdgeWeightKindDiscriminants::Contain,
            )
            .into()),
            (Some(edge_ref), None) => match edge_ref.weight().kind() {
                EdgeWeightKind::Contain(key) => Ok(key.clone()),
                kind @ _ => {
                    Err(WorkspaceSnapshotGraphError::InvalidEdgeWeightKind(kind.into()).into())
                }
            },
            (None, None) => Ok(None),
            (None, Some(_)) => unreachable!("iterator had none then some"),
        }
    }

    fn prop_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<PropId>> {
        let node_idx = self
            .get_node_index_by_id(attribute_value_id)
            .map_err(|_| AttributeValueError::MissingForId(attribute_value_id))?;
        let mut prop_edges = self.targets(node_idx, EdgeWeightKindDiscriminants::Prop);
        match (prop_edges.next(), prop_edges.next()) {
            (Some(prop_idx), None) => {
                let prop_id = self
                    .node_index_to_id(prop_idx)
                    .map(Into::into)
                    .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWithIndexNotFound(prop_idx))?;
                Ok(Some(prop_id))
            }
            (None, Some(_)) => unreachable!("iterator had none then some"),
            (Some(prop_idx), Some(second_prop_idx)) => {
                let prop_id = self
                    .node_index_to_id(prop_idx)
                    .map(Into::into)
                    .ok_or_else(|| WorkspaceSnapshotGraphError::NodeWithIndexNotFound(prop_idx))?;
                let second_prop_id = self
                    .node_index_to_id(second_prop_idx)
                    .map(Into::into)
                    .ok_or_else(|| {
                        WorkspaceSnapshotGraphError::NodeWithIndexNotFound(second_prop_idx)
                    })?;

                Err(AttributeValueError::MultiplePropsFound(
                    second_prop_id,
                    prop_id,
                    attribute_value_id,
                ))
            }
            (None, None) => Ok(None),
        }
    }
}
