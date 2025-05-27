use std::collections::{
    HashMap,
    HashSet,
};

use itertools::Itertools as _;
use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    AttributePrototypeArgumentId,
    AttributeValueId,
    ComponentId,
    OutputSocketId,
    PropId,
    SchemaVariantId,
};

use crate::{
    prop::PropKind,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::{
            EdgeWeight,
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::{
            WorkspaceSnapshotGraph,
            WorkspaceSnapshotGraphResult,
        },
        node_weight::{
            ArgumentTargets,
            AttributePrototypeArgumentNodeWeight,
            NodeWeight,
        },
    },
};

pub fn validate_graph(
    graph: &WorkspaceSnapshotGraph,
) -> WorkspaceSnapshotGraphResult<Vec<ValidationIssue>> {
    let mut issues = vec![];
    for (node_weight, node_index) in graph.nodes() {
        issues.extend(validate_node(graph, node_weight, node_index)?);
    }
    Ok(issues)
}

pub fn validate_node(
    graph: &WorkspaceSnapshotGraph,
    node: &NodeWeight,
    node_index: NodeIndex,
) -> WorkspaceSnapshotGraphResult<Vec<ValidationIssue>> {
    let mut issues = vec![];
    match node {
        NodeWeight::AttributeValue(_) => {
            // If this is an object attribute value, check that it has child attribute values for each child prop
            let attr = node_index;
            let Some(prop) = graph.target_opt(attr, EdgeWeightKind::Prop)? else {
                return Ok(issues);
            };
            if PropKind::Object == graph.get_node_weight(prop)?.as_prop_node_weight()?.kind() {
                // Check our the children we *do* have against the child props we *should* have
                let child_props: HashSet<_> = graph
                    .targets(prop, EdgeWeightKindDiscriminants::Use)
                    .collect();

                // Step through child avs, and record the ones we see (maybe report duplicates)
                let mut attr_content = HashMap::new();
                for child_attr in graph.targets(attr, EdgeWeightKindDiscriminants::Contain) {
                    let child_attr_prop = graph.target(child_attr, EdgeWeightKind::Prop)?;
                    if !child_props.contains(&child_attr_prop) {
                        issues.push(ValidationIssue::UnknownChildAttributeValue {
                            child: graph.get_node_weight(child_attr)?.id().into(),
                        });
                        continue;
                    }
                    let content = graph
                        .get_node_weight(child_attr)?
                        .get_attribute_value_node_weight()?
                        .value();
                    if let Some(orig_content) = attr_content.insert(child_attr_prop, content) {
                        let original = graph.get_node_weight(child_attr)?.id().into();
                        let duplicate = graph.get_node_weight(child_attr)?.id().into();
                        if content == orig_content {
                            issues.push(ValidationIssue::DuplicateAttributeValue {
                                original,
                                duplicate,
                            })
                        } else {
                            issues.push(
                                ValidationIssue::DuplicateAttributeValueWithDifferentValues {
                                    original,
                                    duplicate,
                                },
                            )
                        };
                    }
                }

                // If any child attributes are *not* associated with child props, report those
                let mut missing_children = HashSet::new();
                for child_prop in child_props {
                    if !attr_content.contains_key(&child_prop) {
                        missing_children.insert(graph.get_node_weight(child_prop)?.id().into());
                    }
                }
                if !missing_children.is_empty() {
                    issues.push(ValidationIssue::MissingChildAttributeValues {
                        object: graph.get_node_weight(attr)?.id().into(),
                        missing_children,
                    });
                }
            }
        }
        NodeWeight::AttributePrototypeArgument(AttributePrototypeArgumentNodeWeight {
            targets:
                Some(ArgumentTargets {
                    source_component_id,
                    ..
                }),
            ..
        }) => {
            let destination_apa = node_index;
            let source_component = graph.get_node_index_by_id(source_component_id)?;

            // If this is a connection to a socket or prop, make sure the component on the other
            // end has an AV for it
            let Some(source_socket) =
                graph.target_opt(destination_apa, EdgeWeightKind::PrototypeArgumentValue)?
            else {
                return Ok(issues);
            };
            // Make sure the source is an output socket
            if Some(ContentAddressDiscriminants::OutputSocket)
                != graph
                    .get_node_weight(source_socket)?
                    .content_address_discriminants()
            {
                return Ok(issues);
            };
            // Run through the sockets on the component, and find the one that matches
            let mut component_sockets = graph
                .targets(source_component, EdgeWeightKind::SocketValue)
                .filter_map(|socket_value| graph.target(socket_value, EdgeWeightKind::Socket).ok());
            if !component_sockets.contains(&source_socket) {
                issues.push(ValidationIssue::ConnectionToUnknownSocket {
                    destination_apa: graph.get_node_weight(destination_apa)?.id().into(),
                    source_component: graph.get_node_weight(source_component)?.id().into(),
                    source_socket: graph.get_node_weight(source_socket)?.id().into(),
                })
            }
        }
        _ => {}
    }

    Ok(issues)
}

/// A single validation issue found in the graph
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ValidationIssue {
    /// APA is connected to a socket that does not exist on the source component
    ConnectionToUnknownSocket {
        destination_apa: AttributePrototypeArgumentId,
        source_component: ComponentId,
        source_socket: OutputSocketId,
    },
    /// A child prop of an object has more than one attribute value
    DuplicateAttributeValue {
        original: AttributeValueId,
        duplicate: AttributeValueId,
    },
    /// A child prop of an object has more than one attribute value, with different values
    DuplicateAttributeValueWithDifferentValues {
        original: AttributeValueId,
        duplicate: AttributeValueId,
    },
    /// One or more child props have no corresponding attribute values
    MissingChildAttributeValues {
        object: AttributeValueId,
        missing_children: HashSet<PropId>,
    },
    /// A child attribute value was found under an object, but it was not associated with any
    /// of the object's child props
    UnknownChildAttributeValue { child: AttributeValueId },
}

impl ValidationIssue {
    pub fn display(&self, graph: &WorkspaceSnapshotGraph) -> impl std::fmt::Display {
        DisplayWith(self, graph)
    }

    pub fn fix(&self, graph: &mut WorkspaceSnapshotGraph) -> WorkspaceSnapshotGraphResult<()> {
        match self {
            &ValidationIssue::DuplicateAttributeValue { duplicate, .. }
            | &ValidationIssue::DuplicateAttributeValueWithDifferentValues { duplicate, .. } => {
                println!("Removing duplicate attribute value {duplicate}");
                let node_index = graph.get_node_index_by_id(duplicate)?;
                graph.remove_node(node_index);
            }
            ValidationIssue::ConnectionToUnknownSocket {
                destination_apa, ..
            } => {
                println!("Removing APA {destination_apa}");
                // Remove the APA node (as long as it leads back to an input socket)
                let destination_apa = graph.get_node_index_by_id(destination_apa)?;
                let destination_prototype =
                    graph.source(destination_apa, EdgeWeightKind::PrototypeArgument)?;
                let destination_socket = graph.source(
                    destination_prototype,
                    EdgeWeightKindDiscriminants::Prototype,
                )?;
                let NodeWeight::InputSocket(_) = graph.get_node_weight(destination_socket)? else {
                    // If it's not an input socket, we can't be sure we're fixing what we want to fix
                    return Ok(());
                };
                graph.remove_node(destination_apa);
            }
            ValidationIssue::MissingChildAttributeValues { .. }
            | ValidationIssue::UnknownChildAttributeValue { .. } => {}
        }

        Ok(())
    }
}

/// Helper to format values that need extra context when being displayed (such as graphs, lookup tables,
/// etc).
struct DisplayWith<T, With>(T, With);

impl std::fmt::Display for DisplayWith<AttributeValueId, &'_ WorkspaceSnapshotGraph> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &DisplayWith(id, graph) = self;
        match av_path(id, graph) {
            Ok((_, path)) => write!(f, "{path} ({id})"),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for DisplayWith<PropId, &'_ WorkspaceSnapshotGraph> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &DisplayWith(id, graph) = self;
        match prop_path(id, graph) {
            Ok((_, path)) => write!(f, "{path} ({id})"),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for DisplayWith<&'_ ValidationIssue, &'_ WorkspaceSnapshotGraph> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &DisplayWith(issue, graph) = self;
        match *issue {
            ValidationIssue::ConnectionToUnknownSocket {
                destination_apa,
                source_component,
                source_socket,
            } => {
                write!(
                    f,
                    "Connection from APA {destination_apa} to unknown socket {source_socket} on component {source_component}"
                )
            }
            ValidationIssue::DuplicateAttributeValue {
                original,
                duplicate,
            } => {
                write!(
                    f,
                    "Duplicate attribute value: {} and {}",
                    DisplayWith(original, graph),
                    DisplayWith(duplicate, graph)
                )
            }
            ValidationIssue::DuplicateAttributeValueWithDifferentValues {
                original,
                duplicate,
            } => {
                write!(
                    f,
                    "Duplicate attribute value (with different value!): {} and {}",
                    DisplayWith(original, graph),
                    DisplayWith(duplicate, graph)
                )
            }
            ValidationIssue::MissingChildAttributeValues {
                object,
                ref missing_children,
            } => {
                write!(
                    f,
                    "Missing child attribute values for object {}: missing {}",
                    DisplayWith(object, graph),
                    missing_children
                        .iter()
                        .map(|&child_prop| format!("{}", DisplayWith(child_prop, graph)))
                        .join(", ")
                )
            }
            ValidationIssue::UnknownChildAttributeValue { child } => {
                write!(
                    f,
                    "Child attribute value has unknown (non-child) prop: {}",
                    DisplayWith(child, graph)
                )
            }
        }
    }
}

fn av_path_from_root(
    id: AttributeValueId,
    graph: &WorkspaceSnapshotGraph,
) -> WorkspaceSnapshotGraphResult<(AttributeValueId, jsonptr::PointerBuf)> {
    let mut index = graph.get_node_index_by_id(id)?;
    let mut path = jsonptr::PointerBuf::new();
    while let Some((
        EdgeWeight {
            kind: EdgeWeightKind::Contain(key),
        },
        parent_index,
        _,
    )) = graph
        .edges_directed_for_edge_weight_kind(
            index,
            Direction::Incoming,
            EdgeWeightKindDiscriminants::Contain,
        )
        .next()
    {
        // If we have a key, use it (must be a map)
        let parent_prop = graph
            .get_node_weight(graph.target(parent_index, EdgeWeightKind::Prop)?)?
            .get_prop_node_weight()?;
        match (parent_prop.kind(), key) {
            // Use the key if the parent is a map
            (PropKind::Map, Some(key)) => path.push_front(key),
            // Use the prop name if the parent is an object
            (PropKind::Object, None) => {
                let prop = graph
                    .get_node_weight(graph.target(index, EdgeWeightKind::Prop)?)?
                    .get_prop_node_weight()?;
                path.push_front(prop.name());
            }
            // Use the array index if the parent is an array
            (PropKind::Array, None) => match graph
                .ordered_children_for_node(parent_index)?
                .and_then(|order| order.iter().position(|&child| child == index))
            {
                Some(index) => path.push_front(index),
                // TODO return an error here instead
                None => path.push_front("<NOT IN ORDERING NODE>"),
            },

            // These are errors (and if we move this out of printing code, we should return an error)
            (PropKind::Array | PropKind::Object, Some(_)) => {
                path.push_front("<MAP KEY SPECIFIED FOR ARRAY/OBJECT>")
            }
            (PropKind::Map, None) => path.push_front("<NO MAP KEY>"),
            (kind, _) => path.push_front(format!("<BAD PARENT KIND {}>", kind)),
        }
        index = parent_index;
    }

    let root_id = graph.get_node_weight(index)?.id().into();

    Ok((root_id, path))
}

fn av_path(
    id: AttributeValueId,
    graph: &WorkspaceSnapshotGraph,
) -> WorkspaceSnapshotGraphResult<(ComponentId, jsonptr::PointerBuf)> {
    // index is now either the root of the graph, or a socket node. Just grab its prop or socket name
    let (root_id, mut path) = av_path_from_root(id, graph)?;
    let root_index = graph.get_node_index_by_id(root_id)?;
    let component_index = match graph.source_opt(root_index, EdgeWeightKind::Root)? {
        Some(component_index) => {
            path.push_front("root");
            component_index
        }
        None => {
            let socket_index = graph.target(root_index, EdgeWeightKind::Socket)?;
            let socket_id = graph.get_node_weight(socket_index)?.id();
            // TODO get name (which we don't presently have, because it's in content)
            path.push_front(format!("{}", socket_id));
            match graph.get_node_weight(socket_index)? {
                NodeWeight::InputSocket(_) => path.push_front("inputSockets"),
                _ => path.push_front("outputSockets"),
            }
            graph.source(root_index, EdgeWeightKind::SocketValue)?
        }
    };
    let component_id = graph.get_node_weight(component_index)?.id().into();
    Ok((component_id, path))
}

fn prop_path(
    id: PropId,
    graph: &WorkspaceSnapshotGraph,
) -> WorkspaceSnapshotGraphResult<(SchemaVariantId, jsonptr::PointerBuf)> {
    let mut index = graph.get_node_index_by_id(id)?;
    let mut path = jsonptr::PointerBuf::new();
    while let NodeWeight::Prop(prop_node) = graph.get_node_weight(index)? {
        path.push_front(prop_node.name());
        index = graph.source(index, EdgeWeightKindDiscriminants::Use)?;
    }
    let schema_variant_id = graph.get_node_weight(index)?.id().into();
    Ok((schema_variant_id, path))
}
