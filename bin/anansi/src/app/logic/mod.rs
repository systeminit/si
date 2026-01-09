mod add_edge;
mod delete;
mod edit_modal;
mod navigation;
mod save;
mod undo;

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{
    self,
    Event,
    KeyCode,
    KeyEvent,
    KeyModifiers,
    MouseButton,
    MouseEvent,
    MouseEventKind,
};
use dal::{
    WorkspaceSnapshotGraph,
    workspace_snapshot::node_weight::NodeWeight,
};
use ratatui::layout::{
    Constraint,
    Direction,
    Layout,
    Rect,
};

use super::{
    graph_edit::apply_graph_edit,
    helpers::{
        build_node_list,
        compute_snapshot_stats,
    },
    state::{
        ActiveModal,
        AppState,
        FocusPanel,
        GraphEdit,
    },
    terminal::{
        restore_terminal,
        setup_terminal,
    },
    ui,
};

pub struct App {
    state: AppState,
}

impl App {
    pub fn new(graph: WorkspaceSnapshotGraph) -> Result<Self> {
        let node_list = build_node_list(&graph)?;
        let stats = compute_snapshot_stats(&graph);
        let state = AppState::new(graph, node_list, None, stats);
        Ok(Self { state })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut terminal = setup_terminal()?;

        while !self.state.should_quit {
            terminal.draw(|f| {
                self.state.frame_size = f.area();
                ui::render(f, &self.state);
            })?;

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key_event(key)?,
                    Event::Mouse(mouse) => self.handle_mouse_event(mouse, self.state.frame_size),
                    _ => {}
                }
            }
        }

        restore_terminal(terminal)?;
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        self.state.success_message = None;

        if let Some(modal) = self.state.active_modal {
            match modal {
                ActiveModal::EditNode => self.handle_edit_modal_keys(key.code),
                ActiveModal::DeleteNodeConfirm => self.handle_delete_confirm_keys(key.code),
                ActiveModal::DeleteEdgeConfirm => self.handle_edge_delete_confirm_keys(key.code),
                ActiveModal::AddEdge => self.handle_add_edge_modal_keys(key.code),
                ActiveModal::SaveConfirm => self.handle_save_confirm_keys(key.code),
                ActiveModal::Stats => match key.code {
                    KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                        self.state.stats_scroll = 0;
                        self.state.active_modal = None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        self.state.stats_scroll = self.state.stats_scroll.saturating_sub(1);
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        self.state.stats_scroll = self.state.stats_scroll.saturating_add(1);
                    }
                    KeyCode::PageUp => {
                        self.state.stats_scroll = self.state.stats_scroll.saturating_sub(10);
                    }
                    KeyCode::PageDown => {
                        self.state.stats_scroll = self.state.stats_scroll.saturating_add(10);
                    }
                    KeyCode::Home => {
                        self.state.stats_scroll = 0;
                    }
                    _ => {}
                },
            }
            return Ok(());
        }

        if self.state.node_list.filter_mode {
            match key.code {
                KeyCode::Esc => {
                    self.state.node_list.filter_mode = false;
                }
                KeyCode::Enter => {
                    self.state.node_list.filter_mode = false;
                }
                KeyCode::Backspace => {
                    self.state.node_list.filter_text.pop();
                    self.state.node_list.update_filter();
                }
                KeyCode::Char(c) => {
                    self.state.node_list.filter_text.push(c);
                    self.state.node_list.update_filter();
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') => self.state.should_quit = true,
                KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    if self.state.is_dirty {
                        self.state.save_modal.filename =
                            if let Some(ref path) = self.state.save_path {
                                path.file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("workspace_snapshot.modified.bin")
                                    .to_string()
                            } else {
                                String::from("workspace_snapshot.modified.bin")
                            };
                        self.state.save_modal.editing = true;
                        self.state.active_modal = Some(ActiveModal::SaveConfirm);
                    }
                }
                KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.state.active_modal = Some(ActiveModal::Stats);
                }
                KeyCode::Tab => {
                    self.state.cycle_focus();
                }
                KeyCode::Esc => {
                    if !self.state.node_list.filter_text.is_empty() {
                        self.state.node_list.filter_text.clear();
                        self.state.node_list.update_filter();
                    } else {
                        self.state.should_quit = true;
                    }
                }
                KeyCode::Char('/') => {
                    self.state.node_list.filter_mode = true;
                }
                KeyCode::Char('e') => {
                    self.open_edit_modal();
                }
                KeyCode::Char('a') => {
                    self.open_add_edge_modal();
                }
                KeyCode::Char('u') => {
                    self.undo_last_edit();
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    if !self.state.node_list.filtered_node_list.is_empty() {
                        if self.state.focus == FocusPanel::EdgePanel {
                            self.state.active_modal = Some(ActiveModal::DeleteEdgeConfirm);
                        } else {
                            self.state.active_modal = Some(ActiveModal::DeleteNodeConfirm);
                        }
                    }
                }
                _ => match self.state.focus {
                    FocusPanel::NodeList => self.handle_node_list_keys(key.code),
                    FocusPanel::EdgePanel => self.handle_edge_panel_keys(key.code),
                    FocusPanel::NodeDetails => self.handle_node_details_keys(key.code),
                    FocusPanel::EditHistory => self.handle_edit_history_keys(key.code),
                },
            }
        }
        Ok(())
    }

    fn apply_edit_and_mark_dirty(
        &mut self,
        edit: &GraphEdit,
    ) -> Result<Option<NodeWeight>, &'static str> {
        let result = apply_graph_edit(&mut self.state.working_graph, edit);
        if result.is_ok() {
            self.state.mark_dirty();
        }
        result
    }

    fn handle_mouse_event(&mut self, mouse: MouseEvent, frame_size: Rect) {
        // Only handle left clicks, ignore if modal is open
        if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        if self.state.active_modal.is_some() {
            return;
        }

        let click_x = mouse.column;
        let click_y = mouse.row;

        // Calculate panel areas using the same layout as ui/mod.rs
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Title bar
                Constraint::Length(3),  // Filter input
                Constraint::Min(0),     // Main content
                Constraint::Length(20), // Edit history panel
                Constraint::Length(1),  // Status bar
            ])
            .split(frame_size);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Node list
                Constraint::Percentage(60), // Details panel
            ])
            .split(chunks[2]);

        // Details panel is split into node details and edges
        let details_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10), // Node details
                Constraint::Min(10), // Edges
            ])
            .split(main_chunks[1]);

        let filter_area = chunks[1];
        let node_list_area = main_chunks[0];
        let node_details_area = details_chunks[0];
        let edge_panel_area = details_chunks[1];
        let edit_history_area = chunks[3];

        // Determine which panel was clicked
        if Self::point_in_rect(click_x, click_y, filter_area) {
            self.state.node_list.filter_mode = true;
        } else if Self::point_in_rect(click_x, click_y, node_list_area) {
            self.state.focus = FocusPanel::NodeList;
        } else if Self::point_in_rect(click_x, click_y, node_details_area) {
            self.state.focus = FocusPanel::NodeDetails;
        } else if Self::point_in_rect(click_x, click_y, edge_panel_area) {
            self.state.focus = FocusPanel::EdgePanel;
        } else if Self::point_in_rect(click_x, click_y, edit_history_area) {
            self.state.focus = FocusPanel::EditHistory;
        }
    }

    fn point_in_rect(x: u16, y: u16, rect: Rect) -> bool {
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }
}
