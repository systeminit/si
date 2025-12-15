use chrono::TimeZone;
use dal::{
    Ulid,
    workspace_snapshot::node_weight::NodeWeightDiscriminants,
};
use petgraph::{
    Direction,
    visit::EdgeRef,
};
use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction as LayoutDirection,
        Layout,
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
        Paragraph,
        Row,
        Table,
        Wrap,
    },
};

use super::helpers::ScrollView;
use crate::app::state::{
    AppState,
    FocusPanel,
};

pub fn render_title(f: &mut Frame, area: Rect, state: &AppState) {
    let mut title_text = String::from("anansi k(n)ows about nodes in system initiative");

    // Add save path and dirty indicator if available
    if let Some(ref path) = state.save_path {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        if state.is_dirty {
            title_text.push_str(&format!(" - {filename}*"));
        } else {
            title_text.push_str(&format!(" - {filename}"));
        }
    } else if state.is_dirty {
        title_text.push_str(" - [unsaved]");
    }

    let title = Paragraph::new(title_text).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, area);
}

pub fn render_filter_input(f: &mut Frame, area: Rect, state: &AppState) {
    let filter_display = if state.node_list.filter_mode {
        format!("Filter: {}_", state.node_list.filter_text)
    } else if !state.node_list.filter_text.is_empty() {
        format!(
            "Filter: {} (Press / to edit, Esc to clear)",
            state.node_list.filter_text
        )
    } else {
        "Press / to filter by node weight kind, ID, or name".to_string()
    };

    let style = if state.node_list.filter_mode {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else if !state.node_list.filter_text.is_empty() {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let filter_widget = Paragraph::new(filter_display)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title("Filter"));

    f.render_widget(filter_widget, area);
}

pub fn render_node_list(f: &mut Frame, area: Rect, state: &AppState) {
    if state.node_list.filtered_node_list.is_empty() {
        let empty_msg = if !state.node_list.filter_text.is_empty() {
            "No nodes match the filter"
        } else {
            "No nodes found"
        };
        let empty =
            Paragraph::new(empty_msg).block(Block::default().borders(Borders::ALL).title("Nodes"));
        f.render_widget(empty, area);
        return;
    }

    let total_items = state.node_list.filtered_node_list.len();
    // Visible height for data rows (subtract borders + header)
    let visible_height = area.height.saturating_sub(4).max(1) as usize;
    let selected_index = state
        .node_list
        .selected_index
        .min(total_items.saturating_sub(1));

    // Ensure selected item is visible by adjusting scroll offset
    let scroll_offset = if selected_index < state.node_list.scroll_offset {
        selected_index
    } else if selected_index >= state.node_list.scroll_offset + visible_height {
        selected_index.saturating_sub(visible_height - 1)
    } else {
        state.node_list.scroll_offset
    };

    let start = scroll_offset.min(total_items.saturating_sub(1));
    let end = (start + visible_height).min(total_items);

    // Build rows
    let rows: Vec<Row> = state.node_list.filtered_node_list[start..end]
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = start + i == selected_index;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                if is_selected { "[•]" } else { "[ ]" }.to_string(),
                item.node_id.to_string(),
                item.node_weight_kind.clone(),
                item.name.clone().unwrap_or_default(),
            ])
            .style(style)
        })
        .collect();

    // Create table
    let title = if !state.node_list.filter_text.is_empty() {
        format!(
            "Nodes ({}/{} | {}/{} total)",
            state.node_list.selected_index + 1,
            state.node_list.filtered_node_list.len(),
            state.node_list.filtered_node_list.len(),
            state.node_list.node_list.len()
        )
    } else {
        format!(
            "Nodes ({}/{})",
            state.node_list.selected_index + 1,
            state.node_list.filtered_node_list.len()
        )
    };

    let block = if state.focus == FocusPanel::NodeList {
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .title(title)
    } else {
        Block::default().borders(Borders::ALL).title(title)
    };

    let table = Table::new(
        rows,
        [
            Constraint::Length(3),  // Selection indicator
            Constraint::Length(26), // Node ID
            Constraint::Length(20), // Node weight kind
            Constraint::Min(20),    // Name (if available)
        ],
    )
    .header(
        Row::new(vec!["", "ID", "Node Weight Kind", "Name"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(block);

    f.render_widget(table, area);
}

pub fn render_details_panel(f: &mut Frame, area: Rect, state: &AppState) {
    if state.node_list.filtered_node_list.is_empty() {
        let empty = Paragraph::new("No node selected")
            .block(Block::default().borders(Borders::ALL).title("Details"));
        f.render_widget(empty, area);
        return;
    }

    let selected_item = &state.node_list.filtered_node_list[state.node_list.selected_index];

    // Get node weight - if error, show error message
    let node_weight = match state.working_graph.get_node_weight(selected_item.index) {
        Ok(nw) => nw,
        Err(e) => {
            let error_msg = Paragraph::new(format!("Error loading node: {e}"))
                .block(Block::default().borders(Borders::ALL).title("Details"));
            f.render_widget(error_msg, area);
            return;
        }
    };

    // Split details area into node info and edges
    let chunks = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Min(10), // Node details (expandable)
            Constraint::Min(10), // Edges list
        ])
        .split(area);

    // Render node details with all properties
    render_node_details(
        f,
        chunks[0],
        node_weight,
        &selected_item.node_weight_kind,
        state,
    );

    // Render edges
    render_edges(f, chunks[1], state, selected_item.index);
}

fn render_node_details(
    f: &mut Frame,
    area: Rect,
    node_weight: &dal::workspace_snapshot::node_weight::NodeWeight,
    node_weight_kind: &str,
    state: &AppState,
) {
    // Returns a utc formatted interpreation of the ulid timestamp
    fn timestamp_from_ulid(ulid: Ulid) -> String {
        let timestamp_ms = ulid.timestamp_ms() as i64;
        let timestamp = chrono::Utc.timestamp_millis_opt(timestamp_ms);
        match timestamp {
            chrono::offset::LocalResult::Single(time) => {
                time.format("%Y-%m-%d %H:%M:%S").to_string()
            }
            chrono::offset::LocalResult::Ambiguous(_, _) => "Ambiguous timestamp".into(),
            chrono::offset::LocalResult::None => "Unknown timestamp".into(),
        }
    }

    let mut details_text = vec![
        Line::from(vec![
            Span::styled(
                "Node Weight Kind: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(node_weight_kind),
        ]),
        Line::from(vec![
            Span::styled("ID: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("{}", node_weight.id())),
        ]),
        Line::from(vec![
            Span::styled(
                "UTC Timestamp in ULID: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(timestamp_from_ulid(node_weight.id()).to_string()),
        ]),
        Line::from(vec![
            Span::styled(
                "Lineage ID: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", node_weight.lineage_id())),
        ]),
        Line::from(vec![
            Span::styled(
                "Merkle Tree Hash: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", node_weight.merkle_tree_hash())),
        ]),
    ];

    // Add type-specific details using Debug formatting
    details_text.push(Line::from(""));

    // Format the debug output with syntax highlighting
    let debug_str = format!("{node_weight:#?}");
    for line in debug_str.lines() {
        let formatted_line =
            if line.trim_start().starts_with('}') || line.trim_start().starts_with(')') {
                Line::from(Span::styled(line, Style::default().fg(Color::DarkGray)))
            } else if line.contains(':') {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let indent = line.len() - line.trim_start().len();
                    let indent_str = " ".repeat(indent);
                    let field_name = parts[0].trim();
                    let field_value = parts[1].trim_start();

                    Line::from(vec![
                        Span::raw(indent_str),
                        Span::styled(
                            field_name,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(": "),
                        Span::styled(field_value, Style::default().fg(Color::White)),
                    ])
                } else {
                    Line::from(line)
                }
            } else {
                Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
            };

        details_text.push(formatted_line);
    }

    let block = if state.focus == FocusPanel::NodeDetails {
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .title("Node Details")
    } else {
        Block::default().borders(Borders::ALL).title("Node Details")
    };

    let scroll = ScrollView::new(details_text.len(), area, state.details.scroll_offset);

    let details = Paragraph::new(details_text)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll(scroll.scroll_tuple());

    f.render_widget(details, area);
}

fn render_edges(f: &mut Frame, area: Rect, state: &AppState, node_idx: petgraph::graph::NodeIndex) {
    // Get outgoing edges
    let outgoing_edges: Vec<_> = state
        .working_graph
        .edges_directed(node_idx, Direction::Outgoing)
        .collect();

    // Get incoming edges
    let incoming_edges: Vec<_> = state
        .working_graph
        .edges_directed(node_idx, Direction::Incoming)
        .collect();

    let total_edges = outgoing_edges.len() + incoming_edges.len();

    // Clamp selected edge to valid range
    let selected_edge = state
        .edge_panel
        .selected_edge
        .min(total_edges.saturating_sub(1));

    // Build all lines and track which line contains the selected edge
    let mut lines = Vec::new();
    let mut selected_line: Option<usize> = None;
    let mut edge_cursor = 0;

    // Outgoing section
    lines.push(Line::from(Span::styled(
        "Outgoing:",
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    )));

    if outgoing_edges.is_empty() {
        lines.push(Line::from("  (none)"));
    } else {
        for edge in outgoing_edges.iter() {
            let target = edge.target();
            let edge_kind = edge.weight().kind();

            if let Ok(target_weight) = state.working_graph.get_node_weight(target) {
                let is_selected =
                    state.focus == FocusPanel::EdgePanel && selected_edge == edge_cursor;

                if is_selected {
                    selected_line = Some(lines.len());
                }

                let text = format!(
                    " - {:?} → {} ({})",
                    edge_kind,
                    NodeWeightDiscriminants::from(target_weight),
                    target_weight.id(),
                );

                let line = if is_selected {
                    Line::from(vec![
                        Span::styled(
                            "[•] ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(text, Style::default().bg(Color::DarkGray).fg(Color::White)),
                    ])
                } else {
                    Line::from(format!("[ ] {text}"))
                };

                lines.push(line);
            }
            edge_cursor += 1;
        }
    }

    // Separator
    lines.push(Line::from(""));

    // Incoming section
    lines.push(Line::from(Span::styled(
        "Incoming:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));

    if incoming_edges.is_empty() {
        lines.push(Line::from("  (none)"));
    } else {
        for edge in incoming_edges.iter() {
            let source = edge.source();
            let edge_kind = edge.weight().kind();

            if let Ok(source_weight) = state.working_graph.get_node_weight(source) {
                let is_selected =
                    state.focus == FocusPanel::EdgePanel && selected_edge == edge_cursor;

                if is_selected {
                    selected_line = Some(lines.len());
                }

                let text = format!(
                    " ← {:?} - {} ({})",
                    edge_kind,
                    NodeWeightDiscriminants::from(source_weight),
                    source_weight.id(),
                );

                let line = if is_selected {
                    Line::from(vec![
                        Span::styled(
                            "[•] ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(text, Style::default().bg(Color::DarkGray).fg(Color::White)),
                    ])
                } else {
                    Line::from(format!("[ ] {text}"))
                };

                lines.push(line);
            }
            edge_cursor += 1;
        }
    }

    // Calculate scroll offset to keep selected line visible
    // visible_height is area minus borders (2 lines)
    let visible_height = area.height.saturating_sub(2).max(1) as usize;
    let scroll_offset = if let Some(sel_line) = selected_line {
        // Ensure selected line is visible
        let current_offset = state.edge_panel.scroll_offset;
        if sel_line < current_offset {
            sel_line
        } else if sel_line >= current_offset + visible_height {
            sel_line.saturating_sub(visible_height - 1)
        } else {
            current_offset
        }
    } else {
        state.edge_panel.scroll_offset
    };

    // Determine border style based on focus
    let title = if total_edges > 0 {
        format!(
            "Edges (out: {}, in: {}) [{}/{}]",
            outgoing_edges.len(),
            incoming_edges.len(),
            selected_edge + 1,
            total_edges
        )
    } else {
        format!(
            "Edges (out: {}, in: {})",
            outgoing_edges.len(),
            incoming_edges.len()
        )
    };

    let block = if state.focus == FocusPanel::EdgePanel {
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .title(title)
    } else {
        Block::default().borders(Borders::ALL).title(title)
    };

    let edges_widget = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_offset as u16, 0));

    f.render_widget(edges_widget, area);
}

pub fn render_edit_history(f: &mut Frame, area: Rect, state: &AppState) {
    let mut lines = Vec::new();

    if state.pending_edits.is_empty() {
        lines.push(Line::from(Span::styled(
            "No edits yet. Press 'e' to edit the selected node.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        // Show edits in reverse order (most recent first)
        for (i, pending_edit) in state.pending_edits.iter().rev().enumerate() {
            let edit_num = state.pending_edits.len() - i;
            let (old_value, new_value) = pending_edit.edit.display_values();

            // Header line with node info and operation type
            let op_type = pending_edit.edit.operation_type();
            let op_color = match op_type {
                "Delete" => Color::Red,
                "Add" => Color::Green,
                _ => Color::Yellow,
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{edit_num}. "),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("[{op_type}] "),
                    Style::default().fg(op_color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    pending_edit.edit.node_kind().to_string(),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    format!(" ({}) ", pending_edit.node_id),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            // Field change line
            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!("{}: ", pending_edit.edit.field_name()),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(old_value, Style::default().fg(Color::Red)),
                Span::styled(" → ", Style::default().fg(Color::DarkGray)),
                Span::styled(new_value, Style::default().fg(Color::Green)),
            ]));
        }
    }

    let title = if state.pending_edits.is_empty() {
        "Edit History".to_string()
    } else {
        format!("Edit History ({} edits)", state.pending_edits.len())
    };

    let block = if state.focus == FocusPanel::EditHistory {
        Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .title(title)
    } else {
        Block::default().borders(Borders::ALL).title(title)
    };

    let scroll = ScrollView::new(lines.len(), area, state.edit_history.scroll_offset);

    let edit_history = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll(scroll.scroll_tuple());

    f.render_widget(edit_history, area);
}

pub fn render_status_bar(f: &mut Frame, area: Rect, state: &AppState) {
    // If there's a success message, show it instead of the regular status
    if let Some(ref message) = state.success_message {
        let status = Paragraph::new(Span::styled(
            message,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
        f.render_widget(status, area);
        return;
    }

    let base_text = if state.node_list.filter_mode {
        "Enter: Apply filter | Esc: Cancel | Type to filter".to_string()
    } else {
        let save_hint = if state.is_dirty {
            "Ctrl+S: Save | "
        } else {
            ""
        };

        match state.focus {
            FocusPanel::NodeList => {
                format!(
                    "{save_hint}Tab: Panels | /: Filter | e: Edit | a: Add Edge | d: Delete | u: Undo | q: Quit",
                )
            }
            FocusPanel::NodeDetails => {
                format!(
                    "{save_hint}Tab: Panels | ↑/↓: Scroll | e: Edit | a: Add Edge | d: Delete | u: Undo | q: Quit",
                )
            }
            FocusPanel::EdgePanel => {
                format!(
                    "{save_hint}Tab: Panels | ↑/↓: Select | a: Add Edge | d: Delete | u: Undo | q: Quit",
                )
            }
            FocusPanel::EditHistory => {
                format!(
                    "{save_hint}Tab: Panels | e: Edit | a: Add Edge | d: Delete | u: Undo | q: Quit",
                )
            }
        }
    };

    let mut status_spans = vec![];

    // Add dirty indicator if the graph has been modified
    if state.is_dirty {
        status_spans.push(Span::styled(
            "[Modified] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }

    status_spans.push(Span::styled(base_text, Style::default().fg(Color::Gray)));

    let status = Paragraph::new(Line::from(status_spans));

    f.render_widget(status, area);
}
