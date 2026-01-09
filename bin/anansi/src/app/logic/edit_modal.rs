use crossterm::event::KeyCode;

use super::App;
use crate::app::{
    graph_edit::create_node_edit,
    helpers::{
        extract_node_name,
        get_editable_fields,
    },
    state::{
        ActiveModal,
        EditableFieldType,
        PendingEdit,
    },
};

impl App {
    pub(super) fn open_edit_modal(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
        let node_weight = match self
            .state
            .working_graph
            .get_node_weight(selected_item.index)
        {
            Ok(nw) => nw,
            Err(_) => return,
        };

        let fields = get_editable_fields(node_weight);
        if fields.is_empty() {
            return;
        }

        self.state.edit_modal.editable_fields = fields;
        self.state.edit_modal.field_index = 0;
        self.state.edit_modal.input.clear();
        self.state.edit_modal.editing = false;
        self.state.active_modal = Some(ActiveModal::EditNode);
    }

    pub(super) fn close_edit_modal(&mut self) {
        self.state.active_modal = None;
        self.state.edit_modal.editing = false;
        self.state.edit_modal.input.clear();
        self.state.edit_modal.editable_fields.clear();
    }

    pub(super) fn handle_edit_modal_keys(&mut self, key_code: KeyCode) {
        if self.state.edit_modal.editing {
            match key_code {
                KeyCode::Esc => {
                    self.state.edit_modal.editing = false;
                    self.state.edit_modal.input.clear();
                }
                KeyCode::Enter => {
                    self.apply_current_edit();
                    self.state.edit_modal.editing = false;
                }
                KeyCode::Backspace => {
                    self.state.edit_modal.input.pop();
                }
                KeyCode::Char(c) => {
                    self.state.edit_modal.input.push(c);
                }
                _ => {}
            }
        } else {
            match key_code {
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.close_edit_modal();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    self.reset_enum_selection();
                    if self.state.edit_modal.field_index > 0 {
                        self.state.edit_modal.field_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.reset_enum_selection();
                    if self.state.edit_modal.field_index + 1
                        < self.state.edit_modal.editable_fields.len()
                    {
                        self.state.edit_modal.field_index += 1;
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    self.cycle_enum_option(false);
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    self.cycle_enum_option(true);
                }
                KeyCode::Enter | KeyCode::Char('e') => {
                    if let Some(field) = self
                        .state
                        .edit_modal
                        .editable_fields
                        .get(self.state.edit_modal.field_index)
                    {
                        match &field.field_type {
                            EditableFieldType::Bool => {
                                self.toggle_bool_field();
                            }
                            EditableFieldType::Enum {
                                options,
                                selected_index,
                            } => {
                                if !options.is_empty() {
                                    self.state.edit_modal.input = options[*selected_index].clone();
                                    self.apply_current_edit();
                                }
                            }
                            EditableFieldType::String => {
                                self.state.edit_modal.input = field.value.clone();
                                self.state.edit_modal.editing = true;
                            }
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(field) = self
                        .state
                        .edit_modal
                        .editable_fields
                        .get(self.state.edit_modal.field_index)
                    {
                        if let EditableFieldType::Bool = &field.field_type {
                            self.toggle_bool_field();
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn cycle_enum_option(&mut self, forward: bool) {
        let field_index = self.state.edit_modal.field_index;

        if let Some(field) = self.state.edit_modal.editable_fields.get_mut(field_index) {
            if let EditableFieldType::Enum {
                options,
                selected_index,
            } = &mut field.field_type
            {
                if options.is_empty() {
                    return;
                }

                if forward {
                    *selected_index = (*selected_index + 1) % options.len();
                } else {
                    *selected_index = if *selected_index == 0 {
                        options.len() - 1
                    } else {
                        *selected_index - 1
                    };
                }
            }
        }
    }

    pub(super) fn reset_enum_selection(&mut self) {
        let field_index = self.state.edit_modal.field_index;

        if let Some(field) = self.state.edit_modal.editable_fields.get_mut(field_index) {
            if let EditableFieldType::Enum {
                options,
                selected_index,
            } = &mut field.field_type
            {
                if let Some(idx) = options
                    .iter()
                    .position(|opt| opt.eq_ignore_ascii_case(&field.value))
                {
                    *selected_index = idx;
                }
            }
        }
    }

    pub(super) fn toggle_bool_field(&mut self) {
        if let Some(field) = self
            .state
            .edit_modal
            .editable_fields
            .get(self.state.edit_modal.field_index)
        {
            let new_value = if field.value == "true" {
                "false".to_string()
            } else {
                "true".to_string()
            };
            self.state.edit_modal.input = new_value;
            self.apply_current_edit();
        }
    }

    pub(super) fn apply_current_edit(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let field_index = self.state.edit_modal.field_index;
        let new_value = self.state.edit_modal.input.clone();

        if let Some(field) = self.state.edit_modal.editable_fields.get(field_index) {
            let old_value = field.value.clone();
            let field_name = field.name.clone();

            if old_value == new_value {
                self.state.edit_modal.input.clear();
                return;
            }

            let selected_item =
                &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
            let node_id = selected_item.node_id;

            if let Ok(current_weight) = self
                .state
                .working_graph
                .get_node_weight(selected_item.index)
            {
                if let Ok(node_edit) =
                    create_node_edit(current_weight, node_id, &field_name, &old_value, &new_value)
                {
                    if let Ok(Some(new_weight)) = self.apply_edit_and_mark_dirty(&node_edit) {
                        if let Some(field) =
                            self.state.edit_modal.editable_fields.get_mut(field_index)
                        {
                            field.value = new_value.clone();
                        }

                        let new_name = extract_node_name(&new_weight);
                        for item in &mut self.state.node_list.node_list {
                            if item.node_id == node_id {
                                item.name = new_name.clone();
                                break;
                            }
                        }
                        for item in &mut self.state.node_list.filtered_node_list {
                            if item.node_id == node_id {
                                item.name = new_name.clone();
                                break;
                            }
                        }

                        self.state.pending_edits.push(PendingEdit {
                            node_id,
                            edit: node_edit,
                        });
                    }
                }
            }
        }

        self.state.edit_modal.input.clear();
    }
}
