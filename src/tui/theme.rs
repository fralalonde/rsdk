//! Color palette and shared widget styles for the TUI.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};

use crate::tui::Item;

pub(crate) const C_TITLE_FG: Color = Color::White;
pub(crate) const C_TITLE_BG: Color = Color::Blue;
pub(crate) const C_BORDER: Color = Color::DarkGray;
pub(crate) const C_BORDER_ACTIVE: Color = Color::Cyan;
pub(crate) const C_ACCENT: Color = Color::Cyan;
pub(crate) const C_STAR: Color = Color::Green;
pub(crate) const C_CURRENT: Color = Color::Yellow;
pub(crate) const C_DEFAULT: Color = Color::Magenta;
pub(crate) const C_STATUS_FG: Color = Color::White;
pub(crate) const C_STATUS_BG: Color = Color::DarkGray;
pub(crate) const C_SEARCH: Color = Color::Yellow;
pub(crate) const C_HIGHLIGHT_BG: Color = Color::Blue;
pub(crate) const C_INFO: Color = Color::LightCyan;
pub(crate) const C_DIM: Color = Color::Gray;
pub(crate) const C_MODAL_BORDER: Color = Color::Magenta;
pub(crate) const C_PROGRESS: Color = Color::Cyan;
pub(crate) const C_ERROR: Color = Color::Red;

pub(crate) fn highlight_style() -> Style {
    Style::default().bg(C_HIGHLIGHT_BG).fg(Color::White)
}

pub(crate) fn border_block(title: &str) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(C_BORDER))
}

pub(crate) fn border_block_active(title: &str) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(C_ACCENT)
                .bg(C_HIGHLIGHT_BG)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(
            Style::default()
                .fg(C_BORDER_ACTIVE)
                .add_modifier(Modifier::BOLD),
        )
}

pub(crate) fn item_line(item: &Item) -> Line<'_> {
    let mut spans: Vec<Span> = Vec::new();
    // Star marker for installed (green) or current/default
    if item.starred {
        spans.push(Span::styled(
            "* ",
            Style::default().fg(C_STAR).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("  "));
    }
    // Version name — colored if current or default
    let style = if item.is_current {
        Style::default().fg(C_CURRENT).add_modifier(Modifier::BOLD)
    } else if item.is_default {
        Style::default().fg(C_DEFAULT).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    spans.push(Span::styled(&item.name, style));
    // Suffix tags
    if item.is_current {
        spans.push(Span::styled(" (current)", Style::default().fg(C_CURRENT)));
    }
    if item.is_default {
        spans.push(Span::styled(" (default)", Style::default().fg(C_DEFAULT)));
    }
    Line::from(spans)
}
