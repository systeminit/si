use ratatui::{
    layout::Rect,
    style::{
        Color,
        Modifier,
        Style,
    },
    text::{
        Line,
        Span,
    },
};

use crate::app::state::NodeListItem;

/// A scroll adapter that handles scrolling logic for content that exceeds visible area.
/// This provides a unified way to handle scrolling across panels and modals.
#[derive(Debug, Clone, Copy)]
pub struct ScrollView {
    /// Total number of content lines
    pub content_height: u16,
    /// Height of the visible area (excluding borders)
    pub visible_height: u16,
    /// Current scroll offset
    pub scroll_offset: u16,
}

impl ScrollView {
    /// Create a new ScrollView from content lines and a rect area.
    /// Assumes 2 lines are used for borders (top + bottom).
    pub fn new(content_lines: usize, area: Rect, scroll_offset: u16) -> Self {
        let content_height = content_lines as u16;
        let visible_height = area.height.saturating_sub(2);
        Self {
            content_height,
            visible_height,
            scroll_offset,
        }
    }

    /// Create a ScrollView with explicit visible height (for custom border calculations)
    #[allow(dead_code)]
    pub fn with_visible_height(
        content_lines: usize,
        visible_height: u16,
        scroll_offset: u16,
    ) -> Self {
        Self {
            content_height: content_lines as u16,
            visible_height,
            scroll_offset,
        }
    }

    /// Calculate the maximum valid scroll offset
    pub fn max_scroll(&self) -> u16 {
        self.content_height.saturating_sub(self.visible_height)
    }

    /// Get the clamped scroll offset (never exceeds max_scroll)
    pub fn clamped_offset(&self) -> u16 {
        self.scroll_offset.min(self.max_scroll())
    }

    /// Get the scroll tuple for use with Paragraph::scroll()
    pub fn scroll_tuple(&self) -> (u16, u16) {
        (self.clamped_offset(), 0)
    }

    /// Check if content is scrollable (content exceeds visible area)
    #[allow(dead_code)]
    pub fn is_scrollable(&self) -> bool {
        self.content_height > self.visible_height
    }

    /// Calculate visible range for manual iteration (start_index, end_index)
    #[allow(dead_code)]
    pub fn visible_range(&self, total_items: usize) -> (usize, usize) {
        let offset = self.clamped_offset() as usize;
        let visible = self.visible_height as usize;
        let start = offset.min(total_items.saturating_sub(1));
        let end = (start + visible).min(total_items);
        (start, end)
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

pub fn format_bytes(bytes: usize) -> String {
    if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}

pub fn render_add_edge_field(
    label: &str,
    value: &str,
    focused: bool,
    editing: bool,
) -> Line<'static> {
    Line::from(vec![
        if focused {
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
            format!("{label}: "),
            if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(
            value.to_string(),
            if editing {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else if focused {
                Style::default().fg(Color::White)
            } else {
                Style::default()
            },
        ),
    ])
}

pub fn render_add_edge_node_field(
    label: &str,
    value: &str,
    focused: bool,
    editing: bool,
    valid: bool,
) -> Line<'static> {
    let validity_indicator = if value.is_empty() {
        Span::raw("")
    } else if valid {
        Span::styled(" ✓", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ✗", Style::default().fg(Color::Red))
    };

    Line::from(vec![
        if focused {
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
            format!("{label}: "),
            if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            },
        ),
        Span::styled(
            value.to_string(),
            if editing {
                Style::default().fg(Color::White).bg(Color::DarkGray)
            } else if focused {
                Style::default().fg(Color::White)
            } else {
                Style::default()
            },
        ),
        validity_indicator,
    ])
}

pub fn render_suggestion_line(node: &NodeListItem, is_selected: bool) -> Line<'static> {
    let prefix = if is_selected { "  → " } else { "    " };
    let name_part = node
        .name
        .as_ref()
        .map(|n| format!(" \"{n}\""))
        .unwrap_or_default();

    Line::from(vec![
        Span::styled(
            prefix,
            if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            },
        ),
        Span::styled(
            node.node_weight_kind.clone(),
            if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(
            name_part,
            if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::styled(
            format!(" ({})", truncate_string(&node.node_id.to_string(), 12)),
            if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
    ])
}
