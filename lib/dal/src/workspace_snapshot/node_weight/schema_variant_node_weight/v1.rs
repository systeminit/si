use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use super::{
    SchemaVariantNodeWeight,
    SchemaVariantNodeWeightError,
    SchemaVariantNodeWeightResult,
};
use crate::{
    DalContext,
    EdgeWeightKindDiscriminants,
    SchemaId,
    SchemaVariantError,
    SchemaVariantId,
    WorkspaceSnapshotGraphVCurrent,
    layer_db_types::{
        SchemaVariantContent,
        SchemaVariantContentV3,
    },
    workspace_snapshot::{
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        graph::{
            LineageId,
            WorkspaceSnapshotGraphError,
            WorkspaceSnapshotGraphV3,
            detector::Update,
        },
        node_weight::{
            self,
            ContentNodeWeight,
            NodeWeight,
            NodeWeightDiscriminants,
            NodeWeightError,
            traits::{
                CorrectExclusiveOutgoingEdge,
                CorrectTransforms,
                SiNodeWeight,
            },
        },
    },
};

mod split_corrections;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiNodeWeight, Hash)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::SchemaVariant)]
pub struct SchemaVariantNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "&[u8::from(self.is_locked)]")]
    is_locked: bool,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl SchemaVariantNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, is_locked: bool, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            is_locked,
            content_address: ContentAddress::SchemaVariant(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn set_is_locked(&mut self, new_locked: bool) {
        self.is_locked = new_locked;
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::SchemaVariant(new_content_hash);
    }

    pub(crate) async fn try_upgrade_from_content_node_weight(
        ctx: &DalContext,
        v3_graph: &mut WorkspaceSnapshotGraphV3,
        content_node_weight: &ContentNodeWeight,
    ) -> SchemaVariantNodeWeightResult<()> {
        let content_hash = if let ContentAddress::SchemaVariant(content_hash) =
            content_node_weight.content_address()
        {
            content_hash
        } else {
            return Err(Box::new(NodeWeightError::UnexpectedContentAddressVariant(
                ContentAddressDiscriminants::SchemaVariant,
                content_node_weight.content_address_discriminants(),
            ))
            .into());
        };

        let content: SchemaVariantContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or_else(|| {
                Box::new(NodeWeightError::MissingContentFromStore(
                    content_node_weight.id(),
                ))
            })?;

        let (v3_content, is_locked) = match content {
            SchemaVariantContent::V1(old_content) => {
                let v3_content = SchemaVariantContentV3 {
                    timestamp: old_content.timestamp,
                    ui_hidden: old_content.ui_hidden,
                    version: old_content.timestamp.created_at.to_string(),
                    display_name: old_content.display_name.unwrap_or_else(String::new),
                    category: old_content.category,
                    color: old_content.color,
                    component_type: old_content.component_type,
                    link: old_content.link,
                    description: old_content.description,
                    asset_func_id: old_content.asset_func_id,
                    finalized_once: old_content.finalized_once,
                    is_builtin: old_content.is_builtin,
                };

                // Locking variants didn't exist at this point, so everything should be considered
                // as locked.
                (v3_content, true)
            }
            SchemaVariantContent::V2(old_content) => {
                let v3_content = SchemaVariantContentV3 {
                    timestamp: old_content.timestamp,
                    ui_hidden: old_content.ui_hidden,
                    version: old_content.version,
                    display_name: old_content.display_name,
                    category: old_content.category,
                    color: old_content.color,
                    component_type: old_content.component_type,
                    link: old_content.link,
                    description: old_content.description,
                    asset_func_id: old_content.asset_func_id,
                    finalized_once: old_content.finalized_once,
                    is_builtin: old_content.is_builtin,
                };

                (v3_content, old_content.is_locked)
            }
            SchemaVariantContent::V3(_) => {
                return Err(SchemaVariantNodeWeightError::InvalidContentForNodeWeight(
                    content_node_weight.id(),
                ));
            }
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(SchemaVariantContent::V3(v3_content).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let new_node_weight_inner = Self::new(
            content_node_weight.id(),
            content_node_weight.lineage_id(),
            is_locked,
            hash,
        );
        let new_node_weight =
            NodeWeight::SchemaVariant(SchemaVariantNodeWeight::V1(new_node_weight_inner));

        v3_graph
            .add_or_replace_node(new_node_weight)
            .map_err(Box::new)?;

        Ok(())
    }
}

fn update_unlocks(
    updates: &[Update],
    update_idx: usize,
) -> crate::workspace_snapshot::node_weight::traits::CorrectTransformsResult<bool> {
    match updates.get(update_idx) {
        Some(Update::NewNode { node_weight }) | Some(Update::ReplaceNode { node_weight }) => {
            let schema_variant_node_weight = match node_weight.get_schema_variant_node_weight() {
                Ok(node_weight) => node_weight,
                Err(e) => return Err(e.into()),
            };
            Ok(
                !node_weight::traits::SiVersionedNodeWeight::inner(&schema_variant_node_weight)
                    .is_locked(),
            )
        }
        Some(_) => {
            // We're only adding NewNode & ReplaceNode updates to the lists in
            // variant_updates, so if we find anything else, we got hit by one hell of a
            // cosmic ray.
            unreachable!(
                "Got Update variant that should not exist in list of updates for this node weight"
            );
        }
        None => {
            unreachable!("Unable to retrieve previously found element in unmodified Vec")
        }
    }
}

impl CorrectTransforms for SchemaVariantNodeWeightV1 {
    fn correct_transforms(
        &self,
        workspace_snapshot_graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<crate::workspace_snapshot::graph::detector::Update>,
        _from_different_change_set: bool,
    ) -> crate::workspace_snapshot::node_weight::traits::CorrectTransformsResult<
        Vec<crate::workspace_snapshot::graph::detector::Update>,
    > {
        use crate::workspace_snapshot::graph::SchemaVariantExt;

        let mut maybe_my_schema_id =
            match workspace_snapshot_graph.schema_id_for_schema_variant_id(self.id.into()) {
                Err(WorkspaceSnapshotGraphError::SchemaVariant(schema_variant_error)) => {
                    match *schema_variant_error {
                        SchemaVariantError::NotFound(_) => {
                            // This is a brand-new SchemaVariant. The Schema may also be brand-new,
                            // which means we won't find it until we go through the updates, and find
                            // an Update::NewNode for it, and a corresponding Update::NewEdge that
                            // links us to the new node.
                            None
                        }
                        err => {
                            return Err(
                                WorkspaceSnapshotGraphError::SchemaVariant(Box::new(err)).into()
                            );
                        }
                    }
                }
                Ok(schema_id) => Some(schema_id),
                Err(err) => {
                    return Err(err.into());
                }
            };
        let mut variants_for_schema: HashMap<SchemaId, HashSet<SchemaVariantId>> = HashMap::new();
        let mut new_schemas = HashSet::new();
        let mut new_schema_variants = HashSet::new();
        let mut variant_updates: HashMap<SchemaVariantId, Vec<usize>> = HashMap::new();
        let mut new_variants_for_schema: HashMap<SchemaId, HashSet<SchemaVariantId>> =
            HashMap::new();
        let mut removed_variants_for_schema: HashMap<SchemaId, HashSet<SchemaVariantId>> =
            HashMap::new();

        for (idx, update) in updates.iter().enumerate() {
            match update {
                // If a SchemaVariant is being created, then there will also be an Update::NewEdge
                // from the Schema to the new SchemaVariant.
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    if source.node_weight_kind == NodeWeightDiscriminants::Content
                        && destination.node_weight_kind == NodeWeightDiscriminants::SchemaVariant
                        && EdgeWeightKindDiscriminants::from(edge_weight.kind())
                            == EdgeWeightKindDiscriminants::Use
                    {
                        let schema_id = si_events::ulid::Ulid::from(source.id).into();
                        let schema_variant_id = si_events::ulid::Ulid::from(destination.id).into();
                        variants_for_schema
                            .entry(schema_id)
                            .and_modify(|entry| {
                                entry.insert(schema_variant_id);
                            })
                            .or_insert_with(|| HashSet::from([schema_variant_id]));
                        new_variants_for_schema
                            .entry(schema_id)
                            .and_modify(|entry| {
                                entry.insert(schema_variant_id);
                            })
                            .or_insert_with(|| {
                                let mut new_schema_variant_ids = HashSet::new();
                                new_schema_variant_ids.insert(schema_variant_id);
                                new_schema_variant_ids
                            });

                        if schema_variant_id == self.id().into() {
                            maybe_my_schema_id = Some(schema_id);
                        }
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    if source.node_weight_kind == NodeWeightDiscriminants::Content
                        && destination.node_weight_kind == NodeWeightDiscriminants::SchemaVariant
                        && *edge_kind == EdgeWeightKindDiscriminants::Use
                    {
                        let schema_id = si_events::ulid::Ulid::from(source.id).into();
                        let schema_variant_id = si_events::ulid::Ulid::from(destination.id).into();
                        removed_variants_for_schema
                            .entry(schema_id)
                            .and_modify(|entry| {
                                entry.insert(schema_variant_id);
                            })
                            .or_insert_with(|| {
                                let mut removed_schema_variant_ids = HashSet::new();
                                removed_schema_variant_ids.insert(schema_variant_id);
                                removed_schema_variant_ids
                            });
                    }
                }
                Update::ReplaceNode { node_weight } => {
                    if NodeWeightDiscriminants::from(node_weight)
                        == NodeWeightDiscriminants::SchemaVariant
                    {
                        // If we're replacing the node, then it's supposed to already exist, which
                        // means that the schema should also already exist. If neither already
                        // exists, then it means they were deleted, and this update should be
                        // ignored as a no-op.
                        let schema_id = match workspace_snapshot_graph
                            .schema_id_for_schema_variant_id(node_weight.id().into())
                        {
                            Ok(schema_id) => schema_id,
                            Err(WorkspaceSnapshotGraphError::SchemaVariant(sv_error)) => {
                                match *sv_error {
                                    // Couldn't find the Schema/SchemaVariant; ignore it and keep
                                    // checking for relevant updates.
                                    SchemaVariantError::NotFound(_)
                                    | SchemaVariantError::SchemaNotFound(_) => continue,
                                    _ => {
                                        return Err(WorkspaceSnapshotGraphError::SchemaVariant(
                                            sv_error,
                                        )
                                        .into());
                                    }
                                }
                            }
                            Err(err) => {
                                return Err(err.into());
                            }
                        };
                        variants_for_schema
                            .entry(schema_id)
                            .and_modify(|entry| {
                                entry.insert(node_weight.id().into());
                            })
                            .or_insert_with(|| {
                                let mut new_variant_set = HashSet::new();
                                new_variant_set.insert(node_weight.id().into());
                                new_variant_set
                            });
                        variant_updates
                            .entry(node_weight.id().into())
                            .and_modify(|entry| {
                                entry.push(idx);
                            })
                            .or_insert_with(|| vec![idx]);
                    }
                }
                Update::NewNode { node_weight } => {
                    if node_weight.content_address_discriminants()
                        == Some(ContentAddressDiscriminants::Schema)
                    {
                        new_schemas.insert(idx);
                    } else if let Ok(schema_variant_node_weight) =
                        node_weight.get_schema_variant_node_weight()
                    {
                        new_schema_variants.insert(schema_variant_node_weight.id());
                        variant_updates
                            .entry(schema_variant_node_weight.id().into())
                            .and_modify(|entry| {
                                entry.push(idx);
                            })
                            .or_insert_with(|| vec![idx]);
                    }
                }
            }
        }

        let mut existing_unlocked_variants: HashSet<SchemaVariantId> = HashSet::new();
        let my_schema_id = if let Some(my_schema_id) = maybe_my_schema_id {
            use crate::workspace_snapshot::graph::SchemaVariantExt;

            let my_schema_variants = workspace_snapshot_graph
                .schema_variant_ids_for_schema_id_opt(my_schema_id)?
                .unwrap_or_default();
            variants_for_schema
                .entry(my_schema_id)
                .and_modify(|entry| {
                    entry.extend(my_schema_variants.iter().copied());
                })
                .or_insert_with(|| {
                    let new_variant_set: HashSet<SchemaVariantId> =
                        my_schema_variants.iter().copied().collect();
                    new_variant_set
                });
            for variant_id in my_schema_variants {
                let node_weight = workspace_snapshot_graph
                    .get_node_weight_by_id_opt(variant_id)
                    .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
                // If the existing unlocked variant is also going to be removed, then we don't
                // particularly care about it.
                if node_weight.id() != self.id()
                    && !node_weight::traits::SiVersionedNodeWeight::inner(
                        &node_weight.get_schema_variant_node_weight()?,
                    )
                    .is_locked()
                    && !removed_variants_for_schema
                        .get(&my_schema_id)
                        .map(|removed_sv_ids| removed_sv_ids.contains(&node_weight.id().into()))
                        .unwrap_or(false)
                {
                    existing_unlocked_variants.insert(node_weight.id().into());
                }
            }
            my_schema_id
        } else {
            // We didn't find the schema for ourself in either the existing graph, or in the set of
            // updates, which should mean that this is a creation/modification for a schema variant
            // of a schema that no longer exists in the graph. Since the Schema no longer exists,
            // it doesn't really matter what the creation/update looks like.
            return Ok(updates);
        };

        let my_last_update_idx = if let Some(update_idxs) = variant_updates
            .get(&self.id().into())
            .and_then(|idxs| idxs.last())
        {
            *update_idxs
        } else {
            // There were updates that involved us indirectly, but nothing that changed our
            // `is_locked` (either an Update::NewNode, or Update::ReplaceNode).
            return Ok(updates);
        };
        if !update_unlocks(&updates, my_last_update_idx)? {
            // Our updates result in a locked SchemaVariant, so we don't need to do anything to
            // make sure that unlocking us wouldn't cause multiple unlocked SchemaVariants for the
            // same Schema.
            return Ok(updates);
        }

        // If our updates result in being unlocked, we need to make sure there aren't any updates
        // that happen after that where they also result in an unlocked SchemaVariant. If there
        // are, then we want those updates to "win", which means we should be locked.
        let mut later_winning_variant_update = None;
        let mut earlier_unlock_variant_updates = Vec::new();

        #[inline(always)]
        // This is in its own method, because #[allow(clippy::unwrap_in_result)] was somehow
        // not being respected when used on an individual statement.
        fn get_variants_for_my_schema(
            variants_by_schema: &HashMap<SchemaId, HashSet<SchemaVariantId>>,
            schema_id: SchemaId,
        ) -> &HashSet<SchemaVariantId> {
            // If this expect triggers, we've been hit by cosmic rays, since we would have already
            // bailed out if we weren't able to determine which Schema this SchemaVariant belonged to
            // (and also included it in the list of SchemaVariants for that Schema).
            variants_by_schema.get(&schema_id).expect(
                "Somehow ended up not including ourself in the list of SchemaVariants for our Schema",
            )
        }
        for variant_id in get_variants_for_my_schema(&variants_for_schema, my_schema_id) {
            if self.id() == variant_id.into() {
                continue;
            }
            let last_update_idx = if let Some(update_idxs) =
                variant_updates.get(variant_id).and_then(|idxs| idxs.last())
            {
                *update_idxs
            } else {
                // There were no updates that directly updated this particular SchemaVariant, so it
                // can't be a later update that should "win".
                continue;
            };
            let last_update_unlocks = update_unlocks(&updates, last_update_idx)?;
            // If it's a pre-existing unlocked variant, but the updates end up with it as a locked
            // variant, we won't need to add an update that locks it.
            if existing_unlocked_variants.contains(variant_id) && !last_update_unlocks {
                existing_unlocked_variants.remove(variant_id);
            }
            // This update has to satisfy _ALL_ of the following to "win":
            //   * appear later in the list of updates than our last update does
            //   * be later in the list than any previously discovered "winning" update
            //   * unlock the SchemaVariant
            if last_update_idx > my_last_update_idx
                && Some(last_update_idx) > later_winning_variant_update
                && last_update_unlocks
            {
                later_winning_variant_update = Some(last_update_idx);
            }

            // We'll want to change this update to lock, instead of unlock if it:
            //   * appears _earlier_ in the list of updates than our last update does
            //   * unlocks the SchemaVariant
            if last_update_idx < my_last_update_idx && last_update_unlocks {
                earlier_unlock_variant_updates.push(last_update_idx);
            }
        }
        if later_winning_variant_update.is_some() {
            // Since there is a later update that "wins", we want to change our update to lock,
            // instead of unlock, and we can do that by adding our own update to the list of
            // "earlier" unlock updates to change to lock updates.
            earlier_unlock_variant_updates.push(my_last_update_idx);
        }
        // Even if there are later variant updates than ours that will "win", we can safely update
        // any variant updates that we would have been the winner for. We can't rely on earlier
        // updates having already been corrected by their own node weights as the order of the
        // list of nodes to interrogate for correcting transforms is not the same as the order they
        // appear in the list of updates.
        for earlier_unlock_variant_update_idx in earlier_unlock_variant_updates {
            #[inline(always)]
            // This is in its own method, because #[allow(clippy::unwrap_in_result)] was somehow
            // not being respected when used on an individual statement.
            fn get_last_update_from_updates(updates: &mut [Update], idx: usize) -> &mut Update {
                // If this expect triggers, we've beeen hit by cosmic rays, since we got this index
                // from enumerating the Vec in the first place.
                updates
                    .get_mut(idx)
                    .expect("Unable to retrieve previously found element in unmodified Vec")
            }
            let last_update =
                get_last_update_from_updates(&mut updates, earlier_unlock_variant_update_idx);
            match last_update {
                Update::NewNode { node_weight } | Update::ReplaceNode { node_weight } => {
                    let sv_node_weight = node_weight.get_schema_variant_node_weight_ref_mut()?;
                    node_weight::traits::SiVersionedNodeWeight::inner_mut(sv_node_weight)
                        .set_is_locked(false);
                }
                _ => unreachable!(
                    "Got Update variant that should not exist at this position in list of updates"
                ),
            }
        }
        // Any remaining pre-existing, unlocked SchemaVariant for this Schema need to have an
        // update added to lock them.
        for existing_unlocked_variant_id in existing_unlocked_variants {
            let mut node_weight = workspace_snapshot_graph
                .get_node_weight_by_id_opt(existing_unlocked_variant_id)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
                .clone();
            node_weight::traits::SiVersionedNodeWeight::inner_mut(
                node_weight.get_schema_variant_node_weight_ref_mut()?,
            )
            .set_is_locked(true);
            updates.push(Update::ReplaceNode { node_weight });
        }

        Ok(updates)
    }
}

impl CorrectExclusiveOutgoingEdge for SchemaVariantNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}
