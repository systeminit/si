use std::collections::HashMap;

use color_eyre::Result;
use dal::{
    EdgeWeightKindDiscriminants,
    PropKind,
    WorkspaceSnapshotGraph,
    action::{
        ActionState,
        prototype::ActionKind,
    },
    func::FuncKind,
    workspace_snapshot::node_weight::{
        NodeWeight,
        NodeWeightDiscriminants,
        category_node_weight::CategoryNodeKind,
    },
};

use super::state::{
    EditableField,
    EditableFieldType,
    NodeListItem,
};

// Enum option constants
pub const PROP_KIND_OPTIONS: &[&str] = &["Array", "Boolean", "Integer", "Map", "Object", "String"];
pub const FUNC_KIND_OPTIONS: &[&str] = &[
    "Action",
    "Attribute",
    "Authentication",
    "CodeGeneration",
    "Intrinsic",
    "Management",
    "Qualification",
    "SchemaVariantDefinition",
];
pub const ACTION_KIND_OPTIONS: &[&str] = &["Create", "Destroy", "Manual", "Refresh", "Update"];
pub const ACTION_STATE_OPTIONS: &[&str] = &["Dispatched", "Failed", "OnHold", "Queued", "Running"];
pub const CATEGORY_KIND_OPTIONS: &[&str] = &[
    "Action",
    "Component",
    "DeprecatedActionBatch",
    "Func",
    "Module",
    "Schema",
    "Secret",
    "DependentValueRoots",
    "View",
    "DiagramObject",
    "DefaultSubscriptionSources",
    "Overlays",
];

pub fn extract_node_name(node_weight: &NodeWeight) -> Option<String> {
    match node_weight {
        NodeWeight::Prop(p) => Some(p.name().to_string()),
        NodeWeight::Func(f) => Some(f.name().to_string()),
        NodeWeight::Category(c) => Some(format!("{:?}", c.kind())),
        NodeWeight::Content(c) => Some(c.content_address_discriminants().to_string()),
        _ => None,
    }
}

pub fn build_node_list(graph: &WorkspaceSnapshotGraph) -> Result<Vec<NodeListItem>> {
    let mut items = Vec::new();

    for (node_weight, node_idx) in graph.nodes() {
        let node_id = node_weight.id();
        let node_weight_kind = format!("{}", NodeWeightDiscriminants::from(node_weight));
        let name = extract_node_name(node_weight);

        items.push(NodeListItem {
            index: node_idx,
            node_id,
            node_weight_kind,
            name,
        });
    }

    // Sort by node ID
    items.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    Ok(items)
}

pub fn find_enum_index(options: &[&str], value: &str) -> usize {
    options
        .iter()
        .position(|&opt| opt.eq_ignore_ascii_case(value))
        .unwrap_or(0)
}

/// Extract editable fields from a node weight
pub fn get_editable_fields(node_weight: &NodeWeight) -> Vec<EditableField> {
    match node_weight {
        NodeWeight::Prop(weight) => {
            let kind_str = format!("{:?}", weight.kind());
            vec![
                EditableField {
                    name: "name".to_string(),
                    value: weight.name().to_string(),
                    field_type: EditableFieldType::String,
                },
                EditableField {
                    name: "kind".to_string(),
                    value: kind_str.clone(),
                    field_type: EditableFieldType::Enum {
                        options: PROP_KIND_OPTIONS.iter().map(|s| s.to_string()).collect(),
                        selected_index: find_enum_index(PROP_KIND_OPTIONS, &kind_str),
                    },
                },
                EditableField {
                    name: "can_be_used_as_prototype_arg".to_string(),
                    value: weight.can_be_used_as_prototype_arg().to_string(),
                    field_type: EditableFieldType::Bool,
                },
                EditableField {
                    name: "content_hash".to_string(),
                    value: weight.content_hash().to_string(),
                    field_type: EditableFieldType::String,
                },
            ]
        }
        NodeWeight::Func(weight) => {
            let func_kind_str = format!("{:?}", weight.func_kind());
            vec![
                EditableField {
                    name: "name".to_string(),
                    value: weight.name().to_string(),
                    field_type: EditableFieldType::String,
                },
                EditableField {
                    name: "func_kind".to_string(),
                    value: func_kind_str.clone(),
                    field_type: EditableFieldType::Enum {
                        options: FUNC_KIND_OPTIONS.iter().map(|s| s.to_string()).collect(),
                        selected_index: find_enum_index(FUNC_KIND_OPTIONS, &func_kind_str),
                    },
                },
                EditableField {
                    name: "content_hash".to_string(),
                    value: weight.content_hash().to_string(),
                    field_type: EditableFieldType::String,
                },
            ]
        }
        NodeWeight::FuncArgument(weight) => {
            vec![
                EditableField {
                    name: "name".to_string(),
                    value: weight.name().to_string(),
                    field_type: EditableFieldType::String,
                },
                EditableField {
                    name: "content_hash".to_string(),
                    value: weight.content_hash().to_string(),
                    field_type: EditableFieldType::String,
                },
            ]
        }
        NodeWeight::ActionPrototype(weight) => {
            let kind_str = format!("{:?}", weight.kind());
            vec![
                EditableField {
                    name: "name".to_string(),
                    value: weight.name().to_string(),
                    field_type: EditableFieldType::String,
                },
                EditableField {
                    name: "description".to_string(),
                    value: weight.description().unwrap_or("").to_string(),
                    field_type: EditableFieldType::String,
                },
                EditableField {
                    name: "kind".to_string(),
                    value: kind_str.clone(),
                    field_type: EditableFieldType::Enum {
                        options: ACTION_KIND_OPTIONS.iter().map(|s| s.to_string()).collect(),
                        selected_index: find_enum_index(ACTION_KIND_OPTIONS, &kind_str),
                    },
                },
            ]
        }
        NodeWeight::Component(weight) => {
            vec![
                EditableField {
                    name: "to_delete".to_string(),
                    value: weight.to_delete().to_string(),
                    field_type: EditableFieldType::Bool,
                },
                EditableField {
                    name: "content_hash".to_string(),
                    value: weight.content_hash().to_string(),
                    field_type: EditableFieldType::String,
                },
            ]
        }
        NodeWeight::Action(weight) => {
            let state_str = format!("{:?}", weight.state());
            vec![
                EditableField {
                    name: "state".to_string(),
                    value: state_str.clone(),
                    field_type: EditableFieldType::Enum {
                        options: ACTION_STATE_OPTIONS.iter().map(|s| s.to_string()).collect(),
                        selected_index: find_enum_index(ACTION_STATE_OPTIONS, &state_str),
                    },
                },
                EditableField {
                    name: "originating_change_set_id".to_string(),
                    value: weight.originating_change_set_id().to_string(),
                    field_type: EditableFieldType::String,
                },
            ]
        }
        NodeWeight::Category(weight) => {
            let kind_str = format!("{:?}", weight.kind());
            vec![EditableField {
                name: "kind".to_string(),
                value: kind_str.clone(),
                field_type: EditableFieldType::Enum {
                    options: CATEGORY_KIND_OPTIONS
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                    selected_index: find_enum_index(CATEGORY_KIND_OPTIONS, &kind_str),
                },
            }]
        }
        NodeWeight::AttributePrototypeArgument(_weight) => {
            vec![]
        }
        NodeWeight::AttributeValue(_weight) => {
            vec![]
        }
        NodeWeight::DependentValueRoot(weight) => {
            vec![EditableField {
                name: "value_id".to_string(),
                value: format!("{}", weight.value_id()),
                field_type: EditableFieldType::String,
            }]
        }
        NodeWeight::FinishedDependentValueRoot(weight) => {
            vec![EditableField {
                name: "value_id".to_string(),
                value: format!("{}", weight.value_id()),
                field_type: EditableFieldType::String,
            }]
        }
        NodeWeight::Content(weight) => {
            vec![
                EditableField {
                    name: "to_delete".to_string(),
                    value: weight.to_delete().to_string(),
                    field_type: EditableFieldType::Bool,
                },
                EditableField {
                    name: "content_hash".to_string(),
                    value: weight.content_hash().to_string(),
                    field_type: EditableFieldType::String,
                },
            ]
        }
        NodeWeight::Secret(weight) => {
            vec![EditableField {
                name: "content_hash".to_string(),
                value: weight.content_hash().to_string(),
                field_type: EditableFieldType::String,
            }]
        }
        NodeWeight::Ordering(weight) => {
            vec![EditableField {
                name: "order".to_string(),
                value: weight
                    .order()
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                field_type: EditableFieldType::String,
            }]
        }
        NodeWeight::InputSocket(_)
        | NodeWeight::SchemaVariant(_)
        | NodeWeight::ManagementPrototype(_)
        | NodeWeight::Geometry(_)
        | NodeWeight::View(_)
        | NodeWeight::DiagramObject(_)
        | NodeWeight::ApprovalRequirementDefinition(_)
        | NodeWeight::Reason(_)
        | NodeWeight::LeafPrototype(_) => {
            vec![]
        }
    }
}

// Parsing helper functions
pub fn parse_prop_kind(value: &str) -> Result<PropKind, &'static str> {
    match value.to_lowercase().as_str() {
        "array" => Ok(PropKind::Array),
        "boolean" | "bool" => Ok(PropKind::Boolean),
        "integer" | "int" => Ok(PropKind::Integer),
        "map" => Ok(PropKind::Map),
        "object" => Ok(PropKind::Object),
        "string" | "str" => Ok(PropKind::String),
        _ => Err("Invalid prop kind"),
    }
}

pub fn parse_func_kind(value: &str) -> Result<FuncKind, &'static str> {
    match value.to_lowercase().as_str() {
        "action" => Ok(FuncKind::Action),
        "attribute" => Ok(FuncKind::Attribute),
        "authentication" => Ok(FuncKind::Authentication),
        "codegeneration" | "codegen" => Ok(FuncKind::CodeGeneration),
        "intrinsic" => Ok(FuncKind::Intrinsic),
        "management" => Ok(FuncKind::Management),
        "qualification" => Ok(FuncKind::Qualification),
        "schemavariantdefinition" | "svd" => Ok(FuncKind::SchemaVariantDefinition),
        _ => Err("Invalid func kind"),
    }
}

pub fn parse_action_kind(value: &str) -> Result<ActionKind, &'static str> {
    match value.to_lowercase().as_str() {
        "create" => Ok(ActionKind::Create),
        "destroy" => Ok(ActionKind::Destroy),
        "manual" => Ok(ActionKind::Manual),
        "refresh" => Ok(ActionKind::Refresh),
        "update" => Ok(ActionKind::Update),
        _ => Err("Invalid action kind"),
    }
}

pub fn parse_action_state(value: &str) -> Result<ActionState, &'static str> {
    match value.to_lowercase().as_str() {
        "dispatched" => Ok(ActionState::Dispatched),
        "failed" => Ok(ActionState::Failed),
        "onhold" => Ok(ActionState::OnHold),
        "queued" => Ok(ActionState::Queued),
        "running" => Ok(ActionState::Running),
        _ => Err("Invalid action state"),
    }
}

pub fn parse_category_kind(value: &str) -> Result<CategoryNodeKind, &'static str> {
    match value.to_lowercase().as_str() {
        "action" => Ok(CategoryNodeKind::Action),
        "component" => Ok(CategoryNodeKind::Component),
        "deprecatedactionbatch" => Ok(CategoryNodeKind::DeprecatedActionBatch),
        "func" => Ok(CategoryNodeKind::Func),
        "module" => Ok(CategoryNodeKind::Module),
        "schema" => Ok(CategoryNodeKind::Schema),
        "secret" => Ok(CategoryNodeKind::Secret),
        "dependentvalueroots" => Ok(CategoryNodeKind::DependentValueRoots),
        "view" => Ok(CategoryNodeKind::View),
        "diagramobject" => Ok(CategoryNodeKind::DiagramObject),
        "defaultsubscriptionsources" => Ok(CategoryNodeKind::DefaultSubscriptionSources),
        "overlays" => Ok(CategoryNodeKind::Overlays),
        _ => Err("Invalid category kind"),
    }
}

/// Statistics about node and edge weights in a snapshot
#[derive(Debug, Clone, Default)]
pub struct SnapshotStats {
    /// Count of nodes per NodeWeight kind
    pub node_counts: HashMap<NodeWeightDiscriminants, usize>,
    /// Count of edges per EdgeWeightKind
    pub edge_counts: HashMap<EdgeWeightKindDiscriminants, usize>,
    /// Serialized byte size per NodeWeight kind
    pub node_bytes: HashMap<NodeWeightDiscriminants, usize>,
    /// Serialized byte size per EdgeWeightKind
    pub edge_bytes: HashMap<EdgeWeightKindDiscriminants, usize>,
    /// Total bytes for all nodes
    pub total_node_bytes: usize,
    /// Total bytes for all edges
    pub total_edge_bytes: usize,
}

/// Compute statistics about node and edge weights in a snapshot
pub fn compute_snapshot_stats(graph: &WorkspaceSnapshotGraph) -> SnapshotStats {
    let mut stats = SnapshotStats::default();

    for (node_weight, _node_idx) in graph.nodes() {
        let discriminant = NodeWeightDiscriminants::from(node_weight);
        *stats.node_counts.entry(discriminant).or_insert(0) += 1;

        // Compute serialized byte size for this node weight
        if let Ok(serialized) = postcard::to_stdvec(node_weight) {
            let byte_size = serialized.len();
            *stats.node_bytes.entry(discriminant).or_insert(0) += byte_size;
            stats.total_node_bytes += byte_size;
        }
    }

    for (edge_weight, _source_idx, _target_idx) in graph.edges() {
        let discriminant = EdgeWeightKindDiscriminants::from(edge_weight.kind());
        *stats.edge_counts.entry(discriminant).or_insert(0) += 1;

        // Compute serialized byte size for this edge weight
        if let Ok(serialized) = postcard::to_stdvec(edge_weight) {
            let byte_size = serialized.len();
            *stats.edge_bytes.entry(discriminant).or_insert(0) += byte_size;
            stats.total_edge_bytes += byte_size;
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use dal::{
        PropKind,
        Ulid,
        WorkspaceSnapshotGraph,
        workspace_snapshot::{
            edge_weight::EdgeWeightKind,
            graph::WorkspaceSnapshotGraphVCurrent,
            node_weight::{
                NodeWeight,
                NodeWeightDiscriminants,
            },
        },
    };
    use si_events::ContentHash;

    use super::*;

    fn create_test_graph() -> WorkspaceSnapshotGraph {
        let inner = WorkspaceSnapshotGraphVCurrent::new_with_categories_only()
            .expect("Unable to create WorkspaceSnapshotGraph");
        WorkspaceSnapshotGraph::V4(inner)
    }

    fn generate_ulid(graph: &WorkspaceSnapshotGraph) -> Ulid {
        graph.generate_ulid().expect("Unable to generate Ulid")
    }

    #[test]
    fn test_compute_snapshot_stats_empty_graph() {
        let graph = create_test_graph();
        let stats = compute_snapshot_stats(&graph);

        // A new graph with categories should have some Category nodes
        assert!(
            stats
                .node_counts
                .contains_key(&NodeWeightDiscriminants::Category),
            "Graph should contain Category nodes"
        );

        // Total node bytes should be non-zero since categories exist
        assert!(
            stats.total_node_bytes > 0,
            "Total node bytes should be > 0 for a graph with categories"
        );

        // Node bytes per kind should match sum of individual kinds
        let sum_node_bytes: usize = stats.node_bytes.values().sum();
        assert_eq!(
            stats.total_node_bytes, sum_node_bytes,
            "Total node bytes should equal sum of bytes per kind"
        );

        // Edge bytes per kind should match sum of individual kinds
        let sum_edge_bytes: usize = stats.edge_bytes.values().sum();
        assert_eq!(
            stats.total_edge_bytes, sum_edge_bytes,
            "Total edge bytes should equal sum of bytes per kind"
        );
    }

    #[test]
    fn test_compute_snapshot_stats_with_prop_nodes() {
        let mut graph = create_test_graph();

        // Add some Prop nodes
        let prop1_id = generate_ulid(&graph);
        let prop1_lineage = generate_ulid(&graph);
        let prop1 = NodeWeight::new_prop(
            prop1_id,
            prop1_lineage,
            PropKind::String,
            "test_prop_1",
            ContentHash::new("content1".as_bytes()),
        );
        graph
            .add_or_replace_node(prop1)
            .expect("Failed to add prop1");

        let prop2_id = generate_ulid(&graph);
        let prop2_lineage = generate_ulid(&graph);
        let prop2 = NodeWeight::new_prop(
            prop2_id,
            prop2_lineage,
            PropKind::Integer,
            "test_prop_2",
            ContentHash::new("content2".as_bytes()),
        );
        graph
            .add_or_replace_node(prop2)
            .expect("Failed to add prop2");

        let stats = compute_snapshot_stats(&graph);

        // Should have exactly 2 Prop nodes
        assert_eq!(
            stats.node_counts.get(&NodeWeightDiscriminants::Prop),
            Some(&2),
            "Should have 2 Prop nodes"
        );

        // Prop bytes should be non-zero
        let prop_bytes = stats.node_bytes.get(&NodeWeightDiscriminants::Prop);
        assert!(
            prop_bytes.is_some() && *prop_bytes.unwrap() > 0,
            "Prop bytes should be > 0"
        );

        // Total node bytes should include prop bytes
        assert!(
            stats.total_node_bytes >= *prop_bytes.unwrap(),
            "Total node bytes should include prop bytes"
        );
    }

    #[test]
    fn test_compute_snapshot_stats_with_edges() {
        let mut graph = create_test_graph();

        // Add two Prop nodes
        let prop1_id = generate_ulid(&graph);
        let prop1_lineage = generate_ulid(&graph);
        let prop1 = NodeWeight::new_prop(
            prop1_id,
            prop1_lineage,
            PropKind::Object,
            "parent_prop",
            ContentHash::new("parent".as_bytes()),
        );
        graph
            .add_or_replace_node(prop1)
            .expect("Failed to add prop1");

        let prop2_id = generate_ulid(&graph);
        let prop2_lineage = generate_ulid(&graph);
        let prop2 = NodeWeight::new_prop(
            prop2_id,
            prop2_lineage,
            PropKind::String,
            "child_prop",
            ContentHash::new("child".as_bytes()),
        );
        graph
            .add_or_replace_node(prop2)
            .expect("Failed to add prop2");

        // Add an edge between them using node indices
        let prop1_index = graph
            .get_node_index_by_id(prop1_id)
            .expect("Failed to get prop1 index");
        let prop2_index = graph
            .get_node_index_by_id(prop2_id)
            .expect("Failed to get prop2 index");
        let edge_weight =
            dal::workspace_snapshot::edge_weight::EdgeWeight::new(EdgeWeightKind::Contain(None));
        graph
            .add_edge(prop1_index, edge_weight, prop2_index)
            .expect("Failed to add edge");

        let stats = compute_snapshot_stats(&graph);

        // Should have at least 1 Contain edge
        let contain_count = stats.edge_counts.get(&EdgeWeightKindDiscriminants::Contain);
        assert!(
            contain_count.is_some() && *contain_count.unwrap() >= 1,
            "Should have at least 1 Contain edge"
        );

        // Contain edge bytes should be non-zero
        let contain_bytes = stats.edge_bytes.get(&EdgeWeightKindDiscriminants::Contain);
        assert!(
            contain_bytes.is_some() && *contain_bytes.unwrap() > 0,
            "Contain edge bytes should be > 0"
        );

        // Total edge bytes should be non-zero
        assert!(stats.total_edge_bytes > 0, "Total edge bytes should be > 0");
    }

    #[test]
    fn test_compute_snapshot_stats_byte_consistency() {
        let mut graph = create_test_graph();

        // Add multiple nodes of the same type
        for i in 0..5 {
            let id = generate_ulid(&graph);
            let lineage = generate_ulid(&graph);
            let prop = NodeWeight::new_prop(
                id,
                lineage,
                PropKind::String,
                format!("prop_{i}"),
                ContentHash::new(format!("content_{i}").as_bytes()),
            );
            graph.add_or_replace_node(prop).expect("Failed to add prop");
        }

        let stats = compute_snapshot_stats(&graph);

        // Verify consistency: node_counts keys should match node_bytes keys
        for kind in stats.node_counts.keys() {
            assert!(
                stats.node_bytes.contains_key(kind),
                "node_bytes should have entry for {kind:?}"
            );
        }

        // Verify consistency: edge_counts keys should match edge_bytes keys
        for kind in stats.edge_counts.keys() {
            assert!(
                stats.edge_bytes.contains_key(kind),
                "edge_bytes should have entry for {kind:?}"
            );
        }

        // Verify totals match sums
        let sum_node_bytes: usize = stats.node_bytes.values().sum();
        assert_eq!(
            stats.total_node_bytes, sum_node_bytes,
            "total_node_bytes should equal sum of node_bytes"
        );

        let sum_edge_bytes: usize = stats.edge_bytes.values().sum();
        assert_eq!(
            stats.total_edge_bytes, sum_edge_bytes,
            "total_edge_bytes should equal sum of edge_bytes"
        );
    }

    #[test]
    fn test_compute_snapshot_stats_serialization_produces_bytes() {
        let mut graph = create_test_graph();

        // Add a single node
        let id = generate_ulid(&graph);
        let lineage = generate_ulid(&graph);
        let prop = NodeWeight::new_prop(
            id,
            lineage,
            PropKind::String,
            "serialization_test",
            ContentHash::new("test_content".as_bytes()),
        );
        graph
            .add_or_replace_node(prop.clone())
            .expect("Failed to add prop");

        let stats = compute_snapshot_stats(&graph);

        // Verify that the serialized size matches what we'd get from direct serialization
        let direct_serialized = postcard::to_stdvec(&prop).expect("Failed to serialize prop");
        let prop_bytes = stats
            .node_bytes
            .get(&NodeWeightDiscriminants::Prop)
            .unwrap_or(&0);

        // The prop bytes should be at least as large as one serialized prop
        assert!(
            *prop_bytes >= direct_serialized.len(),
            "Prop bytes ({}) should be >= direct serialization size ({})",
            prop_bytes,
            direct_serialized.len()
        );
    }
}
