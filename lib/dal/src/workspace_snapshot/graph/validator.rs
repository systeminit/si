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
    FuncId,
    InputSocketId,
    OutputSocketId,
    PropId,
    SchemaVariantId,
};

use super::WorkspaceSnapshotGraphError;
use crate::{
    func::FuncKind,
    prop::PropKind,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::{
            EdgeWeight,
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::{
            WorkspaceSnapshotGraphResult,
            WorkspaceSnapshotGraphVCurrent,
        },
        node_weight::{
            ArgumentTargets,
            AttributePrototypeArgumentNodeWeight,
            NodeWeight,
            traits::SiNodeWeight as _,
        },
    },
};

pub mod connections;

/// Validate the entire graph, returning a list of validation issues found.
pub fn validate_graph_with_text(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
) -> WorkspaceSnapshotGraphResult<Vec<(ValidationIssue, String)>> {
    Ok(validate_graph(graph)?
        .into_iter()
        .map(|issue| {
            let text = format!("{}", WithGraph(graph.deref(), &issue));
            (issue, text)
        })
        .collect())
}

/// Validate the entire graph, returning a list of validation issues found.
pub fn validate_graph(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
) -> WorkspaceSnapshotGraphResult<Vec<ValidationIssue>> {
    let mut issues = vec![];
    for (node_weight, node_index) in graph.nodes() {
        issues.extend(validate_node(graph, node_weight, node_index)?);
    }
    Ok(issues)
}

/// Validate a single node in the graph, returning a list of validation issues found.
pub fn validate_node(
    graph: &WorkspaceSnapshotGraphVCurrent,
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
            let dest_apa = node_index;
            let source_component = graph.get_node_index_by_id(source_component_id)?;

            // If this is a connection to a socket or prop, make sure the component on the other
            // end has an AV for it
            let Some(source_socket) =
                graph.target_opt(dest_apa, EdgeWeightKind::PrototypeArgumentValue)?
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
                    dest_apa: graph.get_node_weight(dest_apa)?.id().into(),
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
        dest_apa: AttributePrototypeArgumentId,
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
    pub fn fix(
        &self,
        graph: &mut WorkspaceSnapshotGraphVCurrent,
    ) -> WorkspaceSnapshotGraphResult<()> {
        match self {
            &ValidationIssue::DuplicateAttributeValue { duplicate, .. }
            | &ValidationIssue::DuplicateAttributeValueWithDifferentValues { duplicate, .. } => {
                println!("Removing duplicate attribute value {duplicate}");
                let node_index = graph.get_node_index_by_id(duplicate)?;
                graph.remove_node(node_index);
            }
            ValidationIssue::ConnectionToUnknownSocket { dest_apa, .. } => {
                println!("Removing APA {dest_apa}");
                // Remove the APA node (as long as it leads back to an input socket)
                let dest_apa = graph.get_node_index_by_id(dest_apa)?;
                let dest_prototype = graph.source(dest_apa, EdgeWeightKind::PrototypeArgument)?;
                let dest_socket =
                    graph.source(dest_prototype, EdgeWeightKindDiscriminants::Prototype)?;
                let NodeWeight::InputSocket(_) = graph.get_node_weight(dest_socket)? else {
                    // If it's not an input socket, we can't be sure we're fixing what we want to fix
                    return Ok(());
                };
                graph.remove_node(dest_apa);
            }
            ValidationIssue::MissingChildAttributeValues { .. }
            | ValidationIssue::UnknownChildAttributeValue { .. } => {}
        }

        Ok(())
    }
}

/// Helper to format values that need extra context when being displayed (such as graphs, lookup tables,
/// etc).
pub struct WithGraph<'a, T>(pub &'a WorkspaceSnapshotGraphVCurrent, pub T);

impl std::fmt::Display for WithGraph<'_, AttributeValueId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match av_path_from_root(graph, id) {
            Ok((_, path)) => write!(f, "{path} ({id})"),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, ComponentId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, FuncId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match graph
            .get_node_weight_by_id(id)
            .and_then(|node| node.get_func_node_weight().map_err(Into::into))
        {
            Ok(node) => write!(f, "{} ({id})", node.name()),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, PropId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match prop_path_from_root(graph, id) {
            Ok((_, path)) => write!(f, "{path} ({id})"),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, InputSocketId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, OutputSocketId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ ValidationIssue> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, issue) = self;
        match *issue {
            ValidationIssue::ConnectionToUnknownSocket {
                dest_apa,
                source_component,
                source_socket,
            } => {
                write!(
                    f,
                    "Connection from APA {dest_apa} to unknown socket {source_socket} on component {source_component}"
                )
            }
            ValidationIssue::DuplicateAttributeValue {
                original,
                duplicate,
            } => {
                write!(
                    f,
                    "Duplicate attribute value: {} and {}",
                    WithGraph(graph, original),
                    WithGraph(graph, duplicate)
                )
            }
            ValidationIssue::DuplicateAttributeValueWithDifferentValues {
                original,
                duplicate,
            } => {
                write!(
                    f,
                    "Duplicate attribute value (with different value!): {} and {}",
                    WithGraph(graph, original),
                    WithGraph(graph, duplicate)
                )
            }
            ValidationIssue::MissingChildAttributeValues {
                object,
                ref missing_children,
            } => {
                write!(
                    f,
                    "Missing child attribute values for object {}: missing {}",
                    WithGraph(graph, object),
                    missing_children
                        .iter()
                        .map(|&child_prop| format!("{}", WithGraph(graph, child_prop)))
                        .join(", ")
                )
            }
            ValidationIssue::UnknownChildAttributeValue { child } => {
                write!(
                    f,
                    "Child attribute value has unknown (non-child) prop: {}",
                    WithGraph(graph, child)
                )
            }
        }
    }
}

pub fn is_identity_func(
    graph: &WorkspaceSnapshotGraphVCurrent,
    func_id: FuncId,
) -> WorkspaceSnapshotGraphResult<bool> {
    let func_node = graph
        .get_node_weight_by_id(func_id)?
        .get_func_node_weight()?;
    Ok(func_node.func_kind() == FuncKind::Intrinsic && func_node.name() == "si:identity")
}

pub fn is_normalize_to_array_func(
    graph: &WorkspaceSnapshotGraphVCurrent,
    func_id: FuncId,
) -> WorkspaceSnapshotGraphResult<bool> {
    let func_node = graph
        .get_node_weight_by_id(func_id)?
        .get_func_node_weight()?;
    Ok(func_node.func_kind() == FuncKind::Intrinsic && func_node.name() == "si:normalizeToArray")
}

pub fn resolve_av(
    graph: &WorkspaceSnapshotGraphVCurrent,
    component_id: ComponentId,
    path: impl AsRef<jsonptr::Pointer>,
) -> WorkspaceSnapshotGraphResult<Option<AttributeValueId>> {
    let component = graph.get_node_index_by_id(component_id)?;
    let mut av = graph.target(component, EdgeWeightKind::Root)?;
    for segment in path.as_ref() {
        let prop = graph.target(av, EdgeWeightKind::Prop)?;
        let child_av =
            match graph.get_node_weight(prop)?.get_prop_node_weight()?.kind() {
                PropKind::Object => graph
                    .targets(av, EdgeWeightKindDiscriminants::Contain)
                    .find(|&child_av| {
                        graph
                            .target(child_av, EdgeWeightKind::Prop)
                            .and_then(|child_prop| graph.get_node_weight(child_prop))
                            .and_then(|child_prop| {
                                child_prop.get_prop_node_weight().map_err(Into::into)
                            })
                            .is_ok_and(|child_prop| child_prop.name() == segment.decoded())
                    }),
                PropKind::Map => graph
                    .edges_directed(av, Direction::Outgoing)
                    .find_map(|edge| match edge.weight().kind() {
                        EdgeWeightKind::Contain(Some(key)) if key.as_str() == segment.decoded() => {
                            Some(edge.target())
                        }
                        _ => None,
                    }),
                PropKind::Array => graph.ordered_children_for_node(av)?.and_then(|children| {
                    match segment.to_index() {
                        Ok(jsonptr::index::Index::Num(index)) => children.get(index).copied(),
                        _ => None,
                    }
                }),
                PropKind::Boolean
                | PropKind::Float
                | PropKind::Integer
                | PropKind::Json
                | PropKind::String => None,
            };
        let Some(child_av) = child_av else {
            return Ok(None);
        };
        av = child_av;
    }

    Ok(Some(
        graph
            .get_node_weight(av)?
            .get_attribute_value_node_weight()?
            .id()
            .into(),
    ))
}

pub fn av_path_from_root(
    graph: &WorkspaceSnapshotGraphVCurrent,
    id: AttributeValueId,
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

pub fn prop_path_from_root(
    graph: &WorkspaceSnapshotGraphVCurrent,
    id: PropId,
) -> WorkspaceSnapshotGraphResult<(PropId, jsonptr::PointerBuf)> {
    let mut path = jsonptr::PointerBuf::new();

    let mut prop = graph.get_node_index_by_id(id)?;
    let mut prop_node = graph.get_node_weight(prop)?.get_prop_node_weight()?;
    loop {
        // If the parent is a prop, push the current node onto the path and ascend to the parent
        let parent = graph.source(prop, EdgeWeightKindDiscriminants::Use)?;
        let NodeWeight::Prop(parent_prop_node) = graph.get_node_weight(parent)? else {
            break;
        };
        path.push_front(prop_node.name());
        prop = parent;
        prop_node = parent_prop_node.clone();
    }

    let prop_id = prop_node.id().into();
    Ok((prop_id, path))
}

pub fn prop_path(
    graph: &WorkspaceSnapshotGraphVCurrent,
    id: PropId,
) -> WorkspaceSnapshotGraphResult<(SchemaVariantId, jsonptr::PointerBuf)> {
    // Get the path up to (but not including) the root prop
    let (root_prop_id, mut path) = prop_path_from_root(graph, id)?;

    // Push the root node onto the path
    let root_prop = graph.get_node_index_by_id(root_prop_id)?;
    let root_prop_node = graph.get_node_weight(root_prop)?.get_prop_node_weight()?;
    path.push_front(root_prop_node.name());

    // Get the schema variant (the parent of the root prop)
    let schema_variant = graph.source(root_prop, EdgeWeightKindDiscriminants::Use)?;
    let schema_variant_id = graph
        .get_node_weight(schema_variant)?
        .get_schema_variant_node_weight()?
        .id()
        .into();
    Ok((schema_variant_id, path))
}
