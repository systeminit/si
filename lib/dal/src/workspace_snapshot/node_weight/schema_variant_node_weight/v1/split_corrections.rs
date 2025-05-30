use std::collections::{
    BTreeMap,
    BTreeSet,
};

use si_id::{
    SchemaId,
    SchemaVariantId,
};
use si_split_graph::{
    SplitGraph,
    SplitGraphNodeWeight,
    SplitGraphResult,
    Update,
};

use super::SchemaVariantNodeWeightV1;
use crate::{
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{
            NodeWeight,
            traits::SiVersionedNodeWeight,
        },
        split_snapshot::{
            corrections::{
                CorrectTransforms,
                CorrectTransformsResult,
            },
            schema_ids_for_schema_variant_id,
        },
    },
};

fn schema_variant_id_lock_status_for_schema_id(
    graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    schema_id: SchemaId,
) -> SplitGraphResult<BTreeMap<SchemaVariantId, bool>> {
    Ok(graph
        .outgoing_targets(schema_id.into(), EdgeWeightKindDiscriminants::Use)?
        .filter_map(|variant_id| match graph.node_weight(variant_id) {
            Some(NodeWeight::SchemaVariant(variant)) => {
                Some((variant_id.into(), variant.inner().is_locked()))
            }
            _ => None,
        })
        .collect())
}

/// We need to ensure that there is only one unlocked variant for each schema in a change set. We do this
/// by figuring out if there are more than one unlocked variants for this variant's schema, and if so, we
/// produce ReplaceNode updates which will ensure all but one variant become locked
impl CorrectTransforms<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>
    for SchemaVariantNodeWeightV1
{
    fn correct_transforms(
        &self,
        graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        mut updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>>
    {
        let mut maybe_my_schema_id = if graph.node_exists(self.id) {
            schema_ids_for_schema_variant_id(graph, self.id.into())?
                .first()
                .copied()
        } else {
            None
        };

        let mut new_schemas = BTreeMap::new();
        let mut schema_variants_in_this_update_set = BTreeMap::new();
        let mut new_variants_for_schema: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
        let mut removed_variants = BTreeSet::new();
        let mut variant_lock_statuses = BTreeMap::new();

        let mut self_is_locked = self.is_locked();

        #[inline(always)]
        fn variant_is_locked(node_weight: &SplitGraphNodeWeight<NodeWeight>) -> bool {
            match node_weight {
                SplitGraphNodeWeight::Custom(NodeWeight::SchemaVariant(variant)) => {
                    variant.inner().is_locked()
                }
                _ => false,
            }
        }

        for update in &updates {
            match update {
                Update::NewNode {
                    subgraph_root_id,
                    node_weight,
                } => match node_weight {
                    SplitGraphNodeWeight::Custom(NodeWeight::Content(content_node_weight))
                        if content_node_weight.content_address_discriminants()
                            == ContentAddressDiscriminants::Schema =>
                    {
                        new_schemas.insert(node_weight.id(), subgraph_root_id);
                    }
                    SplitGraphNodeWeight::Custom(NodeWeight::SchemaVariant(_)) => {
                        schema_variants_in_this_update_set
                            .insert(node_weight.id(), (node_weight.clone(), *subgraph_root_id));
                        let is_locked = variant_is_locked(node_weight);
                        variant_lock_statuses.insert(node_weight.id(), is_locked);
                    }
                    _ => {}
                },
                Update::NewEdge { .. }
                    if update.is_edge_of_sort(
                        NodeWeightDiscriminants::Content,
                        EdgeWeightKindDiscriminants::Use,
                        NodeWeightDiscriminants::SchemaVariant,
                    ) =>
                {
                    let Some((schema_id, variant_id)) =
                        update.source_id().zip(update.destination_id())
                    else {
                        continue;
                    };

                    new_variants_for_schema
                        .entry(schema_id)
                        .and_modify(|entry| {
                            entry.insert(variant_id);
                        })
                        .or_insert_with(|| BTreeSet::from([variant_id]));

                    if variant_id == self.id {
                        maybe_my_schema_id = Some(schema_id.into());
                    }
                }
                Update::RemoveEdge { .. }
                    if update.is_edge_of_sort(
                        NodeWeightDiscriminants::Content,
                        EdgeWeightKindDiscriminants::Use,
                        NodeWeightDiscriminants::SchemaVariant,
                    ) =>
                {
                    let Some(variant_id) = update.destination_id() else {
                        continue;
                    };
                    removed_variants.insert(variant_id);
                }
                Update::ReplaceNode {
                    node_weight,
                    subgraph_root_id,
                    ..
                } if node_weight.custom_kind() == Some(NodeWeightDiscriminants::SchemaVariant) => {
                    let variant_id = node_weight.id();
                    // If this is false, this means the node has been removed from the graph
                    // we're applying the update to. So we don't care about it.
                    if !graph.node_exists(variant_id) {
                        continue;
                    }

                    schema_variants_in_this_update_set
                        .insert(node_weight.id(), (node_weight.clone(), *subgraph_root_id));

                    let is_locked = variant_is_locked(node_weight);
                    if variant_id == self.id {
                        self_is_locked = is_locked;
                    }

                    variant_lock_statuses.insert(variant_id, is_locked);
                }
                _ => {}
            }
        }

        // If we are locked, then we don't have to make any corrections, since
        // we don't have to prevent multiple unlocked resulting from this schema
        // variant being unlocked.
        if self_is_locked {
            return Ok(updates);
        }

        // At this point, we should already have a schema id. Either it was on the graph already,
        // or there was a NewEdge update for it in the set of updates. If neither of those are true,
        // then we're a variant without a schema. Which is tragic, but we can't do anything about it
        // here.
        let Some(my_schema_id) = maybe_my_schema_id else {
            return Ok(updates);
        };

        // Get all variant lock statuses on the graph we are modifying with these updates
        let mut variant_lock_statuses_for_schema_id = if graph.node_exists(my_schema_id.into()) {
            schema_variant_id_lock_status_for_schema_id(graph, my_schema_id)?
        } else {
            BTreeMap::new()
        };

        // Now, merge those lock statuses with the changes we have detected in this set of updates,

        // First, all the new variants that were added in this set of updates for this schema
        if let Some(new_variants_for_my_schema) = new_variants_for_schema.get(&my_schema_id.into())
        {
            for variant_id in new_variants_for_my_schema {
                if let Some(variant_lock_status) = variant_lock_statuses.get(variant_id) {
                    variant_lock_statuses_for_schema_id
                        .insert((*variant_id).into(), *variant_lock_status);
                }
            }
        }

        // Now, anything leftover, from the ReplaceNode updates. This will do a tiny bit of double work.
        for (variant_id, lock_status) in variant_lock_statuses {
            variant_lock_statuses_for_schema_id
                .entry(variant_id.into())
                .and_modify(|entry| *entry = lock_status);
        }

        // Find out which variants are unlocked after all the updates.
        let unlocked_variants_for_my_schema: Vec<_> = variant_lock_statuses_for_schema_id
            .iter()
            .filter(|(_, lock_status)| !**lock_status)
            .map(|(variant_id, _)| *variant_id)
            .collect();

        // We can only have one unlocked variant for a schema. If there are more than one,
        // we need to produce ReplaceNode updates to lock all but one of the variants.
        // First, if we don't have more than one unlocked, then we have nothing to do.
        if unlocked_variants_for_my_schema.len() <= 1 {
            return Ok(updates);
        }

        // If we're here, then more than one is unlocked. Find out which one should stay unlocked. We do this
        // by finding the unlocked variant with the latest timestamp.
        let mut winning_variant_node_weight: Option<&Self> = None;
        for variant_id in &unlocked_variants_for_my_schema {
            let variant_inner = if let Some((
                SplitGraphNodeWeight::Custom(NodeWeight::SchemaVariant(sv_node_weight)),
                _,
            )) = schema_variants_in_this_update_set.get(&(variant_id.into()))
            {
                Some(sv_node_weight.inner())
            } else if let Some(NodeWeight::SchemaVariant(sv_node_weight)) =
                graph.node_weight(variant_id.into())
            {
                Some(sv_node_weight.inner())
            } else {
                None
            };

            if let Some(variant_inner) = variant_inner {
                if winning_variant_node_weight.is_none()
                    || winning_variant_node_weight
                        .is_some_and(|winning| variant_inner.timestamp > winning.timestamp)
                {
                    winning_variant_node_weight = Some(variant_inner);
                }
            }
        }

        let Some(winning_variant_node_weight) = winning_variant_node_weight else {
            // Somehow, we did not find a winnner. This could only happen if there is an unlocked
            // variant that wasn't in the update set and wasn't on the graph. That seems impossible?
            // So this should never happen.
            return Ok(updates);
        };

        // Finally, produce updates which will lock the variants.
        // By appending these to the end, they will override any other ReplaceNode updates
        // that are in the update set.
        for variant_id in unlocked_variants_for_my_schema {
            if variant_id == winning_variant_node_weight.id.into() {
                continue;
            }

            if let Some((
                SplitGraphNodeWeight::Custom(NodeWeight::SchemaVariant(sv_node_weight)),
                subgraph_root_id,
            )) = schema_variants_in_this_update_set.get(&variant_id.into())
            {
                let mut sv_node_weight = sv_node_weight.clone();
                sv_node_weight.inner_mut().set_is_locked(true);
                updates.push(Update::ReplaceNode {
                    subgraph_root_id: *subgraph_root_id,
                    base_graph_node_id: None,
                    node_weight: SplitGraphNodeWeight::Custom(NodeWeight::SchemaVariant(
                        sv_node_weight,
                    )),
                });
            } else if let Some(NodeWeight::SchemaVariant(sv_node_weight)) =
                graph.node_weight(variant_id.into())
            {
                let mut sv_node_weight = sv_node_weight.clone();
                sv_node_weight.inner_mut().set_is_locked(true);
                if let Some(subgraph_root_id) = graph.subgraph_root_id_for_node(variant_id.into()) {
                    updates.push(Update::ReplaceNode {
                        subgraph_root_id,
                        base_graph_node_id: None,
                        node_weight: SplitGraphNodeWeight::Custom(NodeWeight::SchemaVariant(
                            sv_node_weight,
                        )),
                    });
                }
            }
        }

        Ok(updates)
    }
}
