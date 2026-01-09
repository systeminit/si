use std::{
    fs::File,
    io::Write,
    path::PathBuf,
};

use color_eyre::Result;
use crossterm::event::KeyCode;
use si_layer_cache::db::serialize;

use super::App;

impl App {
    pub(super) fn handle_save_confirm_keys(&mut self, key_code: KeyCode) {
        if self.state.save_modal.editing {
            match key_code {
                KeyCode::Enter => {
                    self.state.save_modal.editing = false;
                    if let Err(e) = self.save_graph() {
                        eprintln!("Failed to save graph: {e}");
                    }
                    self.state.active_modal = None;
                }
                KeyCode::Esc => {
                    self.state.active_modal = None;
                }
                KeyCode::Backspace => {
                    self.state.save_modal.filename.pop();
                }
                KeyCode::Char(c) => {
                    self.state.save_modal.filename.push(c);
                }
                _ => {}
            }
        } else {
            match key_code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    if let Err(e) = self.save_graph() {
                        eprintln!("Failed to save graph: {e}");
                    }
                    self.state.active_modal = None;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.state.active_modal = None;
                }
                KeyCode::Char('e') => {
                    self.state.save_modal.editing = true;
                }
                _ => {}
            }
        }
    }

    fn save_graph(&mut self) -> Result<()> {
        let save_path = if let Some(ref path) = self.state.save_path {
            let parent = path.parent().unwrap_or(path);
            parent.join(&self.state.save_modal.filename)
        } else {
            PathBuf::from(&self.state.save_modal.filename)
        };

        self.state.working_graph.cleanup_and_merkle_tree_hash()?;

        // Serialize the working graph
        let (graph_bytes, _) = serialize::to_vec(&self.state.working_graph)?;
        self.state.original_graph = self.state.working_graph.clone();

        // Write to file
        let mut file = File::create(&save_path)?;
        file.write_all(&graph_bytes)?;

        self.state.save_path = Some(save_path);
        self.state.is_dirty = false;
        self.state.pending_edits.clear();
        self.state.original_graph = self.state.working_graph.clone();

        self.state.success_message = Some("Graph saved successfully!".to_string());

        Ok(())
    }
}
