use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};

use super::traits::CorrectTransformsResult;
use crate::{
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
    action::prototype::ActionKind,
    workspace_snapshot::{
        NodeInformation,
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::{
            NodeWeight,
            category_node_weight::CategoryNodeKind,
            traits::CorrectTransforms,
        },
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActionPrototypeNodeWeight {
    pub id: Ulid,
    kind: ActionKind,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    name: String,
    // TODO: Move behind ContentHash, and out of the node weight directly.
    description: Option<String>,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
}

impl ActionPrototypeNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        kind: ActionKind,
        name: impl AsRef<str>,
        description: Option<impl AsRef<str>>,
    ) -> Self {
        let name = name.as_ref().to_string();
        let description = description.map(|d| d.as_ref().to_string());

        Self {
            id,
            kind,
            name,
            description,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
        }
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn kind(&self) -> ActionKind {
        self.kind
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_description(&mut self, description: Option<impl Into<String>>) -> &mut Self {
        self.description = description.map(|d| d.into());
        self
    }

    pub fn set_kind(&mut self, kind: ActionKind) -> &mut Self {
        self.kind = kind;
        self
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(&self.id.inner().to_bytes());
        content_hasher.update(self.kind.to_string().as_bytes());
        content_hasher.update(self.name.as_bytes());
        content_hasher.update(
            &self
                .description
                .as_ref()
                .map(|d| d.as_bytes().to_owned())
                .unwrap_or_else(|| vec![0x00]),
        );

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[EdgeWeightKindDiscriminants::Use]
    }
}

impl CorrectTransforms for ActionPrototypeNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut new_overlay_schema_sources = vec![];
        let this_kind = self.kind();

        for update in &updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if destination.id.into_inner() == self.id().inner()
                        && EdgeWeightKindDiscriminants::ActionPrototype == edge_weight.kind().into()
                        // Schemas are content nodes
                        && source.node_weight_kind == NodeWeightDiscriminants::Content =>
                {
                    new_overlay_schema_sources.push(source.id);
                }
                _ => {}
            }
        }

        struct ConflictingPrototype<'a> {
            schema_source_weight: &'a NodeWeight,
            prototype_target_weight: &'a NodeWeight,
            prototype_target_idx: NodeIndex,
        }

        let mut conflicting_prototype_idxes = Vec::new();

        let overlay_category_node_information: Option<NodeInformation> = CategoryNodeKind::Overlays
            .static_id()
            .and_then(|overlay_id| graph.get_node_weight_by_id_opt(overlay_id).map(Into::into));

        for overlay_schema_source_id in new_overlay_schema_sources {
            let Some(schema_source_idx) = graph.get_node_index_by_id_opt(overlay_schema_source_id)
            else {
                continue;
            };
            let Some(schema_source_weight) = graph.get_node_weight_opt(schema_source_idx) else {
                continue;
            };

            graph
                .outgoing_edges(
                    schema_source_idx,
                    EdgeWeightKindDiscriminants::ActionPrototype,
                )
                .filter_map(|edge_ref| {
                    graph
                        .get_node_weight_opt(edge_ref.target())
                        .and_then(|weight| match weight {
                            NodeWeight::ActionPrototype(inner_weight)
                                if inner_weight.kind() == this_kind =>
                            {
                                Some((weight, edge_ref.target()))
                            }
                            _ => None,
                        })
                })
                .for_each(|(prototype_target_weight, prototype_target_idx)| {
                    conflicting_prototype_idxes.push(ConflictingPrototype {
                        schema_source_weight,
                        prototype_target_weight,
                        prototype_target_idx,
                    });
                });
        }

        // Remove the conflicting prototype by adding a remove edge update
        // between it and the schema source, and between it and the overlay
        // category. but also find any actions that are using the prototype,
        // and shift them over to this new prototype.
        for conflicting_prototype in conflicting_prototype_idxes {
            updates.push(Update::RemoveEdge {
                source: conflicting_prototype.schema_source_weight.into(),
                destination: conflicting_prototype.prototype_target_weight.into(),
                edge_kind: EdgeWeightKindDiscriminants::ActionPrototype,
            });

            // Remove overlay category edge
            if let Some(overlay_node_info) = overlay_category_node_information {
                updates.push(Update::RemoveEdge {
                    source: overlay_node_info,
                    destination: conflicting_prototype.prototype_target_weight.into(),
                    edge_kind: EdgeWeightKindDiscriminants::Use,
                });
            }

            let actions_using_prototype = graph
                .incoming_edges(
                    conflicting_prototype.prototype_target_idx,
                    EdgeWeightKindDiscriminants::Use,
                )
                .filter_map(|edge_ref| {
                    graph
                        .get_node_weight_opt(edge_ref.source())
                        .and_then(|weight| {
                            if let NodeWeight::Action(_) = weight {
                                Some(weight)
                            } else {
                                None
                            }
                        })
                });

            for action_node_weight in actions_using_prototype {
                // Remove edge from action to conflicting prototype
                updates.push(Update::RemoveEdge {
                    source: action_node_weight.into(),
                    destination: conflicting_prototype.prototype_target_weight.into(),
                    edge_kind: EdgeWeightKindDiscriminants::Use,
                });

                // Add edge from action to self
                updates.push(Update::NewEdge {
                    source: action_node_weight.into(),
                    destination: NodeInformation {
                        node_weight_kind: NodeWeightDiscriminants::ActionPrototype,
                        id: self.id().into(),
                    },
                    edge_weight: EdgeWeight::new(EdgeWeightKind::new_use()),
                });
            }
        }

        Ok(updates)
    }
}
