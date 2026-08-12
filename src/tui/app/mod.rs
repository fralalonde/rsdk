//! The TUI application: state, navigation, and the main event loop. The
//! `App` implementation is split across child modules: [`events`] (key
//! handling), [`install`] (install worker), [`data`] (loading), and
//! [`render`] (frame rendering).

pub(super) mod data;
pub(super) mod events;
pub(super) mod install;
pub(super) mod render;

use std::collections::HashMap;

use color_eyre::Result;
use ratatui::widgets::ListState;

use rsdk::rsdk_home::RsdkHome;
use rsdk::sdkman_client::SdkManClient;
use rsdk::tool_version::ToolVersion;

use crate::tui::{filter_items, Item, ModalState, Tui};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Pane {
    Left,
    Right,
}

pub(super) struct App {
    running: bool,
    active: Pane,
    search: String,
    searching: bool,
    status_msg: String,

    rsdk_home: RsdkHome,
    sdkman: SdkManClient,

    // Tools list (left pane)
    tools: Vec<Item>,
    tools_state: ListState,
    tool_descriptions: HashMap<String, Vec<String>>,
    tool_info: Vec<String>,

    // Versions list (right pane)
    versions: Vec<Item>,
    versions_state: ListState,

    // Modal
    modal: Option<ModalState>,
}

impl App {
    pub(super) fn new(rsdk_home: RsdkHome) -> Self {
        let sdkman = SdkManClient::new(&rsdk_home.cache());
        Self {
            running: true,
            active: Pane::Left,
            search: String::new(),
            searching: false,
            status_msg: String::new(),
            rsdk_home,
            sdkman,
            tools: Vec::new(),
            tools_state: ListState::default(),
            tool_descriptions: HashMap::new(),
            tool_info: Vec::new(),
            versions: Vec::new(),
            versions_state: ListState::default(),
            modal: None,
        }
    }

    pub(super) fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        self.load_tools()?;
        while self.running {
            // Poll install completion if modal is active
            self.check_install_done();
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    // ── Navigation & selection ─────────────────────────────────────────────
    fn is_leaf_action(&self) -> bool {
        // Selecting a version always opens the modal — never a leaf.
        false
    }

    pub(super) fn screen_depth(&self) -> usize {
        match self.active {
            Pane::Left => 0,
            Pane::Right => 1,
        }
    }

    pub(super) fn move_cursor(&mut self, delta: i32) {
        let len = self.active_items().len();
        if len == 0 {
            return;
        }
        let cur = self.active_state().selected().unwrap_or(0) as i32;
        let next = (cur + delta).clamp(0, len as i32 - 1) as usize;
        self.active_state_mut().select(Some(next));
        self.on_selection_changed();
    }

    fn on_selection_changed(&mut self) {
        if self.active == Pane::Left {
            self.update_tool_info();
        }
    }

    pub(super) fn active_items(&self) -> Vec<Item> {
        let q = &self.search;
        match self.active {
            Pane::Left => filter_items(&self.tools, q),
            Pane::Right => filter_items(&self.versions, q),
        }
    }

    pub(super) fn active_state(&self) -> &ListState {
        match self.active {
            Pane::Left => &self.tools_state,
            Pane::Right => &self.versions_state,
        }
    }

    pub(super) fn active_state_mut(&mut self) -> &mut ListState {
        match self.active {
            Pane::Left => &mut self.tools_state,
            Pane::Right => &mut self.versions_state,
        }
    }

    pub(super) fn enter(&mut self) -> Result<()> {
        match self.active {
            Pane::Left => {
                if let Some(tool) = self.selected_tool_name() {
                    self.load_versions(&tool)?;
                    self.active = Pane::Right;
                    self.search.clear();
                    self.searching = false;
                }
            }
            Pane::Right => {
                if let Some((tool, version)) = self.selected_version() {
                    let installed =
                        ToolVersion::new(&self.rsdk_home, &tool, &version).is_installed();
                    self.open_action_modal(&tool, &version, installed);
                }
            }
        }
        Ok(())
    }

    pub(super) fn esc(&mut self) {
        match self.active {
            Pane::Right => {
                self.active = Pane::Left;
                self.versions.clear();
                self.search.clear();
                self.searching = false;
                self.update_tool_info();
            }
            Pane::Left => self.running = false,
        }
    }

    pub(super) fn selected_tool_name(&self) -> Option<String> {
        let i = self.tools_state.selected()?;
        let items = filter_items(&self.tools, &self.search);
        items.get(i).map(|it| it.name.clone())
    }

    pub(super) fn selected_version(&self) -> Option<(String, String)> {
        let tool = self.selected_tool_name()?;
        let i = self.versions_state.selected()?;
        let items = filter_items(&self.versions, &self.search);
        let version = items.get(i)?.name.clone();
        Some((tool, version))
    }

    // ── Modal ──────────────────────────────────────────────────────────────
    pub(super) fn open_action_modal(&mut self, tool: &str, version: &str, installed: bool) {
        let items = if installed {
            vec![
                Item::new("Use (make current)", false),
                Item::new("Set as default", false),
                Item::new("Remove", false),
            ]
        } else {
            vec![Item::new("Install", false)]
        };
        let mut state = ListState::default();
        state.select(Some(0));
        self.modal = Some(ModalState::Actions {
            tool: tool.to_string(),
            version: version.to_string(),
            installed,
            items,
            state,
        });
    }

    pub(super) fn refresh_after_change(&mut self) -> Result<()> {
        // Preserve the user's tool selection across the reload so the
        // versions pane keeps its context after an action.
        let prev_tool = self.selected_tool_name();
        self.load_tools()?;
        if let Some(tool) = prev_tool {
            if let Some(i) = self.tools.iter().position(|t| t.name == tool) {
                self.tools_state.select(Some(i));
                self.update_tool_info();
            }
        }
        // Reload versions for the (restored) tool and stay on the
        // versions pane.
        if self.active == Pane::Right {
            if let Some(tool) = self.selected_tool_name() {
                self.load_versions(&tool)?;
            }
        }
        Ok(())
    }
}
