use dal::{
    PropKind,
    Ulid,
    WorkspaceSnapshotGraph,
    workspace_snapshot::graph::WorkspaceSnapshotGraphVCurrent,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
};
use si_events::ContentHash;

use crate::app::{
    helpers::{
        build_node_list,
        compute_snapshot_stats,
    },
    state::{
        AppState,
        FocusPanel,
    },
    ui::render,
};

#[cfg(test)]
fn create_test_graph() -> WorkspaceSnapshotGraph {
    let inner = WorkspaceSnapshotGraphVCurrent::new_with_categories_only()
        .expect("Unable to create WorkspaceSnapshotGraph");
    WorkspaceSnapshotGraph::V4(inner)
}

#[cfg(test)]
fn generate_ulid(graph: &WorkspaceSnapshotGraph) -> Ulid {
    graph.generate_ulid().expect("Unable to generate Ulid")
}

#[cfg(test)]
fn create_test_state(graph: WorkspaceSnapshotGraph) -> AppState {
    let node_list = build_node_list(&graph).expect("Failed to build node list");
    let stats = compute_snapshot_stats(&graph);
    AppState::new(graph, node_list, None, stats)
}

#[cfg(test)]
fn render_to_string(state: &AppState, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");

    terminal.draw(|f| render(f, state)).expect("Failed to draw");

    let buffer = terminal.backend().buffer().clone();
    buffer_to_string(&buffer)
}

#[cfg(test)]
fn buffer_to_string(buffer: &Buffer) -> String {
    let mut result = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = buffer.cell((x, y)).unwrap();
            result.push_str(cell.symbol());
        }
        result.push('\n');
    }
    result
}

#[cfg(test)]
fn add_prop_node(graph: &mut WorkspaceSnapshotGraph, name: &str, kind: PropKind) -> Ulid {
    let id = generate_ulid(graph);
    let lineage = generate_ulid(graph);
    let node = dal::workspace_snapshot::node_weight::NodeWeight::new_prop(
        id,
        lineage,
        kind,
        name,
        ContentHash::new(format!("content_{name}").as_bytes()),
    );
    graph.add_or_replace_node(node).expect("Failed to add node");
    id
}

/// Redact dynamic content (ULIDs and timestamps) from output for stable snapshots
#[cfg(test)]
fn redact_dynamic_content(output: &str) -> String {
    use std::borrow::Cow;

    // Redact ULIDs - Crockford's Base32 (0-9, A-Z excluding I, L, O, U)
    // Match 6-26 char sequences, then filter to only those with uppercase letters
    // (to avoid matching pure digit sequences from hex hashes)
    let ulid_re = regex::Regex::new(r"[0-9A-HJKMNP-TV-Z]{6,26}").unwrap();
    let output = ulid_re.replace_all(output, |caps: &regex::Captures| {
        let matched = &caps[0];
        // Only redact if it contains at least one uppercase letter (real ULIDs always do)
        if matched.chars().any(|c| c.is_ascii_uppercase()) {
            "[ULID]".to_string()
        } else {
            matched.to_string()
        }
    });

    // Redact UTC timestamps from ULID display
    let timestamp_re =
        regex::Regex::new(r"UTC Timestamp in ULID: \d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").unwrap();
    let output: Cow<'_, str> =
        timestamp_re.replace_all(&output, "UTC Timestamp in ULID: [TIMESTAMP]");

    output.into_owned()
}

/// Get the snapshot path - works for both Cargo and Buck2 builds
#[cfg(test)]
#[allow(clippy::disallowed_methods)]
fn snapshot_path() -> String {
    // Try CARGO_MANIFEST_DIR first (Cargo builds)
    // In Cargo, this is an absolute path to the crate root
    // In Buck2 runtime, this might be "." or a relative path
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        // Only use CARGO_MANIFEST_DIR if it's an absolute path (Cargo builds)
        if dir.starts_with('/') {
            return format!("{dir}/src/app/ui/snapshots");
        }
    }
    // For Buck2 builds, the test file is already in src/app/ui/,
    // so just return the snapshots subdirectory relative to it
    "snapshots".to_string()
}

#[cfg(test)]
macro_rules! assert_ui_snapshot {
        ($name:expr, $output:expr) => {
            let redacted = redact_dynamic_content(&$output);
            insta::with_settings!({
                snapshot_path => snapshot_path(),
                prepend_module_to_snapshot => false,
            }, {
                insta::assert_snapshot!($name, redacted);
            });
        };
        ($output:expr) => {
            let redacted = redact_dynamic_content(&$output);
            insta::with_settings!({
                snapshot_path => snapshot_path(),
                prepend_module_to_snapshot => false,
            }, {
                insta::assert_snapshot!(redacted);
            });
        };
}

#[test]
fn test_empty_graph_renders() {
    let graph = create_test_graph();
    let state = create_test_state(graph);

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}

#[test]
fn test_node_list_with_props() {
    let mut graph = create_test_graph();

    // Add some prop nodes
    add_prop_node(&mut graph, "first_prop", PropKind::String);
    add_prop_node(&mut graph, "second_prop", PropKind::Integer);
    add_prop_node(&mut graph, "third_prop", PropKind::Boolean);

    let state = create_test_state(graph);

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}

#[test]
fn test_node_list_selection_changes_details() {
    let mut graph = create_test_graph();

    add_prop_node(&mut graph, "prop_one", PropKind::String);
    add_prop_node(&mut graph, "prop_two", PropKind::Object);

    let mut state = create_test_state(graph);

    // Select second node (index 1 after categories)
    // Categories come first, then our props
    let prop_count = state.node_list.filtered_node_list.len();
    if prop_count > 1 {
        state.node_list.selected_index = 1;
    }

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}

#[test]
fn test_node_list_scroll() {
    let mut graph = create_test_graph();

    // Add many prop nodes to force scrolling
    for i in 0..20 {
        add_prop_node(&mut graph, &format!("prop_{i:02}"), PropKind::String);
    }

    let mut state = create_test_state(graph);

    // Scroll down
    state.node_list.scroll_offset = 5;
    state.node_list.selected_index = 7;

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}

#[test]
fn test_focus_changes_panel_highlight() {
    let mut graph = create_test_graph();
    add_prop_node(&mut graph, "test_prop", PropKind::String);

    let mut state = create_test_state(graph);

    // Test NodeList focus
    state.focus = FocusPanel::NodeList;
    let output_node_list = render_to_string(&state, 100, 40);

    // Test NodeDetails focus
    state.focus = FocusPanel::NodeDetails;
    let output_details = render_to_string(&state, 100, 40);

    // Test EdgePanel focus
    state.focus = FocusPanel::EdgePanel;
    let output_edges = render_to_string(&state, 100, 40);

    // Test EditHistory focus
    state.focus = FocusPanel::EditHistory;
    let output_history = render_to_string(&state, 100, 40);

    assert_ui_snapshot!("focus_node_list", output_node_list);
    assert_ui_snapshot!("focus_details", output_details);
    assert_ui_snapshot!("focus_edges", output_edges);
    assert_ui_snapshot!("focus_history", output_history);
}

#[test]
fn test_filter_mode_display() {
    let mut graph = create_test_graph();
    add_prop_node(&mut graph, "filterable_prop", PropKind::String);

    let mut state = create_test_state(graph);

    // Enable filter mode
    state.node_list.filter_mode = true;
    state.node_list.filter_text = "prop".to_string();

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}

#[test]
fn test_filter_applied() {
    let mut graph = create_test_graph();

    add_prop_node(&mut graph, "alpha_prop", PropKind::String);
    add_prop_node(&mut graph, "beta_prop", PropKind::Integer);
    add_prop_node(&mut graph, "gamma_prop", PropKind::Boolean);

    let mut state = create_test_state(graph);

    // Apply filter
    state.node_list.filter_text = "alpha".to_string();
    state.node_list.update_filter();

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}

#[test]
fn test_edges_panel_with_edges() {
    let mut graph = create_test_graph();

    // Add two props and connect them
    let parent_id = add_prop_node(&mut graph, "parent", PropKind::Object);
    let child_id = add_prop_node(&mut graph, "child", PropKind::String);

    // Add edge
    let parent_idx = graph
        .get_node_index_by_id(parent_id)
        .expect("Failed to get parent index");
    let child_idx = graph
        .get_node_index_by_id(child_id)
        .expect("Failed to get child index");
    let edge = dal::workspace_snapshot::edge_weight::EdgeWeight::new(
        dal::workspace_snapshot::edge_weight::EdgeWeightKind::Contain(None),
    );
    graph
        .add_edge(parent_idx, edge, child_idx)
        .expect("Failed to add edge");

    let mut state = create_test_state(graph);

    // Find and select the parent node to see edges
    for (i, item) in state.node_list.filtered_node_list.iter().enumerate() {
        if item.name.as_deref() == Some("parent") {
            state.node_list.selected_index = i;
            break;
        }
    }

    state.focus = FocusPanel::EdgePanel;

    let output = render_to_string(&state, 100, 40);
    assert_ui_snapshot!(output);
}
