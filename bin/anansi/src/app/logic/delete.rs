use crossterm::event::KeyCode;
use petgraph::{
    Direction,
    visit::EdgeRef,
};
use si_events::ulid::Ulid;

use super::App;
use crate::app::state::{
    GraphEdit,
    PendingEdit,
};

impl App {
    pub(super) fn handle_delete_confirm_keys(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.delete_selected_node();
                self.state.active_modal = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state.active_modal = None;
            }
            _ => {}
        }
    }

    pub(super) fn handle_edge_delete_confirm_keys(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.delete_selected_edge();
                self.state.active_modal = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state.active_modal = None;
            }
            _ => {}
        }
    }

    pub(super) fn delete_selected_node(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
        let node_id = selected_item.node_id;
        let node_index = selected_item.index;

        let Ok(node_weight) = self.state.working_graph.get_node_weight(node_index) else {
            return;
        };
        let deleted_weight = Box::new(node_weight.clone());

        let mut edge_edits = Vec::new();

        let outgoing_edges: Vec<_> = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Outgoing)
            .collect();
        let mut outgoing_target_ids = Vec::new();
        for edge_ref in outgoing_edges {
            let target_idx = edge_ref.target();
            if let Ok(target_weight) = self.state.working_graph.get_node_weight(target_idx) {
                let target_id = target_weight.id();
                outgoing_target_ids.push(target_id);
                edge_edits.push(PendingEdit {
                    node_id,
                    edit: GraphEdit::DeleteEdge {
                        source_node_id: node_id,
                        target_node_id: target_id,
                        edge_weight: edge_ref.weight().clone(),
                    },
                });
            }
        }

        let incoming_edges: Vec<_> = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Incoming)
            .collect();
        for edge_ref in incoming_edges {
            let source_idx = edge_ref.source();
            if let Ok(source_weight) = self.state.working_graph.get_node_weight(source_idx) {
                edge_edits.push(PendingEdit {
                    node_id,
                    edit: GraphEdit::DeleteEdge {
                        source_node_id: source_weight.id(),
                        target_node_id: node_id,
                        edge_weight: edge_ref.weight().clone(),
                    },
                });
            }
        }

        let delete_edit = GraphEdit::DeleteNode {
            node_weight_id: node_id,
            deleted_weight,
        };

        if self.apply_edit_and_mark_dirty(&delete_edit).is_ok() {
            self.state.pending_edits.extend(edge_edits);

            self.state.pending_edits.push(PendingEdit {
                node_id,
                edit: delete_edit,
            });

            self.state
                .node_list
                .node_list
                .retain(|item| item.node_id != node_id);
            self.state
                .node_list
                .filtered_node_list
                .retain(|item| item.node_id != node_id);

            self.cascade_delete_orphaned_nodes(outgoing_target_ids);

            if self.state.node_list.selected_index >= self.state.node_list.filtered_node_list.len()
                && self.state.node_list.selected_index > 0
            {
                self.state.node_list.selected_index -= 1;
            }

            if self.state.node_list.scroll_offset >= self.state.node_list.filtered_node_list.len()
                && !self.state.node_list.filtered_node_list.is_empty()
            {
                self.state.node_list.scroll_offset = 0;
            }
        }

        self.state.edge_panel.selected_edge = 0;
        self.state.edge_panel.scroll_offset = 0;
    }

    pub(super) fn cascade_delete_orphaned_nodes(&mut self, target_node_ids: Vec<Ulid>) {
        for target_node_id in target_node_ids {
            let Ok(target_idx) = self
                .state
                .working_graph
                .get_node_index_by_id(target_node_id)
            else {
                continue;
            };

            let remaining_incoming_edges: Vec<_> = self
                .state
                .working_graph
                .edges_directed(target_idx, Direction::Incoming)
                .collect();

            if remaining_incoming_edges.is_empty() {
                let Ok(target_weight) = self.state.working_graph.get_node_weight(target_idx) else {
                    continue;
                };
                let deleted_weight = Box::new(target_weight.clone());

                let outgoing_edges: Vec<_> = self
                    .state
                    .working_graph
                    .edges_directed(target_idx, Direction::Outgoing)
                    .collect();

                let mut edge_edits = Vec::new();
                let mut next_target_ids = Vec::new();
                for edge_ref in outgoing_edges {
                    let next_target_idx = edge_ref.target();
                    if let Ok(next_target_weight) =
                        self.state.working_graph.get_node_weight(next_target_idx)
                    {
                        let next_target_id = next_target_weight.id();
                        next_target_ids.push(next_target_id);
                        edge_edits.push(PendingEdit {
                            node_id: target_node_id,
                            edit: GraphEdit::DeleteEdge {
                                source_node_id: target_node_id,
                                target_node_id: next_target_id,
                                edge_weight: edge_ref.weight().clone(),
                            },
                        });
                    }
                }

                let delete_node_edit = GraphEdit::DeleteNode {
                    node_weight_id: target_node_id,
                    deleted_weight,
                };

                if self.apply_edit_and_mark_dirty(&delete_node_edit).is_ok() {
                    self.state.pending_edits.extend(edge_edits);

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

                    self.cascade_delete_orphaned_nodes(next_target_ids);
                }
            }
        }
    }

    pub(super) fn delete_selected_edge(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
        let node_id = selected_item.node_id;
        let node_index = selected_item.index;

        let outgoing_edges: Vec<_> = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Outgoing)
            .collect();
        let incoming_edges: Vec<_> = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Incoming)
            .collect();

        let outgoing_count = outgoing_edges.len();
        let total_edges = outgoing_count + incoming_edges.len();

        if total_edges == 0 || self.state.edge_panel.selected_edge >= total_edges {
            return;
        }

        let selected_edge_idx = self.state.edge_panel.selected_edge;

        if selected_edge_idx < outgoing_count {
            let edge_ref = &outgoing_edges[selected_edge_idx];
            let target_idx = edge_ref.target();

            if let Ok(target_weight) = self.state.working_graph.get_node_weight(target_idx) {
                let target_node_id = target_weight.id();
                let edge_weight = edge_ref.weight().clone();

                let delete_edit = GraphEdit::DeleteEdge {
                    source_node_id: node_id,
                    target_node_id,
                    edge_weight,
                };

                if self.apply_edit_and_mark_dirty(&delete_edit).is_ok() {
                    self.state.pending_edits.push(PendingEdit {
                        node_id,
                        edit: delete_edit,
                    });

                    // Check if target node is now orphaned
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

                                    if !self.state.node_list.filtered_node_list.is_empty() {
                                        self.state.node_list.selected_index =
                                            self.state.node_list.selected_index.min(
                                                self.state.node_list.filtered_node_list.len() - 1,
                                            );

                                        if self.state.node_list.scroll_offset
                                            >= self.state.node_list.filtered_node_list.len()
                                        {
                                            self.state.node_list.scroll_offset = 0;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let new_total = total_edges - 1;
                    if self.state.edge_panel.selected_edge >= new_total && new_total > 0 {
                        self.state.edge_panel.selected_edge = new_total - 1;
                    } else if new_total == 0 {
                        self.state.edge_panel.selected_edge = 0;
                        self.state.edge_panel.scroll_offset = 0;
                    }
                }
            }
        } else {
            let incoming_idx = selected_edge_idx - outgoing_count;
            let edge_ref = &incoming_edges[incoming_idx];
            let source_idx = edge_ref.source();

            if let Ok(source_weight) = self.state.working_graph.get_node_weight(source_idx) {
                let source_node_id = source_weight.id();
                let edge_weight = edge_ref.weight().clone();

                let delete_edit = GraphEdit::DeleteEdge {
                    source_node_id,
                    target_node_id: node_id,
                    edge_weight,
                };

                if self.apply_edit_and_mark_dirty(&delete_edit).is_ok() {
                    self.state.pending_edits.push(PendingEdit {
                        node_id,
                        edit: delete_edit,
                    });

                    let new_total = total_edges - 1;
                    if self.state.edge_panel.selected_edge >= new_total && new_total > 0 {
                        self.state.edge_panel.selected_edge = new_total - 1;
                    } else if new_total == 0 {
                        self.state.edge_panel.selected_edge = 0;
                        self.state.edge_panel.scroll_offset = 0;
                    }
                }
            }
        }
    }
}
