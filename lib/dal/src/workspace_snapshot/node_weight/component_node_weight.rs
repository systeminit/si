use super::{
    category_node_weight::CategoryNodeKind,
    traits::CorrectTransformsResult,
    NodeWeight,
    NodeWeightDiscriminants::{self, Component},
    NodeWeightError, NodeWeightResult,
};
use crate::layer_db_types::{
    ComponentContent, ComponentContentDiscriminants, ComponentContentV2, GeometryContent,
    GeometryContentV1,
};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphV4;
use crate::{
    workspace_snapshot::{
        content_address::{ContentAddress, ContentAddressDiscriminants},
        graph::{
            correct_transforms::add_dependent_value_root_updates,
            deprecated::v1::DeprecatedComponentNodeWeightV1, detector::Update, LineageId,
        },
        node_weight::traits::CorrectTransforms,
        NodeInformation,
    },
    DalContext, EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants, Timestamp,
    WorkspaceSnapshotGraphVCurrent,
};
use petgraph::{prelude::*, visit::EdgeRef, Direction::Incoming};
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComponentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    to_delete: bool,
}

impl ComponentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_address: ContentAddress) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            to_delete: false,
        }
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn set_to_delete(&mut self, to_delete: bool) -> &mut Self {
        self.to_delete = to_delete;
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Component.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.content_address.content_hash().as_bytes());
        content_hasher.update(&[u8::from(self.to_delete)]);

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn overwrite_id(&mut self, id: Ulid) {
        self.id = id
    }

    pub fn overwrite_lineage_id(&mut self, id: LineageId) {
        self.lineage_id = id
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::Root,
        ]
    }

    pub(crate) async fn try_upgrade_and_create_external_geometry(
        ctx: &DalContext,
        v4_graph: &mut WorkspaceSnapshotGraphV4,
        default_view_idx: NodeIndex,
        component_node_weight: &ComponentNodeWeight,
    ) -> NodeWeightResult<()> {
        let component_idx = v4_graph
            .get_node_index_by_id(component_node_weight.id)
            .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?;

        let layer_db = ctx.layer_db();
        let cas = layer_db.cas();

        let component_content: ComponentContent = cas
            .try_read_as(&component_node_weight.content_hash())
            .await?
            .ok_or_else(|| NodeWeightError::MissingContentFromStore(component_node_weight.id))?;

        // When migrating from graph v3 to v4 all components should be on v1
        let ComponentContent::V1(content) = component_content.clone() else {
            let actual = ComponentContentDiscriminants::from(component_content);
            return Err(NodeWeightError::UnexpectedComponentContentVersion(
                actual,
                ComponentContentDiscriminants::V1,
            ));
        };

        // Create geometry node
        let geometry_content = GeometryContent::V1(GeometryContentV1 {
            timestamp: Timestamp::now(),
            x: content.x,
            y: content.y,
            width: content.width,
            height: content.height,
        });

        let (content_address, _) = cas.write(
            Arc::new(geometry_content.clone().into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let geometry_id = v4_graph
            .generate_ulid()
            .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?;

        let geometry_node_weight = NodeWeight::new_geometry(
            geometry_id,
            v4_graph
                .generate_ulid()
                .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?,
            content_address,
        );

        let geometry_idx = v4_graph
            .add_or_replace_node(geometry_node_weight)
            .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?;

        // Connect geometry to component
        v4_graph
            .add_edge(
                geometry_idx,
                EdgeWeight::new(EdgeWeightKind::Represents),
                component_idx,
            )
            .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?;

        // Connect geometry to default view
        v4_graph
            .add_edge(
                default_view_idx,
                EdgeWeight::new(EdgeWeightKind::new_use()),
                geometry_idx,
            )
            .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?;

        // Upgrade component content
        let updated_content = ComponentContent::V2(ComponentContentV2 {
            timestamp: content.timestamp,
        });

        let (updated_content_address, _) = cas.write(
            Arc::new(updated_content.into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let upgraded_node_weight = NodeWeight::new_component(
            component_node_weight.id,
            component_node_weight.lineage_id,
            updated_content_address,
        );

        v4_graph
            .add_or_replace_node(upgraded_node_weight)
            .map_err(|e| NodeWeightError::WorkspaceSnapshotGraph(Box::new(e)))?;

        Ok(())
    }
}

impl From<DeprecatedComponentNodeWeightV1> for ComponentNodeWeight {
    fn from(value: DeprecatedComponentNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            to_delete: value.to_delete,
        }
    }
}

impl From<&ComponentNodeWeight> for NodeInformation {
    fn from(value: &ComponentNodeWeight) -> Self {
        Self {
            node_weight_kind: Component,
            id: value.id.into(),
        }
    }
}

fn remove_hanging_socket_connections(
    graph: &WorkspaceSnapshotGraphVCurrent,
    component_id: Ulid,
    component_idx: NodeIndex,
) -> CorrectTransformsResult<Vec<Update>> {
    let mut new_updates = vec![];

    // To find the attribute prototype arguments that need to be removed, we
    // have to find the OutputSockets for this component. Once we find them, we
    // need to find the incoming PrototypeArgumentValue edge from the
    // AttributePrototypeArgument to that socket. Then we have to verify that
    // the argument has our component as a source. Then we can issue RemoveEdge
    // updates for all incoming edges to that attribute prototype argument. With
    // no incoming edges, the APA will be removed from the graph.

    let mut affected_attribute_values = HashSet::new();

    for socket_value_target in graph
        .edges_directed(component_idx, Outgoing)
        .filter(|edge_ref| {
            EdgeWeightKindDiscriminants::SocketValue == edge_ref.weight().kind().into()
        })
        .map(|edge_ref| edge_ref.target())
    {
        for output_socket_index in graph
            .edges_directed(socket_value_target, Outgoing)
            .filter(|edge_ref| {
                EdgeWeightKindDiscriminants::Socket == edge_ref.weight().kind().into()
            })
            .filter(|edge_ref| {
                graph
                    .get_node_weight_opt(edge_ref.target())
                    .is_some_and(|weight| match weight {
                        NodeWeight::Content(inner) => {
                            inner.content_address_discriminants()
                                == ContentAddressDiscriminants::OutputSocket
                        }
                        _ => false,
                    })
            })
            .map(|edge_ref| edge_ref.target())
        {
            for (apa_idx, apa_weight) in graph
                .edges_directed(output_socket_index, Incoming)
                .filter(|edge_ref| {
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue
                        == edge_ref.weight().kind().into()
                })
                .filter_map(|edge_ref| {
                    graph
                        .get_node_weight_opt(edge_ref.source())
                        .and_then(|node_weight| match node_weight {
                            NodeWeight::AttributePrototypeArgument(inner) => {
                                inner.targets().and_then(|targets| {
                                    if targets.source_component_id == component_id.into() {
                                        Some((edge_ref.source(), node_weight))
                                    } else {
                                        None
                                    }
                                })
                            }
                            _ => None,
                        })
                })
            {
                for edge_ref in graph.edges_directed(apa_idx, Incoming) {
                    if let Some(source_weight) = graph.get_node_weight_opt(edge_ref.source()) {
                        new_updates.push(Update::RemoveEdge {
                            source: source_weight.into(),
                            destination: apa_weight.into(),
                            edge_kind: edge_ref.weight().kind().into(),
                        })
                    }

                    // Walk to the attribute value for this socket so we can add it to the DVUs
                    graph
                        .edges_directed(edge_ref.source(), Incoming)
                        .for_each(|edge_ref| {
                            graph.edges_directed(edge_ref.source(), Incoming).for_each(
                                |edge_ref| {
                                    if let Some(id) = graph
                                        .get_node_weight_opt(edge_ref.source())
                                        .and_then(|node_weight| match node_weight {
                                            NodeWeight::AttributeValue(_) => Some(node_weight.id()),
                                            _ => None,
                                        })
                                    {
                                        affected_attribute_values.insert(id);
                                    }
                                },
                            );
                        });
                }
            }
        }
    }

    // The input sockets that have had connections removed need to be recalculated now
    new_updates.extend(add_dependent_value_root_updates(
        graph,
        &affected_attribute_values,
    )?);

    Ok(new_updates)
}

impl CorrectTransforms for ComponentNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut valid_frame_contains_source = None;
        let mut existing_remove_edges = vec![];
        let mut updates_to_remove = vec![];
        let mut component_will_be_deleted = false;

        for (i, update) in updates.iter().enumerate() {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if destination.id.into_inner() == self.id.inner() => {
                    match edge_weight.kind().into() {
                        EdgeWeightKindDiscriminants::FrameContains => {
                            // If we get more than one frame contains edge in the set of
                            // updates we will pick the last one. Although there should
                            // never be more than one in a single batch, this makes it
                            // resilient against replaying multiple transform batches
                            // (in order). Last one wins!
                            valid_frame_contains_source = match valid_frame_contains_source {
                                None => Some((i, source.id)),
                                Some((last_index, _)) => {
                                    updates_to_remove.push(last_index);
                                    Some((i, source.id))
                                }
                            }
                        }
                        EdgeWeightKindDiscriminants::Use => {
                            let component_will_be_added = graph
                                .get_node_weight_by_id_opt(source.id)
                                .is_some_and(|node_weight| {
                                    if let NodeWeight::Category(inner) = node_weight {
                                        inner.kind() == CategoryNodeKind::Component
                                    } else {
                                        false
                                    }
                                });
                            if component_will_be_added {
                                component_will_be_deleted = false;
                            }
                        }
                        _ => {}
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if destination.id.into_inner() == self.id.inner() => match edge_kind {
                    EdgeWeightKindDiscriminants::FrameContains => {
                        if let Some(source_index) =
                            graph.get_node_index_by_id_opt(source.id.into_inner())
                        {
                            existing_remove_edges.push(source_index);
                        }
                    }
                    EdgeWeightKindDiscriminants::Use
                        if source.node_weight_kind == NodeWeightDiscriminants::Category =>
                    {
                        component_will_be_deleted = graph
                            .get_node_weight_by_id_opt(source.id)
                            .is_some_and(|node_weight| {
                                if let NodeWeight::Category(inner) = node_weight {
                                    inner.kind() == CategoryNodeKind::Component
                                } else {
                                    false
                                }
                            })
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if component_will_be_deleted {
            if let Some(component_idx) = graph.get_node_index_by_id_opt(self.id) {
                updates.extend(remove_hanging_socket_connections(
                    graph,
                    self.id,
                    component_idx,
                )?);

                // Also remove any incoming edges to the component in case there
                // is a frame contains in another change set
                updates.extend(graph.edges_directed(component_idx, Incoming).filter_map(
                    |edge_ref| {
                        graph
                            .get_node_weight_opt(edge_ref.source())
                            .map(|source_weight| Update::RemoveEdge {
                                source: source_weight.into(),
                                destination: self.into(),
                                edge_kind: edge_ref.weight().kind().into(),
                            })
                    },
                ));
            }
        } else {
            if !updates_to_remove.is_empty() {
                let mut idx = 0;
                // Vec::remove is O(n) for the updates, which will likely always be
                // > than the size of updates_to_remove
                updates.retain(|_| {
                    let should_retain = !updates_to_remove.contains(&idx);
                    idx += 1;
                    should_retain
                })
            }

            // Add updates to remove any incoming FrameContains edges that don't
            // have the source in valid_frame_contains_source. This ensures a
            // component can only have one frame parent
            if let Some((_, valid_frame_contains_source)) = valid_frame_contains_source {
                if let (Some(valid_source), Some(self_index)) = (
                    graph.get_node_index_by_id_opt(valid_frame_contains_source),
                    graph.get_node_index_by_id_opt(self.id),
                ) {
                    updates.extend(
                        graph
                            .edges_directed(self_index, Incoming)
                            // We only want to find incoming FrameContains edges
                            // that  are not from the current valid source
                            .filter(|edge_ref| {
                                EdgeWeightKindDiscriminants::FrameContains
                                    == edge_ref.weight().kind().into()
                                    && edge_ref.source() != valid_source
                                    && !existing_remove_edges.contains(&edge_ref.source())
                            })
                            .filter_map(|edge_ref| {
                                graph
                                    .get_node_weight_opt(edge_ref.source())
                                    .map(|source_weight| Update::RemoveEdge {
                                        source: source_weight.into(),
                                        destination: self.into(),
                                        edge_kind: EdgeWeightKindDiscriminants::FrameContains,
                                    })
                            }),
                    );
                }
            }
        }

        Ok(updates)
    }
}
