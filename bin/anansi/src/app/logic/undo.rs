use dal::workspace_snapshot::node_weight::NodeWeightDiscriminants;
use petgraph::Direction;

use super::App;
use crate::app::{
    helpers::{
        build_node_list,
        extract_node_name,
    },
    state::{
        GraphEdit,
        NodeListItem,
        PendingEdit,
    },
};

impl App {
    pub(super) fn restore_to_original_snapshot(&mut self) {
        self.state.working_graph = self.state.original_graph.clone();
        self.state.is_dirty = false;

        if let Ok(node_list) = build_node_list(&self.state.working_graph) {
            self.state.node_list.node_list = node_list;
            self.state.node_list.update_filter();
        }
    }

    pub(super) fn undo_last_edit(&mut self) {
        let Some(pending_edit) = self.state.pending_edits.pop() else {
            return;
        };

        if self.state.pending_edits.is_empty() {
            self.restore_to_original_snapshot();
            return;
        }

        let node_id = pending_edit.node_id;
        let reverse_edit = pending_edit.edit.reverse();

        let result = self.apply_edit_and_mark_dirty(&reverse_edit);

        match reverse_edit {
            GraphEdit::AddNode {
                node_weight_id,
                added_weight,
            } => {
                if result.is_ok() {
                    if let Ok(node_index) = self
                        .state
                        .working_graph
                        .get_node_index_by_id(node_weight_id)
                    {
                        let node_weight_kind =
                            format!("{}", NodeWeightDiscriminants::from(&*added_weight));
                        let name = extract_node_name(&added_weight);

                        let new_item = NodeListItem {
                            index: node_index,
                            node_id: node_weight_id,
                            node_weight_kind,
                            name,
                        };

                        self.state.node_list.node_list.push(new_item.clone());

                        if self.state.node_list.filter_text.is_empty() {
                            self.state.node_list.filtered_node_list.push(new_item);
                        } else {
                            let filter_lower = self.state.node_list.filter_text.to_lowercase();
                            let matches = new_item
                                .node_weight_kind
                                .to_lowercase()
                                .contains(&filter_lower)
                                || new_item.node_id.to_string().contains(&filter_lower)
                                || new_item
                                    .name
                                    .as_ref()
                                    .is_some_and(|n| n.to_lowercase().contains(&filter_lower));
                            if matches {
                                self.state.node_list.filtered_node_list.push(new_item);
                            }
                        }

                        self.state
                            .node_list
                            .node_list
                            .sort_by(|a, b| a.node_id.cmp(&b.node_id));
                        self.state
                            .node_list
                            .filtered_node_list
                            .sort_by(|a, b| a.node_id.cmp(&b.node_id));
                    }
                }
            }
            GraphEdit::DeleteNode { node_weight_id, .. } => {
                if result.is_ok() {
                    self.state
                        .node_list
                        .node_list
                        .retain(|item| item.node_id != node_weight_id);
                    self.state
                        .node_list
                        .filtered_node_list
                        .retain(|item| item.node_id != node_weight_id);

                    if self.state.node_list.scroll_offset
                        >= self.state.node_list.filtered_node_list.len()
                        && !self.state.node_list.filtered_node_list.is_empty()
                    {
                        self.state.node_list.scroll_offset = 0;
                    }
                }
            }
            GraphEdit::DeleteEdge { target_node_id, .. } => {
                if result.is_ok() {
                    if let Ok(target_idx) = self
                        .state
                        .working_graph
                        .get_node_index_by_id(target_node_id)
                    {
                        let remaining_incoming_edges: Vec<_> = self
                            .state
                            .working_graph
                            .edges_directed(target_idx, Direction::Incoming)
                            .collect();

                        if remaining_incoming_edges.is_empty() {
                            if let Ok(target_weight) =
                                self.state.working_graph.get_node_weight(target_idx)
                            {
                                let delete_node_edit = GraphEdit::DeleteNode {
                                    node_weight_id: target_node_id,
                                    deleted_weight: Box::new(target_weight.clone()),
                                };

                                if self.apply_edit_and_mark_dirty(&delete_node_edit).is_ok() {
                                    self.state.pending_edits.push(PendingEdit {
                                        node_id: target_node_id,
                                        edit: delete_node_edit,
                                    });

                                    self.state
                                        .node_list
                                        .node_list
                                        .retain(|item| item.node_id != target_node_id);
                                    self.state
                                        .node_list
                                        .filtered_node_list
                                        .retain(|item| item.node_id != target_node_id);

                                    if self.state.node_list.scroll_offset
                                        >= self.state.node_list.filtered_node_list.len()
                                        && !self.state.node_list.filtered_node_list.is_empty()
                                    {
                                        self.state.node_list.scroll_offset = 0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            _ => {
                if let Ok(Some(new_weight)) = result {
                    let new_name = extract_node_name(&new_weight);
                    for item in &mut self.state.node_list.node_list {
                        if item.node_id == node_id {
                            item.name = new_name.clone();
                            break;
                        }
                    }
                    for item in &mut self.state.node_list.filtered_node_list {
                        if item.node_id == node_id {
                            item.name = new_name;
                            break;
                        }
                    }
                }
            }
        }
    }
}
