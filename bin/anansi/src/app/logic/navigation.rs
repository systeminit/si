use crossterm::event::KeyCode;
use petgraph::{
    Direction,
    visit::EdgeRef,
};
use ratatui::layout::{
    Constraint,
    Direction as LayoutDirection,
    Layout,
};

use super::App;

impl App {
    // Node list navigation
    pub(super) fn move_selection_up(&mut self) {
        if self.state.node_list.selected_index > 0 {
            self.state.node_list.selected_index -= 1;
            self.adjust_scroll();
            self.state.edge_panel.selected_edge = 0;
            self.state.edge_panel.scroll_offset = 0;
            self.state.details.scroll_offset = 0;
        }
    }

    pub(super) fn move_selection_down(&mut self) {
        if !self.state.node_list.filtered_node_list.is_empty()
            && self.state.node_list.selected_index + 1
                < self.state.node_list.filtered_node_list.len()
        {
            self.state.node_list.selected_index += 1;
            self.adjust_scroll();
            self.state.edge_panel.selected_edge = 0;
            self.state.edge_panel.scroll_offset = 0;
            self.state.details.scroll_offset = 0;
        }
    }

    pub(super) fn page_up(&mut self) {
        self.state.node_list.selected_index =
            self.state.node_list.selected_index.saturating_sub(10);
        self.adjust_scroll();
        self.state.details.scroll_offset = 0;
    }

    pub(super) fn page_down(&mut self) {
        if !self.state.node_list.filtered_node_list.is_empty() {
            self.state.node_list.selected_index = (self.state.node_list.selected_index + 10)
                .min(self.state.node_list.filtered_node_list.len() - 1);
            self.adjust_scroll();
            self.state.details.scroll_offset = 0;
        }
    }

    pub(super) fn jump_to_start(&mut self) {
        self.state.node_list.selected_index = 0;
        self.adjust_scroll();
        self.state.details.scroll_offset = 0;
    }

    pub(super) fn jump_to_end(&mut self) {
        if !self.state.node_list.filtered_node_list.is_empty() {
            self.state.node_list.selected_index = self.state.node_list.filtered_node_list.len() - 1;
            self.adjust_scroll();
            self.state.details.scroll_offset = 0;
        }
    }

    pub(super) fn adjust_scroll(&mut self) {
        // Calculate the visible height based on the current frame size
        // Using the same layout as ui/mod.rs
        let visible_count = self.calculate_node_list_visible_height();

        if self.state.node_list.selected_index < self.state.node_list.scroll_offset {
            self.state.node_list.scroll_offset = self.state.node_list.selected_index;
        } else if self.state.node_list.selected_index
            >= self.state.node_list.scroll_offset + visible_count
        {
            self.state.node_list.scroll_offset =
                self.state.node_list.selected_index - visible_count + 1;
        }
    }

    /// Calculate the visible height of the node list panel based on frame size
    fn calculate_node_list_visible_height(&self) -> usize {
        if self.state.frame_size.height == 0 {
            return 20; // Default fallback
        }

        // Replicate layout from ui/mod.rs
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(1),  // Title bar
                Constraint::Length(3),  // Filter input
                Constraint::Min(0),     // Main content
                Constraint::Length(20), // Edit history panel
                Constraint::Length(1),  // Status bar
            ])
            .split(self.state.frame_size);

        let main_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Node list
                Constraint::Percentage(60), // Details panel
            ])
            .split(chunks[2]);

        // Node list area is main_chunks[0]
        // Subtract 4 for borders (2) + header (1) + spacing (1)
        // Ensure at least 1 to prevent math issues with empty range
        main_chunks[0].height.saturating_sub(4).max(1) as usize
    }

    // Edge panel navigation
    pub(super) fn move_edge_selection_up(&mut self) {
        if self.state.edge_panel.selected_edge > 0 {
            self.state.edge_panel.selected_edge -= 1;
            self.adjust_edge_scroll();
        }
    }

    pub(super) fn move_edge_selection_down(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
        let node_index = selected_item.index;

        let outgoing_count = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Outgoing)
            .count();
        let incoming_count = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Incoming)
            .count();
        let total_edges = outgoing_count + incoming_count;

        if self.state.edge_panel.selected_edge + 1 < total_edges {
            self.state.edge_panel.selected_edge += 1;
            self.adjust_edge_scroll();
        }
    }

    pub(super) fn page_edges_up(&mut self) {
        self.state.edge_panel.selected_edge =
            self.state.edge_panel.selected_edge.saturating_sub(10);
        self.adjust_edge_scroll();
    }

    pub(super) fn page_edges_down(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
        let node_index = selected_item.index;

        let outgoing_count = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Outgoing)
            .count();
        let incoming_count = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Incoming)
            .count();
        let total_edges = outgoing_count + incoming_count;

        if total_edges > 0 {
            self.state.edge_panel.selected_edge =
                (self.state.edge_panel.selected_edge + 10).min(total_edges - 1);
            self.adjust_edge_scroll();
        }
    }

    pub(super) fn jump_edges_to_start(&mut self) {
        self.state.edge_panel.selected_edge = 0;
        self.adjust_edge_scroll();
    }

    pub(super) fn jump_edges_to_end(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
        let node_index = selected_item.index;

        let outgoing_count = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Outgoing)
            .count();
        let incoming_count = self
            .state
            .working_graph
            .edges_directed(node_index, Direction::Incoming)
            .count();
        let total_edges = outgoing_count + incoming_count;

        if total_edges > 0 {
            self.state.edge_panel.selected_edge = total_edges - 1;
            self.adjust_edge_scroll();
        }
    }

    pub(super) fn adjust_edge_scroll(&mut self) {
        // Calculate the visible height based on the current frame size
        let visible_count = self.calculate_edge_panel_visible_height();

        if self.state.edge_panel.selected_edge < self.state.edge_panel.scroll_offset {
            self.state.edge_panel.scroll_offset = self.state.edge_panel.selected_edge;
        } else if self.state.edge_panel.selected_edge
            >= self.state.edge_panel.scroll_offset + visible_count
        {
            self.state.edge_panel.scroll_offset =
                self.state.edge_panel.selected_edge - visible_count + 1;
        }
    }

    /// Calculate the visible height of the edge panel based on frame size
    fn calculate_edge_panel_visible_height(&self) -> usize {
        if self.state.frame_size.height == 0 {
            return 10; // Default fallback
        }

        // Replicate layout from ui/mod.rs
        let chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(1),  // Title bar
                Constraint::Length(3),  // Filter input
                Constraint::Min(0),     // Main content
                Constraint::Length(20), // Edit history panel
                Constraint::Length(1),  // Status bar
            ])
            .split(self.state.frame_size);

        let main_chunks = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Node list
                Constraint::Percentage(60), // Details panel
            ])
            .split(chunks[2]);

        // Details panel is split into node details and edges
        let details_chunks = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Min(10), // Node details
                Constraint::Min(10), // Edges
            ])
            .split(main_chunks[1]);

        // Edge panel area is details_chunks[1]
        // Subtract 4 for borders (2) + headers (2)
        // Ensure at least 1 to prevent math issues with empty range
        details_chunks[1].height.saturating_sub(4).max(1) as usize
    }

    // Panel key handlers
    pub(super) fn handle_node_list_keys(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => self.move_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection_down(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Home | KeyCode::Char('g') => self.jump_to_start(),
            KeyCode::End | KeyCode::Char('G') => self.jump_to_end(),
            _ => {}
        }
    }

    pub(super) fn handle_node_details_keys(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.details.scroll_offset =
                    self.state.details.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.details.scroll_offset =
                    self.state.details.scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.state.details.scroll_offset =
                    self.state.details.scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.state.details.scroll_offset =
                    self.state.details.scroll_offset.saturating_add(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.details.scroll_offset = 0;
            }
            _ => {}
        }
    }

    pub(super) fn handle_edge_panel_keys(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => self.move_edge_selection_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_edge_selection_down(),
            KeyCode::PageUp => self.page_edges_up(),
            KeyCode::PageDown => self.page_edges_down(),
            KeyCode::Home | KeyCode::Char('g') => self.jump_edges_to_start(),
            KeyCode::End | KeyCode::Char('G') => self.jump_edges_to_end(),
            KeyCode::Enter => self.navigate_to_edge_target(),
            _ => {}
        }
    }

    pub(super) fn handle_edit_history_keys(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.edit_history.scroll_offset =
                    self.state.edit_history.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.edit_history.scroll_offset =
                    self.state.edit_history.scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.state.edit_history.scroll_offset =
                    self.state.edit_history.scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.state.edit_history.scroll_offset =
                    self.state.edit_history.scroll_offset.saturating_add(10);
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.state.edit_history.scroll_offset = 0;
            }
            _ => {}
        }
    }

    /// Navigate to the node connected by the selected edge.
    /// For outgoing edges, navigates to the target node.
    /// For incoming edges, navigates to the source node.
    pub(super) fn navigate_to_edge_target(&mut self) {
        if self.state.node_list.filtered_node_list.is_empty() {
            return;
        }

        let selected_item =
            &self.state.node_list.filtered_node_list[self.state.node_list.selected_index];
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

        // Determine the target node based on whether this is an outgoing or incoming edge
        let target_node_id = if selected_edge_idx < outgoing_count {
            // Outgoing edge: navigate to target
            let edge_ref = &outgoing_edges[selected_edge_idx];
            let target_idx = edge_ref.target();
            self.state
                .working_graph
                .get_node_weight(target_idx)
                .ok()
                .map(|w| w.id())
        } else {
            // Incoming edge: navigate to source
            let incoming_idx = selected_edge_idx - outgoing_count;
            let edge_ref = &incoming_edges[incoming_idx];
            let source_idx = edge_ref.source();
            self.state
                .working_graph
                .get_node_weight(source_idx)
                .ok()
                .map(|w| w.id())
        };

        let Some(target_node_id) = target_node_id else {
            return;
        };

        // Find the target node in the filtered list
        let target_position = self
            .state
            .node_list
            .filtered_node_list
            .iter()
            .position(|item| item.node_id == target_node_id);

        if let Some(new_index) = target_position {
            self.state.node_list.selected_index = new_index;
            self.adjust_scroll();
            // Reset edge selection for the new node
            self.state.edge_panel.selected_edge = 0;
            self.state.edge_panel.scroll_offset = 0;
            self.state.details.scroll_offset = 0;
            // Keep focus on edge panel (don't change self.state.focus)
        }
    }
}

#[cfg(test)]
mod tests {
    use dal::{
        PropKind,
        WorkspaceSnapshotGraph,
        workspace_snapshot::graph::WorkspaceSnapshotGraphVCurrent,
    };
    use ratatui::layout::Rect;
    use si_events::ContentHash;

    use crate::app::{
        helpers::{
            build_node_list,
            compute_snapshot_stats,
        },
        logic::App,
        state::AppState,
    };

    fn create_test_graph() -> WorkspaceSnapshotGraph {
        let inner = WorkspaceSnapshotGraphVCurrent::new_with_categories_only()
            .expect("Unable to create WorkspaceSnapshotGraph");
        WorkspaceSnapshotGraph::V4(inner)
    }

    fn create_test_app(graph: WorkspaceSnapshotGraph) -> App {
        let node_list = build_node_list(&graph).expect("Failed to build node list");
        let stats = compute_snapshot_stats(&graph);
        let state = AppState::new(graph, node_list, None, stats);
        App { state }
    }

    fn add_prop_node(graph: &mut WorkspaceSnapshotGraph, name: &str, kind: PropKind) {
        let id = graph.generate_ulid().expect("Unable to generate Ulid");
        let lineage = graph.generate_ulid().expect("Unable to generate Ulid");
        let node = dal::workspace_snapshot::node_weight::NodeWeight::new_prop(
            id,
            lineage,
            kind,
            name,
            ContentHash::new(format!("content_{name}").as_bytes()),
        );
        graph.add_or_replace_node(node).expect("Failed to add node");
    }

    #[test]
    fn test_adjust_scroll_keeps_selection_visible_for_small_terminal() {
        let mut graph = create_test_graph();

        // Add many nodes to ensure we have more than can fit in a small terminal
        for i in 0..50 {
            add_prop_node(&mut graph, &format!("prop_{i:02}"), PropKind::String);
        }

        let mut app = create_test_app(graph);

        // Set a small frame size that would only show a few items
        // With height 30: title(1) + filter(3) + main(5) + history(20) + status(1) = 30
        // The main area would be very small, showing only ~1-2 items after borders
        app.state.frame_size = Rect::new(0, 0, 100, 30);

        // Calculate the expected visible height
        let visible_height = app.calculate_node_list_visible_height();

        // Select an item that would be past the visible range
        let target_index = visible_height + 5;
        app.state.node_list.selected_index = target_index;
        app.state.node_list.scroll_offset = 0;

        // Call adjust_scroll
        app.adjust_scroll();

        // Verify the scroll_offset was adjusted so the selected item is visible
        // The selected_index should be within [scroll_offset, scroll_offset + visible_height)
        assert!(
            app.state.node_list.selected_index >= app.state.node_list.scroll_offset,
            "Selected index {} should be >= scroll offset {}",
            app.state.node_list.selected_index,
            app.state.node_list.scroll_offset
        );
        assert!(
            app.state.node_list.selected_index < app.state.node_list.scroll_offset + visible_height,
            "Selected index {} should be < scroll offset {} + visible height {}",
            app.state.node_list.selected_index,
            app.state.node_list.scroll_offset,
            visible_height
        );
    }

    #[test]
    fn test_adjust_scroll_scrolls_up_when_selection_above_viewport() {
        let mut graph = create_test_graph();

        for i in 0..50 {
            add_prop_node(&mut graph, &format!("prop_{i:02}"), PropKind::String);
        }

        let mut app = create_test_app(graph);
        app.state.frame_size = Rect::new(0, 0, 100, 40);

        // Set scroll offset past the selected index
        app.state.node_list.selected_index = 5;
        app.state.node_list.scroll_offset = 10;

        // Call adjust_scroll
        app.adjust_scroll();

        // Scroll offset should be adjusted down to make index 5 visible
        assert_eq!(
            app.state.node_list.scroll_offset, 5,
            "Scroll offset should be adjusted to match selected index when scrolling up"
        );
    }

    #[test]
    fn test_visible_height_changes_with_frame_size() {
        let graph = create_test_graph();
        let mut app = create_test_app(graph);

        // Test with a small frame
        app.state.frame_size = Rect::new(0, 0, 100, 30);
        let small_height = app.calculate_node_list_visible_height();

        // Test with a larger frame
        app.state.frame_size = Rect::new(0, 0, 100, 60);
        let large_height = app.calculate_node_list_visible_height();

        // The larger frame should have more visible items
        assert!(
            large_height > small_height,
            "Larger frame ({large_height}) should show more items than smaller frame ({small_height})"
        );
    }

    #[test]
    fn test_edge_panel_scroll_keeps_selection_visible() {
        let mut graph = create_test_graph();

        // Add a parent node with many edges
        let parent_id = graph.generate_ulid().expect("Unable to generate Ulid");
        let parent_lineage = graph.generate_ulid().expect("Unable to generate Ulid");
        let parent = dal::workspace_snapshot::node_weight::NodeWeight::new_prop(
            parent_id,
            parent_lineage,
            PropKind::Object,
            "parent",
            ContentHash::new(b"parent_content"),
        );
        let parent_idx = graph
            .add_or_replace_node(parent)
            .expect("Failed to add parent");

        // Add many child nodes and edges
        for i in 0..30 {
            let child_id = graph.generate_ulid().expect("Unable to generate Ulid");
            let child_lineage = graph.generate_ulid().expect("Unable to generate Ulid");
            let child = dal::workspace_snapshot::node_weight::NodeWeight::new_prop(
                child_id,
                child_lineage,
                PropKind::String,
                format!("child_{i:02}"),
                ContentHash::new(format!("child_content_{i}").as_bytes()),
            );
            let child_idx = graph
                .add_or_replace_node(child)
                .expect("Failed to add child");

            let edge = dal::workspace_snapshot::edge_weight::EdgeWeight::new(
                dal::workspace_snapshot::edge_weight::EdgeWeightKind::Contain(None),
            );
            graph
                .add_edge(parent_idx, edge, child_idx)
                .expect("Failed to add edge");
        }

        let mut app = create_test_app(graph);

        // Set a small frame size
        app.state.frame_size = Rect::new(0, 0, 100, 40);

        // Find and select the parent node
        for (i, item) in app.state.node_list.filtered_node_list.iter().enumerate() {
            if item.name.as_deref() == Some("parent") {
                app.state.node_list.selected_index = i;
                break;
            }
        }

        let visible_height = app.calculate_edge_panel_visible_height();

        // Select an edge past the visible range
        let target_edge = visible_height + 3;
        app.state.edge_panel.selected_edge = target_edge;
        app.state.edge_panel.scroll_offset = 0;

        // Call adjust_edge_scroll
        app.adjust_edge_scroll();

        // Verify the selected edge is now visible
        assert!(
            app.state.edge_panel.selected_edge >= app.state.edge_panel.scroll_offset,
            "Selected edge {} should be >= scroll offset {}",
            app.state.edge_panel.selected_edge,
            app.state.edge_panel.scroll_offset
        );
        assert!(
            app.state.edge_panel.selected_edge
                < app.state.edge_panel.scroll_offset + visible_height,
            "Selected edge {} should be < scroll offset {} + visible height {}",
            app.state.edge_panel.selected_edge,
            app.state.edge_panel.scroll_offset,
            visible_height
        );
    }
}
