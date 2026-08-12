//! Frame rendering: title bar, tool/version panes, status line, and modals.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem as RItem, Paragraph};
use ratatui::Frame;

use crate::tui::theme::{
    border_block, border_block_active, highlight_style, item_line, C_ACCENT, C_BORDER, C_DIM,
    C_ERROR, C_INFO, C_MODAL_BORDER, C_PROGRESS, C_SEARCH, C_STAR, C_STATUS_BG, C_STATUS_FG,
    C_TITLE_BG, C_TITLE_FG,
};
use crate::tui::{filter_items, Item, ModalState, Progress};

use super::{App, Pane};

impl App {
    pub(super) fn render(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(f.area());

        self.render_title(f, chunks[0]);
        self.render_content(f, chunks[1]);
        self.render_status(f, chunks[2]);

        if self.modal.is_some() {
            self.render_modal(f);
        }
    }

    fn render_title(&self, f: &mut Frame, area: Rect) {
        let title = Line::from(vec![
            Span::styled(
                " rsdk ",
                Style::default()
                    .fg(C_TITLE_FG)
                    .bg(C_TITLE_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                "SDK Manager",
                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]);
        f.render_widget(Paragraph::new(title), area);
    }

    fn render_content(&self, f: &mut Frame, area: Rect) {
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        // Left: tools
        let left_block = if self.active == Pane::Left {
            border_block_active("Tools")
        } else {
            border_block("Tools")
        };
        let left_items = filter_items(&self.tools, &self.search);
        let left_list = List::new(
            left_items
                .iter()
                .map(|i| RItem::new(item_line(i)))
                .collect::<Vec<_>>(),
        )
        .block(left_block)
        .highlight_style(highlight_style());
        f.render_stateful_widget(left_list, panes[0], &mut self.tools_state.clone());

        // Right: info or versions
        if self.active == Pane::Right && !self.versions.is_empty() {
            // Versions
            let right_block = border_block_active("Versions");
            let right_items = filter_items(&self.versions, &self.search);
            let right_list = List::new(
                right_items
                    .iter()
                    .map(|i| RItem::new(item_line(i)))
                    .collect::<Vec<_>>(),
            )
            .block(right_block)
            .highlight_style(highlight_style());
            f.render_stateful_widget(right_list, panes[1], &mut self.versions_state.clone());
        } else {
            // Info pane
            let right_block = if self.active == Pane::Right {
                border_block_active("Details")
            } else {
                border_block("Details")
            };
            let lines: Vec<Line> = if self.tool_info.is_empty() {
                vec![Line::from(Span::styled(
                    "Select a tool to see details",
                    Style::default().fg(C_DIM),
                ))]
            } else {
                self.tool_info
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        if i == 0 {
                            Line::from(Span::styled(
                                s,
                                Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
                            ))
                        } else if s.starts_with("Installed") {
                            Line::from(Span::styled(s, Style::default().fg(C_INFO)))
                        } else if s.is_empty() || s.chars().all(|c| c == '─') {
                            Line::from(Span::styled(s, Style::default().fg(C_BORDER)))
                        } else {
                            Line::from(s.as_str())
                        }
                    })
                    .collect()
            };
            f.render_widget(Paragraph::new(lines).block(right_block), panes[1]);
        }
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let spans = if self.searching {
            vec![Span::styled(
                format!("/{}", self.search),
                Style::default().fg(C_SEARCH),
            )]
        } else if !self.status_msg.is_empty() {
            vec![Span::styled(&self.status_msg, Style::default().fg(C_STAR))]
        } else {
            let pane = match self.active {
                Pane::Left => "L",
                Pane::Right => "R",
            };
            vec![
                Span::styled(format!("[{pane}] "), Style::default().fg(C_ACCENT)),
                Span::styled(
                    "↑↓ navigate  ←→ drill/back  Tab pane  Enter select  type-to-search  Esc/Ctrl+Q quit",
                    Style::default().fg(C_STATUS_FG),
                ),
            ]
        };
        f.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(C_STATUS_BG)),
            area,
        );
    }

    // ── Modal rendering ─────────────────────────────────────────────────────
    fn render_modal(&self, f: &mut Frame) {
        let Some(modal) = &self.modal else { return };
        match modal {
            ModalState::Actions { items, state, .. } => {
                // Content-sized: width fits the longest action label, height
                // fits the item count (plus borders and a little breathing
                // room) so a single "Install" item doesn't open a huge box.
                let w = (items_line_width(items) + 6).clamp(16, 70);
                let h = items.len() as u16 + 2;
                let area = centered_rect_fixed(w, h, f.area());
                f.render_widget(Clear, area);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(C_MODAL_BORDER));
                let list = List::new(
                    items
                        .iter()
                        .map(|i| RItem::new(item_line(i)))
                        .collect::<Vec<_>>(),
                )
                .block(block)
                .highlight_style(highlight_style());
                f.render_stateful_widget(list, area, &mut state.clone());
            }
            ModalState::Installing {
                tool,
                version,
                progress,
                ..
            } => {
                // Content-sized: a fixed-width progress bar, a blank line, and
                // the cancel hint (the tool/version rides in the title).
                let area = centered_rect_fixed(48, 5, f.area());
                f.render_widget(Clear, area);
                let block = Block::default()
                    .title(Span::styled(
                        format!(" Installing {tool} {version} "),
                        Style::default().fg(C_PROGRESS).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(C_PROGRESS));

                let p = progress
                    .lock()
                    .map(|p| p.clone())
                    .unwrap_or(Progress::new());
                let pct = p
                    .bytes
                    .saturating_mul(100)
                    .checked_div(p.total)
                    .unwrap_or(0) as u32;

                // Inner width: area.width minus borders (2) minus padding (2)
                // minus brackets (2) and surrounding spaces (2).
                let bar_len = (area.width as u32).saturating_sub(8).max(10);
                let filled = if p.total > 0 {
                    (bar_len * pct / 100).min(bar_len) as usize
                } else {
                    0
                };
                let empty = (bar_len as usize).saturating_sub(filled);
                let bar: String = format!("[{}{}]", "#".repeat(filled), "-".repeat(empty));

                let lines = vec![
                    Line::from(Span::styled(
                        format!("{bar} {pct}%"),
                        Style::default().fg(C_PROGRESS),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Esc or c to cancel",
                        Style::default().fg(C_DIM),
                    )),
                ];
                f.render_widget(
                    Paragraph::new(lines)
                        .block(block)
                        .alignment(Alignment::Center),
                    area,
                );
            }
            ModalState::ConfirmDefault {
                tool,
                version,
                state,
            } => {
                let items = [
                    Item::new(format!("Yes — make {tool} {version} the default"), false),
                    Item::new("No — keep current default".to_string(), false),
                ];
                // Content-sized: width fits the longer option, height fits the
                // two options (plus borders and breathing room).
                let w = (items_line_width(&items) + 6).clamp(20, 80);
                let h = items.len() as u16 + 2;
                let area = centered_rect_fixed(w, h, f.area());
                f.render_widget(Clear, area);
                let block = Block::default()
                    .title(Span::styled(
                        " Make default? ",
                        Style::default()
                            .fg(C_MODAL_BORDER)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(C_MODAL_BORDER));
                let list = List::new(
                    items
                        .iter()
                        .map(|i| RItem::new(item_line(i)))
                        .collect::<Vec<_>>(),
                )
                .block(block)
                .highlight_style(highlight_style());
                f.render_stateful_widget(list, area, &mut state.clone());
            }
            ModalState::Done { msg, is_error } => {
                // Content-sized: width fits the message, height fits one line.
                let w = (msg.chars().count() as u16 + 6).clamp(16, 70);
                let h = 3;
                let area = centered_rect_fixed(w, h, f.area());
                f.render_widget(Clear, area);
                let color = if *is_error { C_ERROR } else { C_STAR };
                let block = Block::default()
                    .title(Span::styled(
                        " Done ",
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(color));
                f.render_widget(
                    Paragraph::new(msg.as_str())
                        .block(block)
                        .alignment(Alignment::Center),
                    area,
                );
            }
        }
    }
}

/// Longest rendered line width among `items`: the 2-char star/indent prefix,
/// the name, plus any ` (current)` / ` (default)` tags.
fn items_line_width(items: &[Item]) -> u16 {
    items
        .iter()
        .map(|i| {
            2 + i.name.chars().count() as u16
                + if i.is_current { 10 } else { 0 }
                + if i.is_default { 10 } else { 0 }
        })
        .max()
        .unwrap_or(0)
}

/// Centered rect with absolute width (chars) and height (lines).
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let h = height.min(r.height);
    let w = width.min(r.width);
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    Rect {
        x,
        y,
        width: w,
        height: h,
    }
}
