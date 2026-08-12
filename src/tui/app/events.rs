//! Key handling for the main loop, search mode, and modal interaction.

use std::sync::atomic::Ordering;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use rsdk::tool_version::ToolVersion;

use crate::tui::ModalState;

use super::{App, Pane};

impl App {
    pub(super) fn handle_events(&mut self) -> Result<()> {
        if !event::poll(std::time::Duration::from_millis(100))? {
            return Ok(());
        }
        let Event::Key(key) = event::read()? else {
            return Ok(());
        };
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }
        // Ctrl+Q quits from any state (search, modal, navigation).
        if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL {
            self.running = false;
            return Ok(());
        }
        if self.searching {
            self.handle_search_key(key)?;
        } else if self.modal.is_some() {
            self.handle_modal_key(key)?;
        } else {
            self.status_msg.clear();
            self.handle_key(key)?;
        }
        Ok(())
    }

    pub(super) fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.search.clear();
            }
            KeyCode::Char(c) if c.is_alphanumeric() => {
                self.search.push(c);
            }
            KeyCode::Backspace => {
                if self.search.is_empty() {
                    // Empty filter + backspace exits search mode.
                    self.searching = false;
                } else {
                    self.search.pop();
                }
            }
            // Let navigation, selection, and pane-switching keys work
            // while searching so the user can filter and move at once.
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Tab
            | KeyCode::Enter
            | KeyCode::Char('j')
            | KeyCode::Char('k') => {
                // Keep search active for arrows/PgUp/PgDn/Tab; only Enter
                // exits search (it's also the drill-in/select key).
                if matches!(key.code, KeyCode::Enter) {
                    self.searching = false;
                }
                self.status_msg.clear();
                self.handle_key(key)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char(c) if c.is_alphanumeric() => {
                self.searching = true;
                self.search = c.to_string();
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::PageUp => self.move_cursor(-10),
            KeyCode::PageDown => self.move_cursor(10),
            KeyCode::Enter | KeyCode::Right if !self.is_leaf_action() => self.enter()?,
            KeyCode::Esc => self.esc(),
            KeyCode::Left if self.screen_depth() > 0 => self.esc(),
            KeyCode::Tab => {
                self.active = match self.active {
                    Pane::Left => Pane::Right,
                    Pane::Right => Pane::Left,
                };
            }
            _ => {}
        }
        Ok(())
    }

    pub(super) fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
        let Some(modal) = self.modal.as_mut() else {
            return Ok(());
        };
        match modal {
            ModalState::Actions { state, items, .. } => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = state.selected().unwrap_or(0);
                    if i + 1 < items.len() {
                        state.select(Some(i + 1));
                    }
                }
                KeyCode::Enter => {
                    self.execute_modal_action()?;
                }
                KeyCode::Esc => {
                    self.modal = None;
                }
                _ => {}
            },
            ModalState::Installing { cancel, .. } => match key.code {
                KeyCode::Char('c') | KeyCode::Esc => {
                    cancel.store(true, Ordering::Relaxed);
                }
                _ => {}
            },
            ModalState::ConfirmDefault {
                state,
                tool,
                version,
            } => match key.code {
                KeyCode::Up | KeyCode::Char('k') | KeyCode::Left => {
                    let i = state.selected().unwrap_or(0);
                    state.select(Some(i.saturating_sub(1)));
                }
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Right => {
                    let i = state.selected().unwrap_or(0);
                    if i + 1 < 2 {
                        state.select(Some(i + 1));
                    }
                }
                KeyCode::Enter => {
                    let i = state.selected().unwrap_or(0);
                    let tool = tool.clone();
                    let version = version.clone();
                    if i == 0 {
                        // Yes — make default + current
                        let tv = ToolVersion::new(&self.rsdk_home, &tool, &version);
                        let _ = tv.make_default();
                        let _ = tv.make_current();
                        self.modal = Some(ModalState::Done {
                            msg: format!("✓ Installed {tool} {version} (default)"),
                            is_error: false,
                        });
                    } else {
                        // No — leave as-is
                        self.modal = Some(ModalState::Done {
                            msg: format!("✓ Installed {tool} {version}"),
                            is_error: false,
                        });
                    }
                }
                KeyCode::Esc => {
                    self.modal = Some(ModalState::Done {
                        msg: format!("✓ Installed {tool} {version}"),
                        is_error: false,
                    });
                }
                _ => {}
            },
            ModalState::Done { .. } => {
                self.modal = None;
                self.refresh_after_change()?;
            }
        }
        Ok(())
    }

    pub(super) fn execute_modal_action(&mut self) -> Result<()> {
        // Extract everything we need from the modal before borrowing mutably.
        let (tool, version, installed, idx) = match &self.modal {
            Some(ModalState::Actions {
                tool,
                version,
                installed,
                items,
                state,
            }) => {
                let i = state.selected().unwrap_or(0);
                let action = items.get(i).map(|it| it.name.clone());
                (tool.clone(), version.clone(), *installed, action)
            }
            _ => return Ok(()),
        };

        let Some(action) = idx else {
            return Ok(());
        };

        if installed {
            // Installed: Use / Set as default / Remove
            let tv = ToolVersion::new(&self.rsdk_home, &tool, &version);
            match action.as_str() {
                a if a.starts_with("Use") => {
                    tv.make_current()?;
                    self.modal = Some(ModalState::Done {
                        msg: format!("✓ Using {tool} {version}"),
                        is_error: false,
                    });
                }
                a if a.starts_with("Set") => {
                    tv.make_default()?;
                    self.modal = Some(ModalState::Done {
                        msg: format!("✓ Default {tool} = {version}"),
                        is_error: false,
                    });
                }
                a if a.starts_with("Remove") => {
                    tv.uninstall()?;
                    self.modal = Some(ModalState::Done {
                        msg: format!("✓ Removed {tool} {version}"),
                        is_error: false,
                    });
                }
                _ => {}
            }
        } else {
            // Not installed: spawn install worker
            self.spawn_install(&tool, &version);
        }
        Ok(())
    }
}
