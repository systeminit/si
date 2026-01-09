use dal::{
    ChangeSetId,
    WorkspaceSnapshotGraph,
    workspace_snapshot::{
        edge_weight::EdgeWeightKindDiscriminants,
        node_weight::NodeWeight,
    },
};
use si_events::{
    ContentHash,
    ulid::Ulid,
};

use super::{
    helpers::{
        parse_action_kind,
        parse_action_state,
        parse_category_kind,
        parse_func_kind,
        parse_prop_kind,
    },
    state::GraphEdit,
};

/// Create a typed NodeEdit from field name and string values
pub fn create_node_edit(
    node_weight: &NodeWeight,
    node_weight_id: Ulid,
    field_name: &str,
    old_value: &str,
    new_value: &str,
) -> Result<GraphEdit, &'static str> {
    // Handle content_hash for all node types that support it
    if field_name == "content_hash" {
        let old: ContentHash = old_value.parse().map_err(|_| "Invalid old content hash")?;
        let new: ContentHash = new_value.parse().map_err(|_| "Invalid new content hash")?;
        return Ok(GraphEdit::ContentHash {
            node_weight_id,
            old,
            new,
        });
    }

    match node_weight {
        NodeWeight::Prop(weight) => match field_name {
            "name" => Ok(GraphEdit::PropName {
                node_weight_id,
                old: old_value.to_string(),
                new: new_value.to_string(),
            }),
            "kind" => {
                let old = parse_prop_kind(old_value)?;
                let new = parse_prop_kind(new_value)?;
                Ok(GraphEdit::PropKind {
                    node_weight_id,
                    old,
                    new,
                })
            }
            "can_be_used_as_prototype_arg" => {
                let old = weight.can_be_used_as_prototype_arg();
                let new = new_value.to_lowercase() == "true" || new_value == "1";
                Ok(GraphEdit::PropCanBeUsedAsPrototypeArg {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::Func(weight) => match field_name {
            "name" => Ok(GraphEdit::FuncName {
                node_weight_id,
                old: old_value.to_string(),
                new: new_value.to_string(),
            }),
            "func_kind" => {
                let old = weight.func_kind();
                let new = parse_func_kind(new_value)?;
                Ok(GraphEdit::FuncKind {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::FuncArgument(_) => match field_name {
            "name" => Ok(GraphEdit::FuncArgumentName {
                node_weight_id,
                old: old_value.to_string(),
                new: new_value.to_string(),
            }),
            _ => Err("Unknown field"),
        },
        NodeWeight::ActionPrototype(weight) => match field_name {
            "name" => Ok(GraphEdit::ActionPrototypeName {
                node_weight_id,
                old: old_value.to_string(),
                new: new_value.to_string(),
            }),
            "description" => {
                let old = weight.description().map(|s| s.to_string());
                let new = if new_value.is_empty() {
                    None
                } else {
                    Some(new_value.to_string())
                };
                Ok(GraphEdit::ActionPrototypeDescription {
                    node_weight_id,
                    old,
                    new,
                })
            }
            "kind" => {
                let old = weight.kind();
                let new = parse_action_kind(new_value)?;
                Ok(GraphEdit::ActionPrototypeKind {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::Component(weight) => match field_name {
            "to_delete" => {
                let old = weight.to_delete();
                let new = new_value.to_lowercase() == "true" || new_value == "1";
                Ok(GraphEdit::ComponentToDelete {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::Action(weight) => match field_name {
            "state" => {
                let old = weight.state();
                let new = parse_action_state(new_value)?;
                Ok(GraphEdit::ActionState {
                    node_weight_id,
                    old,
                    new,
                })
            }
            "originating_change_set_id" => {
                let old = weight.originating_change_set_id();
                let ulid = Ulid::from_string(new_value).map_err(|_| "Invalid ULID")?;
                let new = ChangeSetId::from(ulid);
                Ok(GraphEdit::ActionOriginatingChangeSetId {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::Category(weight) => match field_name {
            "kind" => {
                let old = weight.kind();
                let new = parse_category_kind(new_value)?;
                Ok(GraphEdit::CategoryKind {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::DependentValueRoot(weight) => match field_name {
            "value_id" => {
                let old = weight.value_id();
                let new = Ulid::from_string(new_value).map_err(|_| "Invalid ULID")?;
                Ok(GraphEdit::DependentValueRootValueId {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::FinishedDependentValueRoot(weight) => match field_name {
            "value_id" => {
                let old = weight.value_id();
                let new = Ulid::from_string(new_value).map_err(|_| "Invalid ULID")?;
                Ok(GraphEdit::FinishedDependentValueRootValueId {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::Content(weight) => match field_name {
            "to_delete" => {
                let old = weight.to_delete();
                let new = new_value.to_lowercase() == "true" || new_value == "1";
                Ok(GraphEdit::ContentToDelete {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        NodeWeight::Ordering(weight) => match field_name {
            "order" => {
                let old = weight.order().to_vec();
                let new: Result<Vec<Ulid>, _> = new_value
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| Ulid::from_string(s.trim()))
                    .collect();
                let new = new.map_err(|_| "Invalid ULID in order list")?;
                Ok(GraphEdit::OrderingOrder {
                    node_weight_id,
                    old,
                    new,
                })
            }
            _ => Err("Unknown field"),
        },
        _ => Err("Node type not editable"),
    }
}

/// Apply a GraphEdit to the graph. Returns the modified NodeWeight for node edits.
pub fn apply_graph_edit(
    graph: &mut WorkspaceSnapshotGraph,
    edit: &GraphEdit,
) -> Result<Option<NodeWeight>, &'static str> {
    match edit {
        GraphEdit::ContentHash {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            new_weight
                .new_content_hash(*new)
                .map_err(|_| "Cannot set content hash on this node type")?;
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::PropName {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Prop(weight) = &mut new_weight {
                weight.set_name(new);
            } else {
                return Err("Not a Prop node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::PropKind {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Prop(weight) = &mut new_weight {
                weight.set_kind(*new);
            } else {
                return Err("Not a Prop node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::PropCanBeUsedAsPrototypeArg {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Prop(weight) = &mut new_weight {
                weight.set_can_be_used_as_prototype_arg(*new);
            } else {
                return Err("Not a Prop node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::FuncName {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Func(weight) = &mut new_weight {
                weight.set_name(new);
            } else {
                return Err("Not a Func node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::FuncKind {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Func(weight) = &mut new_weight {
                weight.set_func_kind(*new);
            } else {
                return Err("Not a Func node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::FuncArgumentName {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::FuncArgument(weight) = &mut new_weight {
                weight.set_name(new);
            } else {
                return Err("Not a FuncArgument node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ActionPrototypeName {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::ActionPrototype(weight) = &mut new_weight {
                weight.set_name(new);
            } else {
                return Err("Not an ActionPrototype node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ActionPrototypeDescription {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::ActionPrototype(weight) = &mut new_weight {
                weight.set_description(new.clone());
            } else {
                return Err("Not an ActionPrototype node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ActionPrototypeKind {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::ActionPrototype(weight) = &mut new_weight {
                weight.set_kind(*new);
            } else {
                return Err("Not an ActionPrototype node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ComponentToDelete {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Component(weight) = &mut new_weight {
                weight.set_to_delete(*new);
            } else {
                return Err("Not a Component node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ActionState {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Action(weight) = &mut new_weight {
                weight.set_state(*new);
            } else {
                return Err("Not an Action node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ActionOriginatingChangeSetId {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Action(weight) = &mut new_weight {
                weight.set_originating_change_set_id(*new);
            } else {
                return Err("Not an Action node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::CategoryKind {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Category(weight) = &mut new_weight {
                weight.set_kind(*new);
            } else {
                return Err("Not a Category node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::DependentValueRootValueId {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::DependentValueRoot(weight) = &mut new_weight {
                weight.set_value_id(*new);
            } else {
                return Err("Not a DependentValueRoot node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::FinishedDependentValueRootValueId {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::FinishedDependentValueRoot(weight) = &mut new_weight {
                weight.set_value_id(*new);
            } else {
                return Err("Not a FinishedDependentValueRoot node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::ContentToDelete {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Content(weight) = &mut new_weight {
                weight.set_to_delete(*new);
            } else {
                return Err("Not a Content node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::OrderingOrder {
            node_weight_id,
            new,
            ..
        } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            let current_weight = graph
                .get_node_weight(node_index)
                .map_err(|_| "Failed to get node weight")?;
            let mut new_weight = current_weight.clone();
            if let NodeWeight::Ordering(weight) = &mut new_weight {
                weight.set_order(new.clone());
            } else {
                return Err("Not an Ordering node");
            }
            graph
                .add_or_replace_node(new_weight.clone())
                .map_err(|_| "Failed to replace node")?;
            Ok(Some(new_weight))
        }
        GraphEdit::DeleteNode { node_weight_id, .. } => {
            let node_index = graph
                .get_node_index_by_id(*node_weight_id)
                .map_err(|_| "Node not found")?;
            graph.remove_node(node_index);
            graph.remove_node_id(*node_weight_id);
            Ok(None)
        }
        GraphEdit::AddNode { added_weight, .. } => {
            graph
                .add_or_replace_node((**added_weight).clone())
                .map_err(|_| "Failed to add node")?;
            Ok(Some((**added_weight).clone()))
        }
        GraphEdit::DeleteEdge {
            source_node_id,
            target_node_id,
            edge_weight,
        } => {
            let source_index = graph
                .get_node_index_by_id(*source_node_id)
                .map_err(|_| "Source node not found")?;
            let target_index = graph
                .get_node_index_by_id(*target_node_id)
                .map_err(|_| "Target node not found")?;
            let edge_kind_discriminant: EdgeWeightKindDiscriminants = edge_weight.kind().into();
            graph
                .remove_edge(source_index, target_index, edge_kind_discriminant)
                .map_err(|_| "Failed to remove edge")?;
            Ok(None)
        }
        GraphEdit::AddEdge {
            source_node_id,
            target_node_id,
            edge_weight,
        } => {
            let source_index = graph
                .get_node_index_by_id(*source_node_id)
                .map_err(|_| "Source node not found")?;
            let target_index = graph
                .get_node_index_by_id(*target_node_id)
                .map_err(|_| "Target node not found")?;
            graph
                .add_edge(source_index, edge_weight.clone(), target_index)
                .map_err(|_| "Failed to add edge")?;
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use dal::{
        PropKind,
        workspace_snapshot::{
            graph::WorkspaceSnapshotGraphVCurrent,
            node_weight::NodeWeight,
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
    fn test_apply_content_hash_on_prop() {
        let mut graph = create_test_graph();
        let node_id = generate_ulid(&graph);
        let lineage_id = generate_ulid(&graph);

        let old_hash = ContentHash::new("old_content".as_bytes());
        let new_hash = ContentHash::new("new_content".as_bytes());

        let prop =
            NodeWeight::new_prop(node_id, lineage_id, PropKind::String, "test_prop", old_hash);
        graph.add_or_replace_node(prop).expect("add node");

        let edit = GraphEdit::ContentHash {
            node_weight_id: node_id,
            old: old_hash,
            new: new_hash,
        };

        let result = apply_graph_edit(&mut graph, &edit);
        assert!(result.is_ok());

        let new_weight = result.unwrap().unwrap();
        if let NodeWeight::Prop(p) = new_weight {
            assert_eq!(p.content_hash(), new_hash);
        } else {
            panic!("Expected Prop node");
        }
    }

    #[test]
    fn test_apply_prop_name() {
        let mut graph = create_test_graph();
        let node_id = generate_ulid(&graph);
        let lineage_id = generate_ulid(&graph);
        let hash = ContentHash::new("content".as_bytes());

        let prop = NodeWeight::new_prop(node_id, lineage_id, PropKind::String, "old_name", hash);
        graph.add_or_replace_node(prop).expect("add node");

        let edit = GraphEdit::PropName {
            node_weight_id: node_id,
            old: "old_name".to_string(),
            new: "new_name".to_string(),
        };

        let result = apply_graph_edit(&mut graph, &edit);
        assert!(result.is_ok());

        let new_weight = result.unwrap().unwrap();
        if let NodeWeight::Prop(p) = new_weight {
            assert_eq!(p.name(), "new_name");
        } else {
            panic!("Expected Prop node");
        }
    }

    #[test]
    fn test_apply_delete_node() {
        let mut graph = create_test_graph();
        let node_id = generate_ulid(&graph);
        let lineage_id = generate_ulid(&graph);
        let hash = ContentHash::new("content".as_bytes());

        let prop = NodeWeight::new_prop(node_id, lineage_id, PropKind::String, "test_prop", hash);
        graph.add_or_replace_node(prop.clone()).expect("add node");

        assert!(graph.get_node_index_by_id(node_id).is_ok());

        let edit = GraphEdit::DeleteNode {
            node_weight_id: node_id,
            deleted_weight: Box::new(prop),
        };

        let result = apply_graph_edit(&mut graph, &edit);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        assert!(graph.get_node_index_by_id(node_id).is_err());
    }

    #[test]
    fn test_apply_add_node() {
        let mut graph = create_test_graph();
        let node_id = generate_ulid(&graph);
        let lineage_id = generate_ulid(&graph);
        let hash = ContentHash::new("content".as_bytes());

        let prop = NodeWeight::new_prop(node_id, lineage_id, PropKind::String, "new_prop", hash);

        assert!(graph.get_node_index_by_id(node_id).is_err());

        let edit = GraphEdit::AddNode {
            node_weight_id: node_id,
            added_weight: Box::new(prop),
        };

        let result = apply_graph_edit(&mut graph, &edit);
        assert!(result.is_ok());

        assert!(graph.get_node_index_by_id(node_id).is_ok());
    }
}
