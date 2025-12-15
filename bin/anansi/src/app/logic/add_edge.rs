use crossterm::event::KeyCode;
use dal::workspace_snapshot::edge_weight::EdgeWeight;
use si_events::ulid::Ulid;

use super::App;
use crate::app::state::{
    ActiveModal,
    AddEdgeField,
    GraphEdit,
    PendingEdit,
    SelectedEdgeWeightKind,
};

impl App {
    pub(super) fn open_add_edge_modal(&mut self) {
        self.state.add_edge_modal.source_node_id.clear();
        self.state.add_edge_modal.target_node_id.clear();
        self.state.add_edge_modal.edge_kind_index = 0;
        self.state.add_edge_modal.key.clear();
        self.state.add_edge_modal.is_default = false;
        self.state.add_edge_modal.path.clear();
        self.state.add_edge_modal.focused_field = AddEdgeField::SourceNodeId;
        self.state.add_edge_modal.editing = false;
        self.state.add_edge_modal.source_suggestions.clear();
        self.state.add_edge_modal.source_suggestion_index = 0;
        self.state.add_edge_modal.target_suggestions.clear();
        self.state.add_edge_modal.target_suggestion_index = 0;
        self.state.add_edge_modal.source_valid = false;
        self.state.add_edge_modal.target_valid = false;
        self.state.add_edge_modal.error_message = None;

        if !self.state.node_list.filtered_node_list.is_empty() {
            let selected =
                &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
            self.state.add_edge_modal.source_node_id = selected.node_id.to_string();
            self.state.add_edge_modal.source_valid = true;
        }

        self.update_source_suggestions();
        self.update_target_suggestions();

        self.state.active_modal = Some(ActiveModal::AddEdge);
    }

    pub(super) fn handle_add_edge_modal_keys(&mut self, key_code: KeyCode) {
        self.state.add_edge_modal.error_message = None;

        let state = &mut self.state.add_edge_modal;

        if state.editing {
            match key_code {
                KeyCode::Esc => {
                    state.editing = false;
                }
                KeyCode::Enter => {
                    let field = state.focused_field;
                    self.apply_selected_suggestion(field);
                    self.state.add_edge_modal.editing = false;
                }
                KeyCode::Tab | KeyCode::Down => {
                    let field = state.focused_field;
                    match field {
                        AddEdgeField::SourceNodeId => {
                            if !state.source_suggestions.is_empty() {
                                state.source_suggestion_index = (state.source_suggestion_index + 1)
                                    % state.source_suggestions.len();
                            }
                        }
                        AddEdgeField::TargetNodeId => {
                            if !state.target_suggestions.is_empty() {
                                state.target_suggestion_index = (state.target_suggestion_index + 1)
                                    % state.target_suggestions.len();
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::BackTab | KeyCode::Up => {
                    let field = state.focused_field;
                    match field {
                        AddEdgeField::SourceNodeId => {
                            if !state.source_suggestions.is_empty() {
                                state.source_suggestion_index =
                                    if state.source_suggestion_index == 0 {
                                        state.source_suggestions.len() - 1
                                    } else {
                                        state.source_suggestion_index - 1
                                    };
                            }
                        }
                        AddEdgeField::TargetNodeId => {
                            if !state.target_suggestions.is_empty() {
                                state.target_suggestion_index =
                                    if state.target_suggestion_index == 0 {
                                        state.target_suggestions.len() - 1
                                    } else {
                                        state.target_suggestion_index - 1
                                    };
                            }
                        }
                        _ => {}
                    }
                }
                KeyCode::Backspace => {
                    let field = state.focused_field;
                    match field {
                        AddEdgeField::SourceNodeId => {
                            state.source_node_id.pop();
                        }
                        AddEdgeField::TargetNodeId => {
                            state.target_node_id.pop();
                        }
                        AddEdgeField::Key => {
                            state.key.pop();
                        }
                        AddEdgeField::Path => {
                            state.path.pop();
                        }
                        _ => {}
                    }
                    self.update_suggestions_for_field(field);
                }
                KeyCode::Char(c) => {
                    let field = state.focused_field;
                    match field {
                        AddEdgeField::SourceNodeId => {
                            state.source_node_id.push(c);
                        }
                        AddEdgeField::TargetNodeId => {
                            state.target_node_id.push(c);
                        }
                        AddEdgeField::Key => {
                            state.key.push(c);
                        }
                        AddEdgeField::Path => {
                            state.path.push(c);
                        }
                        _ => {}
                    }
                    self.update_suggestions_for_field(field);
                }
                _ => {}
            }
        } else {
            match key_code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.state.active_modal = None;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.move_add_edge_focus_up();
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.move_add_edge_focus_down();
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let state = &mut self.state.add_edge_modal;
                    match state.focused_field {
                        AddEdgeField::EdgeKind => {
                            if state.edge_kind_index > 0 {
                                state.edge_kind_index -= 1;
                            } else {
                                state.edge_kind_index = SelectedEdgeWeightKind::ALL.len() - 1;
                            }
                        }
                        AddEdgeField::IsDefault => {
                            state.is_default = !state.is_default;
                        }
                        _ => {}
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let state = &mut self.state.add_edge_modal;
                    match state.focused_field {
                        AddEdgeField::EdgeKind => {
                            state.edge_kind_index =
                                (state.edge_kind_index + 1) % SelectedEdgeWeightKind::ALL.len();
                        }
                        AddEdgeField::IsDefault => {
                            state.is_default = !state.is_default;
                        }
                        _ => {}
                    }
                }
                KeyCode::Enter | KeyCode::Char('e') => {
                    let state = &mut self.state.add_edge_modal;
                    match state.focused_field {
                        AddEdgeField::SourceNodeId
                        | AddEdgeField::TargetNodeId
                        | AddEdgeField::Key
                        | AddEdgeField::Path => {
                            state.editing = true;
                        }
                        AddEdgeField::IsDefault => {
                            state.is_default = !state.is_default;
                        }
                        AddEdgeField::EdgeKind => {}
                    }
                }
                KeyCode::Char(' ') => {
                    let state = &mut self.state.add_edge_modal;
                    if state.focused_field == AddEdgeField::IsDefault {
                        state.is_default = !state.is_default;
                    }
                }
                KeyCode::Char('c') => {
                    self.create_edge();
                }
                _ => {}
            }
        }
    }

    pub(super) fn move_add_edge_focus_up(&mut self) {
        let state = &mut self.state.add_edge_modal;
        let current_kind = SelectedEdgeWeightKind::from_index(
            state.edge_kind_index,
            if state.key.is_empty() {
                None
            } else {
                Some(state.key.clone())
            },
            state.is_default,
            state.path.clone(),
        );

        state.focused_field = match state.focused_field {
            AddEdgeField::SourceNodeId => AddEdgeField::SourceNodeId,
            AddEdgeField::TargetNodeId => AddEdgeField::SourceNodeId,
            AddEdgeField::EdgeKind => AddEdgeField::TargetNodeId,
            AddEdgeField::Key => AddEdgeField::EdgeKind,
            AddEdgeField::IsDefault => AddEdgeField::EdgeKind,
            AddEdgeField::Path => AddEdgeField::EdgeKind,
        };

        if state.focused_field == AddEdgeField::Key && !current_kind.needs_key() {
            state.focused_field = AddEdgeField::EdgeKind;
        }
        if state.focused_field == AddEdgeField::IsDefault && !current_kind.needs_is_default() {
            state.focused_field = AddEdgeField::EdgeKind;
        }
        if state.focused_field == AddEdgeField::Path && !current_kind.needs_path() {
            state.focused_field = AddEdgeField::EdgeKind;
        }
    }

    pub(super) fn move_add_edge_focus_down(&mut self) {
        let state = &mut self.state.add_edge_modal;
        let current_kind = SelectedEdgeWeightKind::from_index(
            state.edge_kind_index,
            if state.key.is_empty() {
                None
            } else {
                Some(state.key.clone())
            },
            state.is_default,
            state.path.clone(),
        );

        state.focused_field = match state.focused_field {
            AddEdgeField::SourceNodeId => AddEdgeField::TargetNodeId,
            AddEdgeField::TargetNodeId => AddEdgeField::EdgeKind,
            AddEdgeField::EdgeKind => {
                if current_kind.needs_key() {
                    AddEdgeField::Key
                } else if current_kind.needs_is_default() {
                    AddEdgeField::IsDefault
                } else if current_kind.needs_path() {
                    AddEdgeField::Path
                } else {
                    AddEdgeField::EdgeKind
                }
            }
            AddEdgeField::Key => AddEdgeField::Key,
            AddEdgeField::IsDefault => AddEdgeField::IsDefault,
            AddEdgeField::Path => AddEdgeField::Path,
        };
    }

    pub(super) fn create_edge(&mut self) {
        if !self.state.add_edge_modal.source_valid {
            self.state.add_edge_modal.error_message =
                Some("Source node ID does not exist".to_string());
            return;
        }

        if !self.state.add_edge_modal.target_valid {
            self.state.add_edge_modal.error_message =
                Some("Target node ID does not exist".to_string());
            return;
        }

        let state = &self.state.add_edge_modal;

        let source_ulid = match Ulid::from_string(&state.source_node_id) {
            Ok(ulid) => ulid,
            Err(_) => {
                self.state.add_edge_modal.error_message =
                    Some("Invalid source node ID format".to_string());
                return;
            }
        };

        let target_ulid = match Ulid::from_string(&state.target_node_id) {
            Ok(ulid) => ulid,
            Err(_) => {
                self.state.add_edge_modal.error_message =
                    Some("Invalid target node ID format".to_string());
                return;
            }
        };

        let source_index = self
            .state
            .node_list
            .node_list
            .iter()
            .find(|item| item.node_id == source_ulid)
            .map(|item| item.index);

        let target_index = self
            .state
            .node_list
            .node_list
            .iter()
            .find(|item| item.node_id == target_ulid)
            .map(|item| item.index);

        if source_index.is_none() || target_index.is_none() {
            self.state.add_edge_modal.error_message =
                Some("Source or target node not found".to_string());
            return;
        }

        let key = if state.key.is_empty() {
            None
        } else {
            Some(state.key.clone())
        };

        let selected_kind = SelectedEdgeWeightKind::from_index(
            state.edge_kind_index,
            key,
            state.is_default,
            state.path.clone(),
        );

        let edge_weight_kind = selected_kind.to_edge_weight_kind();
        let edge_weight = EdgeWeight::new(edge_weight_kind.clone());

        let add_edit = GraphEdit::AddEdge {
            source_node_id: source_ulid,
            target_node_id: target_ulid,
            edge_weight,
        };

        if self.apply_edit_and_mark_dirty(&add_edit).is_ok() {
            self.state.pending_edits.push(PendingEdit {
                node_id: source_ulid,
                edit: add_edit,
            });
            self.state.active_modal = None;
        } else {
            self.state.add_edge_modal.error_message = Some("Failed to add edge".to_string());
        }
    }

    pub(super) fn update_source_suggestions(&mut self) {
        let input = self.state.add_edge_modal.source_node_id.to_lowercase();

        self.state.add_edge_modal.source_valid = self
            .state
            .node_list
            .node_list
            .iter()
            .any(|item| item.node_id.to_string().to_lowercase() == input);

        self.state.add_edge_modal.source_suggestions = self
            .state
            .node_list
            .node_list
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if input.is_empty() {
                    return true;
                }
                let id_match = item.node_id.to_string().to_lowercase().contains(&input);
                let name_match = item
                    .name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&input))
                    .unwrap_or(false);
                let kind_match = item.node_weight_kind.to_lowercase().contains(&input);
                id_match || name_match || kind_match
            })
            .take(10)
            .map(|(idx, _)| idx)
            .collect();

        if self.state.add_edge_modal.source_suggestion_index
            >= self.state.add_edge_modal.source_suggestions.len()
        {
            self.state.add_edge_modal.source_suggestion_index = 0;
        }
    }

    pub(super) fn update_target_suggestions(&mut self) {
        let input = self.state.add_edge_modal.target_node_id.to_lowercase();

        self.state.add_edge_modal.target_valid = self
            .state
            .node_list
            .node_list
            .iter()
            .any(|item| item.node_id.to_string().to_lowercase() == input);

        self.state.add_edge_modal.target_suggestions = self
            .state
            .node_list
            .node_list
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if input.is_empty() {
                    return true;
                }
                let id_match = item.node_id.to_string().to_lowercase().contains(&input);
                let name_match = item
                    .name
                    .as_ref()
                    .map(|n| n.to_lowercase().contains(&input))
                    .unwrap_or(false);
                let kind_match = item.node_weight_kind.to_lowercase().contains(&input);
                id_match || name_match || kind_match
            })
            .take(10)
            .map(|(idx, _)| idx)
            .collect();

        if self.state.add_edge_modal.target_suggestion_index
            >= self.state.add_edge_modal.target_suggestions.len()
        {
            self.state.add_edge_modal.target_suggestion_index = 0;
        }
    }

    pub(super) fn update_suggestions_for_field(&mut self, field: AddEdgeField) {
        match field {
            AddEdgeField::SourceNodeId => self.update_source_suggestions(),
            AddEdgeField::TargetNodeId => self.update_target_suggestions(),
            _ => {}
        }
    }

    pub(super) fn apply_selected_suggestion(&mut self, field: AddEdgeField) {
        match field {
            AddEdgeField::SourceNodeId => {
                if !self.state.add_edge_modal.source_suggestions.is_empty() {
                    let idx = self.state.add_edge_modal.source_suggestion_index;
                    let node_idx = self.state.add_edge_modal.source_suggestions[idx];
                    let node_id = self.state.node_list.node_list[node_idx].node_id.to_string();
                    self.state.add_edge_modal.source_node_id = node_id;
                    self.state.add_edge_modal.source_valid = true;
                }
            }
            AddEdgeField::TargetNodeId => {
                if !self.state.add_edge_modal.target_suggestions.is_empty() {
                    let idx = self.state.add_edge_modal.target_suggestion_index;
                    let node_idx = self.state.add_edge_modal.target_suggestions[idx];
                    let node_id = self.state.node_list.node_list[node_idx].node_id.to_string();
                    self.state.add_edge_modal.target_node_id = node_id;
                    self.state.add_edge_modal.target_valid = true;
                }
            }
            _ => {}
        }
    }
}
