use std::{
    collections::{
        HashMap,
        HashSet,
    },
    env,
    fs::File,
    io::{
        Read as _,
        Write,
    },
};

use dal::{
    AttributeValueId,
    ComponentId,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    OutputSocketId,
    PropId,
    PropKind,
    WorkspaceSnapshotGraph,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{
            ArgumentTargets,
            AttributePrototypeArgumentNodeWeight,
            NodeWeight,
        },
    },
};
use itertools::Itertools;
use petgraph::prelude::*;
use si_id::AttributePrototypeArgumentId;
use si_layer_cache::db::serialize;
use tokio::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // To fix a snapshot, first download the snapshot from the admin portal
    // Then run this like so in dal/examples:
    // cargo run --example snapshot-fixer <PATH>
    // ex: cargo run --example snapshot-fixer ~/Downloads/head.snapshot
    let mut args = env::args();
    let snap_path = args.nth(1).expect("usage: program <SNAPSHOT_FILE_PATH>");

    let mut snap_file = File::open(snap_path)?;

    let mut snap_bytes = vec![];
    snap_file.read_to_end(&mut snap_bytes)?;

    println!("snap bytes compressed: {}", snap_bytes.len());
    let decompressed = serialize::decompress_to_vec(&snap_bytes)?;
    println!("decompressed: {}", decompressed.len());

    let now = Instant::now();
    let mut graph: WorkspaceSnapshotGraph = serialize::from_bytes(&snap_bytes)?;
    println!("deserialization took: {:?}", now.elapsed());
    // Make your edits:
    // Example: We were seeing:
    // attribute value error: attribute value 01JTXGMYKFFPY7H2ZNV7SKFQ9X has no outgoing edge to a prop or socket"
    // so we needed to remove it
    // let node_id = "01JTXGMYKFFPY7H2ZNV7SKFQ9X";
    // remove_node_by_id(&mut graph, node_id)?;

    for issue in validate_graph(&graph)? {
        println!("{}", issue.report(&graph)?);
        // Only fix ConnectionToUnknownSocket issues for now
        if let issue @ ValidationIssue::ConnectionToUnknownSocket { .. } = issue {
            issue.fix(&mut graph)?
        }
    }

    // Cleanup and update merkle tree
    graph.cleanup_and_merkle_tree_hash()?;

    // write snapshot
    write_snapshot_graph("./fixed.snapshot", &graph)?;

    // then head back to the admin portal and replace snapshot with this new fixed snapshot
    Ok(())
}

fn validate_graph(graph: &WorkspaceSnapshotGraph) -> Result<Vec<ValidationIssue>> {
    Ok(graph
        .nodes()
        .flat_map(|(node, node_index)| validate_node(graph, node, node_index).unwrap())
        .collect())
}

enum ValidationIssue {
    /// A child prop of an object has more than one attribute value
    DuplicateAttributeValue {
        original: AttributeValueId,
        duplicate: AttributeValueId,
        content_matches: bool,
    },
    /// One or more child props have no corresponding attribute values
    MissingChildAttributeValues {
        object: AttributeValueId,
        missing_children: HashSet<PropId>,
    },
    /// A child attribute value was found under an object, but it was not associated with any
    /// of the object's child props
    UnknownChildAttributeValue { child: AttributeValueId },
    /// APA is connected to a socket that does not exist on the source component
    ConnectionToUnknownSocket {
        destination_apa: AttributePrototypeArgumentId,
        source_component: ComponentId,
        source_socket: OutputSocketId,
    },
}

impl ValidationIssue {
    fn report(&self, graph: &WorkspaceSnapshotGraph) -> Result<String> {
        Ok(match self {
            &ValidationIssue::DuplicateAttributeValue {
                original,
                duplicate,
                content_matches,
            } => format!(
                "Duplicate attribute value: {} ({}) and {} ({}){}",
                graph
                    .get_node_weight(
                        graph
                            .target(graph.get_node_index_by_id(original)?, EdgeWeightKind::Prop)?
                    )?
                    .as_prop_node_weight()?
                    .name(),
                original,
                graph
                    .get_node_weight(
                        graph
                            .target(graph.get_node_index_by_id(duplicate)?, EdgeWeightKind::Prop)?
                    )?
                    .as_prop_node_weight()?
                    .name(),
                duplicate,
                match content_matches {
                    true => "",
                    false => " (CONTENT MISMATCH)",
                }
            ),
            ValidationIssue::MissingChildAttributeValues {
                object,
                missing_children,
            } => format!(
                "Missing child attribute values for object {} ({}): missing {}",
                graph
                    .get_node_weight(
                        graph.target(graph.get_node_index_by_id(object)?, EdgeWeightKind::Prop)?
                    )?
                    .as_prop_node_weight()?
                    .name(),
                object,
                missing_children
                    .iter()
                    .map(|&child_prop| format!(
                        "{} ({})",
                        graph
                            .get_node_weight_by_id(child_prop)
                            .unwrap()
                            .as_prop_node_weight()
                            .unwrap()
                            .name(),
                        child_prop
                    ))
                    .join(", ")
            ),
            ValidationIssue::UnknownChildAttributeValue { child } => format!(
                "Child attribute value has unknown (non-child) prop: {} ({})",
                graph
                    .get_node_weight(
                        graph.target(graph.get_node_index_by_id(child)?, EdgeWeightKind::Prop)?
                    )?
                    .as_prop_node_weight()?
                    .name(),
                child,
            ),
            ValidationIssue::ConnectionToUnknownSocket {
                destination_apa: destination,
                source_component,
                source_socket,
            } => {
                format!(
                    "Connection from APA {} to unknown socket {} on component {}",
                    destination, source_socket, source_component,
                )
            }
        })
    }

    fn fix(&self, graph: &mut WorkspaceSnapshotGraph) -> Result<()> {
        match self {
            &ValidationIssue::DuplicateAttributeValue { duplicate, .. } => {
                println!("Removing duplicate attribute value {}", duplicate);
                let node_index = graph.get_node_index_by_id(duplicate)?;
                graph.remove_node(node_index);
            }
            ValidationIssue::ConnectionToUnknownSocket {
                destination_apa, ..
            } => {
                println!("Removing APA {}", destination_apa);
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

fn validate_node(
    graph: &WorkspaceSnapshotGraph,
    node: &NodeWeight,
    node_index: NodeIndex,
) -> Result<Vec<ValidationIssue>> {
    let mut issues = vec![];
    match node {
        NodeWeight::AttributeValue(_) => {
            // If this is an object attribute value, check that it has child attribute values for each child prop
            let attr = node_index;
            let Some(prop) = graph.target_opt(attr, EdgeWeightKind::Prop)? else {
                return Ok(issues);
            };
            if graph.get_node_weight(prop)?.as_prop_node_weight()?.kind() == PropKind::Object {
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
                            child: graph.node_index_to_id(child_attr).unwrap().into(),
                        });
                        continue;
                    }
                    let content = graph
                        .get_node_weight(child_attr)?
                        .get_attribute_value_node_weight()
                        .unwrap()
                        .value();
                    if let Some(orig_content) = attr_content.insert(child_attr_prop, content) {
                        issues.push(ValidationIssue::DuplicateAttributeValue {
                            original: graph.node_index_to_id(child_attr).unwrap().into(),
                            duplicate: graph.node_index_to_id(child_attr).unwrap().into(),
                            content_matches: content == orig_content,
                        });
                    }
                }

                // If any child attributes are *not* associated with child props, report those
                let missing_children: HashSet<_> = child_props
                    .into_iter()
                    .filter(|child_prop| !attr_content.contains_key(child_prop))
                    .map(|child_prop| graph.node_index_to_id(child_prop).unwrap().into())
                    .collect();
                if !missing_children.is_empty() {
                    issues.push(ValidationIssue::MissingChildAttributeValues {
                        object: graph.node_index_to_id(attr).unwrap().into(),
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
                .map(|socket_value| graph.target(socket_value, EdgeWeightKind::Socket).unwrap());
            if !component_sockets.contains(&source_socket) {
                issues.push(ValidationIssue::ConnectionToUnknownSocket {
                    destination_apa: graph.node_index_to_id(destination_apa).unwrap().into(),
                    source_component: graph.node_index_to_id(source_component).unwrap().into(),
                    source_socket: graph.node_index_to_id(source_socket).unwrap().into(),
                })
            }
        }
        _ => {}
    }

    Ok(issues)
}

fn write_snapshot_graph(path: &str, graph: &WorkspaceSnapshotGraph) -> Result<()> {
    let mut file = File::create(path)?;
    let (bytes, _) = serialize::to_vec(graph)?;
    file.write_all(&bytes)?;

    Ok(())
}

#[allow(unused)]
fn remove_node_by_id(graph: &mut WorkspaceSnapshotGraph, id: &str) -> Result<()> {
    let node_id = si_id::ulid::Ulid::from_string(id)?;
    let node_idx = graph.get_node_index_by_id(node_id)?;
    graph.remove_node(node_idx);
    Ok(())
}
