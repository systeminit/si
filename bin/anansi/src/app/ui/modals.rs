use dal::workspace_snapshot::node_weight::NodeWeightDiscriminants;
use petgraph::{
    Direction,
    visit::EdgeRef,
};
use ratatui::{
    Frame,
    layout::{
        Alignment,
        Rect,
    },
    style::{
        Color,
        Modifier,
        Style,
    },
    text::{
        Line,
        Span,
    },
    widgets::{
        Block,
        Borders,
        Clear,
        Paragraph,
        Wrap,
    },
};

use super::helpers::{
    ScrollView,
    format_bytes,
    render_add_edge_field,
    render_add_edge_node_field,
    render_suggestion_line,
    truncate_string,
};
use crate::app::state::{
    AddEdgeField,
    AppState,
    EditableFieldType,
    SelectedEdgeWeightKind,
};

pub fn render_edit_modal(f: &mut Frame, state: &AppState) {
    // Calculate centered modal area
    let area = f.area();
    let modal_width = area.width.clamp(50, 70);
    let modal_height = (state.edit_modal.editable_fields.len() as u16 + 6).min(area.height - 4);

    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);

    // Get the node info for the title
    let title = if !state.node_list.filtered_node_list.is_empty() {
        let item = &state.node_list.filtered_node_list[state.node_list.selected_index];
        format!("Edit {} ({})", item.node_weight_kind, item.node_id)
    } else {
        "Edit Node".to_string()
    };

    // Build the content lines
    let mut lines = Vec::new();

    // Instructions
    if state.edit_modal.editing {
        lines.push(Line::from(Span::styled(
            "Type to edit, Enter to save, Esc to cancel",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "↑/↓: Select | ←/→: Cycle enum | Enter: Confirm/Edit | Space: Toggle bool | Esc: Close",
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::from(""));

    // Render each editable field
    for (i, field) in state.edit_modal.editable_fields.iter().enumerate() {
        let is_selected = i == state.edit_modal.field_index;

        let value_spans = match &field.field_type {
            EditableFieldType::String => {
                if state.edit_modal.editing && is_selected {
                    // Show the edit input with cursor
                    vec![Span::styled(
                        format!("{}_", state.edit_modal.input),
                        Style::default().fg(Color::White).bg(Color::DarkGray),
                    )]
                } else {
                    vec![Span::styled(
                        field.value.clone(),
                        if is_selected {
                            Style::default().fg(Color::White).bg(Color::DarkGray)
                        } else {
                            Style::default()
                        },
                    )]
                }
            }
            EditableFieldType::Bool => {
                // Show checkbox for booleans
                let checkbox = if field.value == "true" { "[x]" } else { "[ ]" };
                vec![Span::styled(
                    checkbox,
                    if is_selected {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                )]
            }
            EditableFieldType::Enum {
                options,
                selected_index,
            } => {
                // Show select box style: < Option >
                let mut spans = Vec::new();

                if is_selected {
                    spans.push(Span::styled(
                        "◀ ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::raw("  "));
                }

                // Show all options with the selected one highlighted
                for (opt_idx, option) in options.iter().enumerate() {
                    if opt_idx > 0 {
                        spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
                    }

                    if opt_idx == *selected_index {
                        spans.push(Span::styled(
                            option.clone(),
                            if is_selected {
                                Style::default()
                                    .fg(Color::White)
                                    .bg(Color::Blue)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD)
                            },
                        ));
                    } else {
                        spans.push(Span::styled(
                            option.clone(),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                }

                if is_selected {
                    spans.push(Span::styled(
                        " ▶",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                spans
            }
        };

        let mut field_spans = if is_selected {
            vec![
                Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}: ", field.name),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ]
        } else {
            vec![
                Span::raw("  "),
                Span::styled(
                    format!("{}: ", field.name),
                    Style::default().fg(Color::Gray),
                ),
            ]
        };

        field_spans.extend(value_spans);
        lines.push(Line::from(field_spans));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .title(title)
        .title_alignment(Alignment::Center);

    let modal = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(modal, modal_area);
}

pub fn render_delete_confirm(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let dialog_width = 50.min(area.width.saturating_sub(4));
    let dialog_height = 7;

    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    // Get node info for the dialog
    let (node_kind, node_id, node_name) = if !state.node_list.filtered_node_list.is_empty() {
        let item = &state.node_list.filtered_node_list[state.node_list.selected_index];
        (
            item.node_weight_kind.clone(),
            item.node_id.to_string(),
            item.name.clone().unwrap_or_default(),
        )
    } else {
        ("Unknown".to_string(), "".to_string(), "".to_string())
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Delete "),
            Span::styled(
                &node_kind,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            if !node_name.is_empty() {
                Span::styled(
                    format!(" \"{node_name}\""),
                    Style::default().fg(Color::Cyan),
                )
            } else {
                Span::raw("")
            },
            Span::raw(" and its edges?"),
        ]),
        Line::from(Span::styled(
            format!("  ID: {node_id}"),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[Y]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Yes  "),
            Span::styled(
                "[N]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" No / Esc"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .title("Delete Node and its edges?")
        .title_alignment(Alignment::Center);

    let dialog = Paragraph::new(lines).block(block);

    f.render_widget(dialog, dialog_area);
}

pub fn render_save_confirm(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 8;

    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Save Graph")
        .border_style(Style::default().fg(Color::Yellow));

    // Show the filename with a cursor if editing
    let filename_display = if state.save_modal.editing {
        format!("{}_", state.save_modal.filename)
    } else {
        state.save_modal.filename.clone()
    };

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Filename: ", Style::default().fg(Color::Gray)),
            Span::styled(
                filename_display,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to save, ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Esc",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to cancel", Style::default().fg(Color::Gray)),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, dialog_area);
}

pub fn render_edge_delete_confirm(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 8;

    let dialog_x = (area.width.saturating_sub(dialog_width)) / 2;
    let dialog_y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect::new(dialog_x, dialog_y, dialog_width, dialog_height);

    f.render_widget(Clear, dialog_area);

    // Get edge info for the dialog
    let (edge_kind, source_info, target_info) = if !state.node_list.filtered_node_list.is_empty() {
        let selected_item = &state.node_list.filtered_node_list[state.node_list.selected_index];
        let node_index = selected_item.index;

        // Collect edges
        let outgoing_edges: Vec<_> = state
            .working_graph
            .edges_directed(node_index, Direction::Outgoing)
            .collect();
        let incoming_edges: Vec<_> = state
            .working_graph
            .edges_directed(node_index, Direction::Incoming)
            .collect();
        let total_edges = outgoing_edges.len() + incoming_edges.len();

        if total_edges > 0 && state.edge_panel.selected_edge < total_edges {
            if state.edge_panel.selected_edge < outgoing_edges.len() {
                // Outgoing edge
                let edge = &outgoing_edges[state.edge_panel.selected_edge];
                let edge_kind = format!("{:?}", edge.weight().kind());
                let source_info = format!(
                    "{} ({})",
                    selected_item.node_weight_kind, selected_item.node_id
                );
                let target_info =
                    if let Ok(target_weight) = state.working_graph.get_node_weight(edge.target()) {
                        format!(
                            "{} ({})",
                            NodeWeightDiscriminants::from(target_weight),
                            target_weight.id()
                        )
                    } else {
                        "Unknown".to_string()
                    };
                (edge_kind, source_info, target_info)
            } else {
                // Incoming edge
                let incoming_idx = state.edge_panel.selected_edge - outgoing_edges.len();
                let edge = &incoming_edges[incoming_idx];
                let edge_kind = format!("{:?}", edge.weight().kind());
                let source_info =
                    if let Ok(source_weight) = state.working_graph.get_node_weight(edge.source()) {
                        format!(
                            "{} ({})",
                            NodeWeightDiscriminants::from(source_weight),
                            source_weight.id()
                        )
                    } else {
                        "Unknown".to_string()
                    };
                let target_info = format!(
                    "{} ({})",
                    selected_item.node_weight_kind, selected_item.node_id
                );
                (edge_kind, source_info, target_info)
            }
        } else {
            ("Unknown".to_string(), "".to_string(), "".to_string())
        }
    } else {
        ("Unknown".to_string(), "".to_string(), "".to_string())
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Delete edge: "),
            Span::styled(
                &edge_kind,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  From: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                truncate_string(&source_info, 40),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("  To:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                truncate_string(&target_info, 40),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                "[Y]",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Yes  "),
            Span::styled(
                "[N]",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" No / Esc"),
        ]),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .title("Delete Edge?")
        .title_alignment(Alignment::Center);

    let dialog = Paragraph::new(lines).block(block);

    f.render_widget(dialog, dialog_area);
}

pub fn render_add_edge_modal(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let modal_width = 80.min(area.width.saturating_sub(4));
    let modal_height = 22.min(area.height.saturating_sub(4));

    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    f.render_widget(Clear, modal_area);

    let add_state = &state.add_edge_modal;

    // Build the current edge kind for checking which fields to show
    let current_kind = SelectedEdgeWeightKind::from_index(
        add_state.edge_kind_index,
        if add_state.key.is_empty() {
            None
        } else {
            Some(add_state.key.clone())
        },
        add_state.is_default,
        add_state.path.clone(),
    );

    let mut lines = Vec::new();

    // Instructions
    if add_state.editing {
        lines.push(Line::from(Span::styled(
            "Type to filter | Tab/↑↓: Suggestions | Enter: Select | Esc: Cancel",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "↑/↓: Navigate | ←/→: Cycle kind | Enter: Edit | c: Create | Esc: Close",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // Error message
    if let Some(ref error) = add_state.error_message {
        lines.push(Line::from(Span::styled(
            format!("Error: {error}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )));
    } else {
        lines.push(Line::from(""));
    }

    // Source Node ID field
    let source_focused = add_state.focused_field == AddEdgeField::SourceNodeId;
    let source_editing = add_state.editing && source_focused;
    let source_value = if source_editing {
        format!("{}_", add_state.source_node_id)
    } else {
        add_state.source_node_id.clone()
    };
    lines.push(render_add_edge_node_field(
        "Source Node ID",
        &source_value,
        source_focused,
        source_editing,
        add_state.source_valid,
    ));

    // Source suggestions (only show when editing source field)
    if source_editing && !add_state.source_suggestions.is_empty() {
        for (i, &node_idx) in add_state.source_suggestions.iter().take(5).enumerate() {
            let node = &state.node_list.node_list[node_idx];
            let is_selected = i == add_state.source_suggestion_index;
            lines.push(render_suggestion_line(node, is_selected));
        }
        if add_state.source_suggestions.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!(
                    "    ... and {} more",
                    add_state.source_suggestions.len() - 5
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Target Node ID field
    let target_focused = add_state.focused_field == AddEdgeField::TargetNodeId;
    let target_editing = add_state.editing && target_focused;
    let target_value = if target_editing {
        format!("{}_", add_state.target_node_id)
    } else {
        add_state.target_node_id.clone()
    };
    lines.push(render_add_edge_node_field(
        "Target Node ID",
        &target_value,
        target_focused,
        target_editing,
        add_state.target_valid,
    ));

    // Target suggestions (only show when editing target field)
    if target_editing && !add_state.target_suggestions.is_empty() {
        for (i, &node_idx) in add_state.target_suggestions.iter().take(5).enumerate() {
            let node = &state.node_list.node_list[node_idx];
            let is_selected = i == add_state.target_suggestion_index;
            lines.push(render_suggestion_line(node, is_selected));
        }
        if add_state.target_suggestions.len() > 5 {
            lines.push(Line::from(Span::styled(
                format!(
                    "    ... and {} more",
                    add_state.target_suggestions.len() - 5
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }
    }

    // Edge Kind selector
    let kind_focused = add_state.focused_field == AddEdgeField::EdgeKind;
    let kind_name = SelectedEdgeWeightKind::ALL[add_state.edge_kind_index];
    lines.push(Line::from(vec![
        if kind_focused {
            Span::styled(
                "> ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("  ")
        },
        Span::styled(
            "Edge Kind: ",
            if kind_focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        if kind_focused {
            Span::styled(
                "◀ ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("  ")
        },
        Span::styled(
            kind_name,
            if kind_focused {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            },
        ),
        if kind_focused {
            Span::styled(
                " ▶",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        },
    ]));

    // Optional Key field (for Contain/Prototype)
    if current_kind.needs_key() {
        let key_focused = add_state.focused_field == AddEdgeField::Key;
        let key_value = if add_state.editing && key_focused {
            format!("{}_", add_state.key)
        } else if add_state.key.is_empty() {
            "(none)".to_string()
        } else {
            add_state.key.clone()
        };
        lines.push(render_add_edge_field(
            "Key (optional)",
            &key_value,
            key_focused,
            add_state.editing && key_focused,
        ));
    }

    // is_default field (for Use)
    if current_kind.needs_is_default() {
        let default_focused = add_state.focused_field == AddEdgeField::IsDefault;
        let checkbox = if add_state.is_default { "[x]" } else { "[ ]" };
        lines.push(Line::from(vec![
            if default_focused {
                Span::styled(
                    "> ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("  ")
            },
            Span::styled(
                "Is Default: ",
                if default_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
            Span::styled(
                checkbox,
                if default_focused {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
        ]));
    }

    // Path field (for ValueSubscription)
    if current_kind.needs_path() {
        let path_focused = add_state.focused_field == AddEdgeField::Path;
        let path_value = if add_state.editing && path_focused {
            format!("{}_", add_state.path)
        } else if add_state.path.is_empty() {
            "(e.g. /domain/foo/bar)".to_string()
        } else {
            add_state.path.clone()
        };
        lines.push(render_add_edge_field(
            "Path",
            &path_value,
            path_focused,
            add_state.editing && path_focused,
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  Press "),
        Span::styled(
            "'c'",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" to create the edge"),
    ]));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .title("Add Edge")
        .title_alignment(Alignment::Center);

    let modal = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(modal, modal_area);
}

pub fn render_stats_modal(f: &mut Frame, state: &AppState) {
    let area = f.area();
    let modal_width = 80.min(area.width.saturating_sub(4));
    let modal_height = 40.min(area.height.saturating_sub(4));

    let modal_x = (area.width.saturating_sub(modal_width)) / 2;
    let modal_y = (area.height.saturating_sub(modal_height)) / 2;

    let modal_area = Rect::new(modal_x, modal_y, modal_width, modal_height);

    f.render_widget(Clear, modal_area);

    let stats = &state.stats;
    let mut lines = Vec::new();

    // Summary section
    lines.push(Line::from(Span::styled(
        "Summary",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )));
    lines.push(Line::from(""));

    let total_nodes: usize = stats.node_counts.values().sum();
    let total_edges: usize = stats.edge_counts.values().sum();

    lines.push(Line::from(vec![
        Span::styled("  Total Nodes: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{total_nodes}"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({})", format_bytes(stats.total_node_bytes)),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Total Edges: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{total_edges}"),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({})", format_bytes(stats.total_edge_bytes)),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    lines.push(Line::from(vec![
        Span::styled("  Total Size:  ", Style::default().fg(Color::Gray)),
        Span::styled(
            format_bytes(stats.total_node_bytes + stats.total_edge_bytes),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));

    // Node weights section
    lines.push(Line::from(Span::styled(
        "Node Weights by Kind",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )));
    lines.push(Line::from(""));

    // Sort node kinds by byte size (descending)
    let mut node_kinds: Vec<_> = stats.node_counts.keys().collect();
    node_kinds.sort_by(|a, b| {
        let bytes_a = stats.node_bytes.get(*a).unwrap_or(&0);
        let bytes_b = stats.node_bytes.get(*b).unwrap_or(&0);
        bytes_b.cmp(bytes_a)
    });

    for kind in node_kinds {
        let count = stats.node_counts.get(kind).unwrap_or(&0);
        let bytes = stats.node_bytes.get(kind).unwrap_or(&0);
        let avg_bytes = if *count > 0 { bytes / count } else { 0 };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:30}", format!("{:?}", kind)),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(format!("{count:>6}"), Style::default().fg(Color::White)),
            Span::styled(" nodes  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>10}", format_bytes(*bytes)),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("  (avg: {})", format_bytes(avg_bytes)),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Edge weights section
    lines.push(Line::from(Span::styled(
        "Edge Weights by Kind",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
    )));
    lines.push(Line::from(""));

    // Sort edge kinds by byte size (descending)
    let mut edge_kinds: Vec<_> = stats.edge_counts.keys().collect();
    edge_kinds.sort_by(|a, b| {
        let bytes_a = stats.edge_bytes.get(*a).unwrap_or(&0);
        let bytes_b = stats.edge_bytes.get(*b).unwrap_or(&0);
        bytes_b.cmp(bytes_a)
    });

    for kind in edge_kinds {
        let count = stats.edge_counts.get(kind).unwrap_or(&0);
        let bytes = stats.edge_bytes.get(kind).unwrap_or(&0);
        let avg_bytes = if *count > 0 { bytes / count } else { 0 };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {:30}", format!("{:?}", kind)),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(format!("{count:>6}"), Style::default().fg(Color::White)),
            Span::styled(" edges  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{:>10}", format_bytes(*bytes)),
                Style::default().fg(Color::Green),
            ),
            Span::styled(
                format!("  (avg: {})", format_bytes(avg_bytes)),
                Style::default().fg(Color::DarkGray),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "↑/↓/j/k: Scroll | PgUp/PgDn: Fast scroll | Home: Top | Esc/Enter/q: Close",
        Style::default().fg(Color::DarkGray),
    )));

    let scroll = ScrollView::new(lines.len(), modal_area, state.stats_scroll);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .title("Snapshot Stats")
        .title_alignment(Alignment::Center);

    let modal = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll(scroll.scroll_tuple());

    f.render_widget(modal, modal_area);
}
