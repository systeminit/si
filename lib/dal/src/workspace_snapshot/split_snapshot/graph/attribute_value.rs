use std::collections::{
    HashMap,
    VecDeque,
};

use petgraph::Direction;
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
    PropId,
};
use si_split_graph::SplitGraphError;

use crate::{
    EdgeWeightKind,
    PropKind,
    attribute::value::{
        AttributeValueError,
        AttributeValueResult,
    },
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants,
        graph::traits::{
            attribute_value::{
                AttributeValueExt,
                AttributeValueTree,
                AttributeValueTreeEntry,
            },
            prop::PropExt as _,
        },
        split_snapshot::SplitSnapshotGraphV1,
    },
};

impl AttributeValueExt for SplitSnapshotGraphV1 {
    fn attribute_value_tree(
        &self,
        root_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueTree> {
        let root_prop_id = self
            .prop_for_attribute_value_id(root_attribute_value_id)?
            .ok_or_else(|| AttributeValueError::PropNotFound(root_attribute_value_id))?;
        let root_prop_kind = self
            .node_weight(root_prop_id.into())
            .ok_or_else(|| SplitGraphError::NodeNotFound(root_prop_id.into()))?
            .get_prop_node_weight()?
            .kind();
        let mut tree =
            AttributeValueTree::new(root_attribute_value_id, root_prop_id, root_prop_kind);
        let mut work_queue = VecDeque::from([root_attribute_value_id]);

        while let Some(attribute_value_id) = work_queue.pop_front() {
            let children = self.child_attribute_values_in_order(attribute_value_id)?;
            work_queue.extend(&children);

            let mut child_tree_entries = Vec::with_capacity(children.len());
            for child_id in children {
                let child_prop_id = self
                    .prop_for_attribute_value_id(child_id)?
                    .ok_or_else(|| AttributeValueError::PropNotFound(child_id))?;
                let child_prop_kind = self
                    .node_weight(child_prop_id.into())
                    .ok_or_else(|| SplitGraphError::NodeNotFound(root_prop_id.into()))?
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
        Ok(self
            .outgoing_targets(
                attribute_value_id.into(),
                EdgeWeightKindDiscriminants::Contain,
            )?
            .map(Into::into)
            .collect())
    }

    fn child_attribute_values_in_order(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let prop_id = self
            .prop_for_attribute_value_id(attribute_value_id)?
            .ok_or_else(|| AttributeValueError::PropNotFound(attribute_value_id))?;
        let prop = self
            .node_weight(prop_id.into())
            .ok_or_else(|| SplitGraphError::NodeNotFound(prop_id.into()))?
            .get_prop_node_weight()?;

        match prop.kind() {
            PropKind::Boolean
            | PropKind::Float
            | PropKind::Integer
            | PropKind::Json
            | PropKind::String => Ok(Vec::new()),
            PropKind::Array | PropKind::Map => Ok(self
                .ordered_children(attribute_value_id.into())
                .ok_or_else(|| {
                    AttributeValueError::NoOrderingNodeForAttributeValue(attribute_value_id)
                })?
                .iter()
                .copied()
                .map(Into::into)
                .collect()),
            PropKind::Object => {
                let mut child_av_ids = self.child_attribute_values(attribute_value_id)?;
                let child_props = self.ordered_child_prop_ids(prop_id)?;
                let mut avs_by_prop = HashMap::with_capacity(child_props.len());

                for &child_av_id in &child_av_ids {
                    let child_prop_id = self
                        .prop_for_attribute_value_id(child_av_id)?
                        .ok_or_else(|| AttributeValueError::PropNotFound(child_av_id))?;
                    avs_by_prop.insert(child_prop_id, child_av_id);
                }

                let mut ordered_child_av_ids: Vec<_> = child_props
                    .iter()
                    .filter_map(|child_prop_id| avs_by_prop.get(child_prop_id).copied())
                    .collect();

                // Ensure we haven't missed any child attribute values because of having multiple
                // for the same child Prop.
                //
                // See: https://linear.app/system-initiative/issue/BUG-878/multiple-avs-for-one-prop
                if ordered_child_av_ids.len() != child_av_ids.len() {
                    child_av_ids.retain(|av_id| !ordered_child_av_ids.contains(av_id));
                    // Sort whatever is left over by their IDs to keep them in a stable order.
                    child_av_ids.sort();
                    ordered_child_av_ids.append(&mut child_av_ids);
                }

                Ok(ordered_child_av_ids)
            }
        }
    }

    fn component_prototype_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>> {
        let mut iter = self.outgoing_targets(
            attribute_value_id.into(),
            EdgeWeightKindDiscriminants::Prototype,
        )?;

        match (iter.next(), iter.next()) {
            (None, None) => Ok(None),
            (Some(ap_id), None) => Ok(Some(ap_id.into())),
            (Some(_), Some(_)) => Err(AttributeValueError::MultiplePrototypesFound(
                attribute_value_id,
            )),
            (None, Some(_)) => unreachable!("iterator had none then some"),
        }
    }

    fn key_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<String>> {
        let mut incoming_edges = self.incoming_edges(
            attribute_value_id.into(),
            EdgeWeightKindDiscriminants::Contain,
        )?;
        match (incoming_edges.next(), incoming_edges.next()) {
            (None, None) => Ok(None),
            (Some(edge), None) => match edge.weight().kind() {
                EdgeWeightKind::Contain(key) => Ok(key.clone()),
                _ => unreachable!("incoming_edges iterator returned unexpected kind"),
            },
            (Some(_), Some(_)) => Err(SplitGraphError::TooManyEdgesOfKind(
                attribute_value_id.into(),
                Direction::Incoming,
                EdgeWeightKindDiscriminants::Contain.to_string(),
            )
            .into()),
            (None, Some(_)) => unreachable!("iterator had none then some"),
        }
    }

    fn prop_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<PropId>> {
        let mut prop_ids: Vec<_> = self
            .edges_directed_for_edge_weight_kind(
                attribute_value_id.into(),
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::Prop,
            )?
            .map(|edge_ref| edge_ref.target())
            .collect();
        match (prop_ids.pop(), prop_ids.pop()) {
            (Some(prop_id), None) => Ok(Some(prop_id.into())),
            (None, None) => Ok(None),
            (Some(prop_id), Some(second_prop_id)) => Err(AttributeValueError::MultiplePropsFound(
                prop_id.into(),
                second_prop_id.into(),
                attribute_value_id,
            )),
            (None, Some(_)) => unreachable!("Vec::pop() had None then Some"),
        }
    }
}
