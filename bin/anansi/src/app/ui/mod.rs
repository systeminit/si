mod helpers;
mod modals;
mod panels;
#[cfg(test)]
mod tests;

use modals::{
    render_add_edge_modal,
    render_delete_confirm,
    render_edge_delete_confirm,
    render_edit_modal,
    render_save_confirm,
    render_stats_modal,
};
use panels::{
    render_details_panel,
    render_edit_history,
    render_filter_input,
    render_node_list,
    render_status_bar,
    render_title,
};
use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction as LayoutDirection,
        Layout,
    },
};

use super::state::{
    ActiveModal,
    AppState,
};

pub fn render(f: &mut Frame, state: &AppState) {
    // Create layout with five sections
    let chunks = Layout::default()
        .direction(LayoutDirection::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Length(3),  // Filter input
            Constraint::Min(0),     // Main content
            Constraint::Length(20), // Edit history panel
            Constraint::Length(1),  // Status bar
        ])
        .split(f.area());

    render_title(f, chunks[0], state);
    render_filter_input(f, chunks[1], state);

    // Split main area into left (node list) and right (details)
    let main_chunks = Layout::default()
        .direction(LayoutDirection::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Node list
            Constraint::Percentage(60), // Details panel
        ])
        .split(chunks[2]);

    render_node_list(f, main_chunks[0], state);
    render_details_panel(f, main_chunks[1], state);
    render_edit_history(f, chunks[3], state);
    render_status_bar(f, chunks[4], state);

    // Render active modal on top if one is showing
    if let Some(modal) = state.active_modal {
        match modal {
            ActiveModal::EditNode => render_edit_modal(f, state),
            ActiveModal::DeleteNodeConfirm => render_delete_confirm(f, state),
            ActiveModal::DeleteEdgeConfirm => render_edge_delete_confirm(f, state),
            ActiveModal::AddEdge => render_add_edge_modal(f, state),
            ActiveModal::SaveConfirm => render_save_confirm(f, state),
            ActiveModal::Stats => render_stats_modal(f, state),
        }
    }
}
