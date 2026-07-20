use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem as RItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io::{self, Stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use rsdk::rsdk_home::RsdkHome;
use rsdk::sdkman_client::SdkManClient;
use rsdk::tool_version::ToolVersion;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

// ── Colors ──────────────────────────────────────────────────────────────────
const C_TITLE_FG: Color = Color::White;
const C_TITLE_BG: Color = Color::Blue;
const C_BORDER: Color = Color::DarkGray;
const C_BORDER_ACTIVE: Color = Color::Cyan;
const C_ACCENT: Color = Color::Cyan;
const C_STAR: Color = Color::Green;
const C_CURRENT: Color = Color::Yellow;
const C_DEFAULT: Color = Color::Magenta;
const C_STATUS_FG: Color = Color::White;
const C_STATUS_BG: Color = Color::DarkGray;
const C_SEARCH: Color = Color::Yellow;
const C_HIGHLIGHT_BG: Color = Color::Blue;
const C_INFO: Color = Color::LightCyan;
const C_DIM: Color = Color::Gray;
const C_MODAL_BORDER: Color = Color::Magenta;
const C_PROGRESS: Color = Color::Cyan;
const C_ERROR: Color = Color::Red;

fn highlight_style() -> Style {
    Style::default().bg(C_HIGHLIGHT_BG).fg(Color::White)
}

fn border_block(title: &str) -> Block<'_> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(C_ACCENT).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(C_BORDER))
}

fn border_block_active(title: &str) -> Block<'_> {
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
        .border_style(Style::default().fg(C_BORDER_ACTIVE).add_modifier(Modifier::BOLD))
}

fn item_line(item: &Item) -> Line<'_> {
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
        spans.push(Span::styled(
            " (current)",
            Style::default().fg(C_CURRENT),
        ));
    }
    if item.is_default {
        spans.push(Span::styled(
            " (default)",
            Style::default().fg(C_DEFAULT),
        ));
    }
    Line::from(spans)
}

// ── Data ────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
struct Item {
    name: String,
    starred: bool,
    /// True if this version is the active `current` symlink target.
    is_current: bool,
    /// True if this version is the `default` symlink target.
    is_default: bool,
}

impl Item {
    fn new(name: impl Into<String>, starred: bool) -> Self {
        Self {
            name: name.into(),
            starred,
            is_current: false,
            is_default: false,
        }
    }
}

/// Sort items: installed first (default → current → other installed, each by
/// version descending), then uninstalled (version descending).
fn sort_items(items: &mut [Item]) {
    items.sort_by(|a, b| {
        // Installed (starred) bubble to top.
        b.starred
            .cmp(&a.starred)
            // Within installed: default first, then current, then others.
            .then_with(|| {
                let rank = |i: &Item| match (i.is_default, i.is_current) {
                    (true, _) => 0,
                    (false, true) => 1,
                    (false, false) => 2,
                };
                rank(a).cmp(&rank(b))
            })
            // Within the same rank: version descending (latest first).
            .then_with(|| b.name.cmp(&a.name))
    });
}

fn filter_items(items: &[Item], query: &str) -> Vec<Item> {
    if query.is_empty() {
        return items.to_vec();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .filter(|i| i.name.to_lowercase().contains(&q))
        .cloned()
        .collect()
}

// ── Description parser ──────────────────────────────────────────────────────
fn parse_tool_descriptions(text: &str) -> HashMap<String, Vec<String>> {
    let mut out = HashMap::new();
    for entry in text.split("\n---") {
        let lines: Vec<&str> = entry.lines().collect();
        let mut install_line = None;
        let mut tool = None;
        for l in &lines {
            let trimmed = l.trim();
            if let Some(rest) = trimmed.strip_prefix("$ sdk install ") {
                install_line = Some(l);
                tool = Some(rest.trim().to_string());
                break;
            }
        }
        let Some(tool) = tool else { continue };
        let Some(install_idx) = install_line.and_then(|il| lines.iter().position(|&x| x == *il))
        else { continue };

        let header_idx = lines
            .iter()
            .position(|l| !l.trim().is_empty() && !l.trim_start().starts_with("---"))
            .unwrap_or(0);
        let header = lines.get(header_idx).copied().unwrap_or("");

        let body: Vec<String> = lines[header_idx + 1..install_idx]
            .iter()
            .map(|l| l.trim_end().to_string())
            .collect();
        let body: Vec<String> = body
            .iter()
            .rev()
            .skip_while(|l| l.trim().is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        let mut full = vec![header.trim().to_string()];
        full.extend(body);
        out.insert(tool, full);
    }
    out
}

// ── Modal state ─────────────────────────────────────────────────────────────
/// Action modal: pops when a version is selected.
#[derive(Debug, Clone)]
enum ModalState {
    /// List of actions to choose from.
    Actions {
        tool: String,
        version: String,
        installed: bool,
        items: Vec<Item>,
        state: ListState,
    },
    /// Install in progress; progress is shared with the worker thread.
    Installing {
        tool: String,
        version: String,
        progress: Arc<Mutex<Progress>>,
        cancel: Arc<AtomicBool>,
        done: Arc<Mutex<Option<Result<(ToolVersion, bool)>>>>,
    },
    /// Post-install: ask whether to make the new version the default
    /// (only shown when other versions are already installed).
    ConfirmDefault {
        tool: String,
        version: String,
        state: ListState,
    },
    /// Install finished — transient, dismissed by any key.
    Done { msg: String, is_error: bool },
}

#[derive(Debug, Clone)]
struct Progress {
    bytes: u64,
    total: u64,
}

impl Progress {
    fn new() -> Self {
        Self { bytes: 0, total: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Pane {
    Left,
    Right,
}

// ── App ─────────────────────────────────────────────────────────────────────
pub struct App {
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
    pub fn new(rsdk_home: RsdkHome) -> Self {
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

    pub fn run(&mut self, terminal: &mut Tui) -> Result<()> {
        self.load_tools()?;
        while self.running {
            // Poll install completion if modal is active
            self.check_install_done();
            terminal.draw(|f| self.render(f))?;
            self.handle_events()?;
        }
        Ok(())
    }

    // ── Events ──────────────────────────────────────────────────────────────
    fn handle_events(&mut self) -> Result<()> {
        if !event::poll(std::time::Duration::from_millis(100))? {
            return Ok(());
        }
        let Event::Key(key) = event::read()? else {
            return Ok(());
        };
        if key.kind != KeyEventKind::Press {
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

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.search.clear();
            }
            KeyCode::Enter => {
                self.searching = false;
            }
            KeyCode::Char(c) if c.is_alphanumeric() => {
                self.search.push(c);
            }
            KeyCode::Backspace => {
                self.search.pop();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
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

    fn is_leaf_action(&self) -> bool {
        // Selecting a version always opens the modal — never a leaf.
        false
    }

    fn screen_depth(&self) -> usize {
        match self.active {
            Pane::Left => 0,
            Pane::Right => 1,
        }
    }

    fn move_cursor(&mut self, delta: i32) {
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

    // ── Modal key handling ──────────────────────────────────────────────────
    fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
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
            ModalState::ConfirmDefault { state, tool, version } => {
                match key.code {
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
                }
            }
            ModalState::Done { .. } => {
                self.modal = None;
                self.refresh_after_change()?;
            }
        }
        Ok(())
    }

    fn execute_modal_action(&mut self) -> Result<()> {
        // Extract everything we need from the modal before borrowing mutably.
        let (tool, version, installed, idx) = match &self.modal {
            Some(ModalState::Actions { tool, version, installed, items, state }) => {
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

    fn spawn_install(&mut self, tool: &str, version: &str) {
        let progress = Arc::new(Mutex::new(Progress::new()));
        let cancel = Arc::new(AtomicBool::new(false));
        let done = Arc::new(Mutex::new(None));

        let home = self.rsdk_home.clone();
        let tool_owned = tool.to_string();
        let version_owned = version.to_string();
        let prog = Arc::clone(&progress);
        let can = Arc::clone(&cancel);
        let dn = Arc::clone(&done);
        thread::spawn(move || {
            let result = ToolVersion::install_monitored(
                &home,
                &tool_owned,
                &version_owned,
                &mut |bytes, total| {
                    if let Ok(mut p) = prog.lock() {
                        p.bytes = bytes;
                        p.total = total;
                    }
                },
                &can,
            );
            *dn.lock().unwrap() = Some(result);
        });

        self.modal = Some(ModalState::Installing {
            tool: tool.to_string(),
            version: version.to_string(),
            progress,
            cancel,
            done,
        });
    }

    fn check_install_done(&mut self) {
        let done_clone = match &self.modal {
            Some(ModalState::Installing { done, .. }) => Arc::clone(done),
            _ => return,
        };
        let result = done_clone.lock().unwrap().take();
        let Some(result) = result else { return };
        let (tool, version) = match &self.modal {
            Some(ModalState::Installing { tool, version, .. }) => (tool.clone(), version.clone()),
            _ => return,
        };
        match result {
            Ok((_tv, new)) => {
                if !new {
                    // Already installed — nothing to ask.
                    self.modal = Some(ModalState::Done {
                        msg: format!("{tool} {version} already installed"),
                        is_error: false,
                    });
                    return;
                }
                // Count other installed versions of this tool. If >1 (the one
                // we just installed plus at least one pre-existing), ask
                // whether to make it the default.
                let other_count = self
                    .rsdk_home
                    .installed_versions(&tool)
                    .map(|iter| iter.count())
                    .unwrap_or(0);
                if other_count > 1 {
                    let mut state = ListState::default();
                    state.select(Some(0));
                    self.modal = Some(ModalState::ConfirmDefault {
                        tool,
                        version,
                        state,
                    });
                } else {
                    // Only version — auto-make current + default.
                    let tv = ToolVersion::new(&self.rsdk_home, &tool, &version);
                    let _ = tv.make_default();
                    let _ = tv.make_current();
                    self.modal = Some(ModalState::Done {
                        msg: format!("✓ Installed {tool} {version} (default)"),
                        is_error: false,
                    });
                }
            }
            Err(e) => {
                let msg = e.to_string();
                self.modal = Some(ModalState::Done { msg, is_error: true });
            }
        }
    }

    fn refresh_after_change(&mut self) -> Result<()> {
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

    // ── Active list abstraction ─────────────────────────────────────────────
    fn active_items(&self) -> Vec<Item> {
        let q = &self.search;
        match self.active {
            Pane::Left => filter_items(&self.tools, q),
            Pane::Right => filter_items(&self.versions, q),
        }
    }

    fn active_state(&self) -> &ListState {
        match self.active {
            Pane::Left => &self.tools_state,
            Pane::Right => &self.versions_state,
        }
    }

    fn active_state_mut(&mut self) -> &mut ListState {
        match self.active {
            Pane::Left => &mut self.tools_state,
            Pane::Right => &mut self.versions_state,
        }
    }

    // ── Enter / Esc ─────────────────────────────────────────────────────────
    fn enter(&mut self) -> Result<()> {
        match self.active {
            Pane::Left => {
                if let Some(tool) = self.selected_tool_name() {
                    self.load_versions(&tool)?;
                    self.active = Pane::Right;
                }
            }
            Pane::Right => {
                if let Some((tool, version)) = self.selected_version() {
                    let installed = ToolVersion::new(&self.rsdk_home, &tool, &version).is_installed();
                    self.open_action_modal(&tool, &version, installed);
                }
            }
        }
        Ok(())
    }

    fn esc(&mut self) {
        match self.active {
            Pane::Right => {
                self.active = Pane::Left;
                self.versions.clear();
                self.update_tool_info();
            }
            Pane::Left => self.running = false,
        }
    }

    fn selected_tool_name(&self) -> Option<String> {
        let i = self.tools_state.selected()?;
        let items = filter_items(&self.tools, &self.search);
        items.get(i).map(|it| it.name.clone())
    }

    fn selected_version(&self) -> Option<(String, String)> {
        let tool = self.selected_tool_name()?;
        let i = self.versions_state.selected()?;
        let items = filter_items(&self.versions, &self.search);
        let version = items.get(i)?.name.clone();
        Some((tool, version))
    }

    // ── Data loading ────────────────────────────────────────────────────────
    fn load_tools(&mut self) -> Result<()> {
        let names = self.sdkman.get_tools()?;
        let installed: std::collections::HashSet<String> =
            self.rsdk_home.all_installed()?.map(|tv| tv.tool).collect();

        if let Ok(text) = self.sdkman.get_tools_list_text() {
            self.tool_descriptions = parse_tool_descriptions(&text);
        }

        self.tools = names
            .into_iter()
            .map(|n| {
                let starred = installed.contains(&n);
                Item::new(n, starred)
            })
            .collect();
        sort_items(&mut self.tools);
        self.tools_state.select(Some(0));
        self.update_tool_info();
        Ok(())
    }

    fn update_tool_info(&mut self) {
        let Some(i) = self.tools_state.selected() else {
            self.tool_info.clear();
            return;
        };
        let items = filter_items(&self.tools, &self.search);
        let Some(tool) = items.get(i) else {
            self.tool_info.clear();
            return;
        };

        let installed: Vec<String> = self
            .rsdk_home
            .installed_versions(&tool.name)
            .map(|iter| iter.map(|tv| tv.version).collect())
            .unwrap_or_default();

        let mut lines: Vec<String> = Vec::new();
        if let Some(desc) = self.tool_descriptions.get(&tool.name) {
            lines.extend(desc.iter().cloned());
            lines.push(String::new());
            lines.push("─".repeat(60));
            lines.push(String::new());
        }
        if installed.is_empty() {
            lines.push("No versions installed".to_string());
        } else {
            lines.push("Installed versions:".to_string());
            for v in &installed {
                lines.push(format!("  • {v}"));
            }
        }
        self.tool_info = lines;
    }

    fn load_versions(&mut self, tool: &str) -> Result<()> {
        let all = self.sdkman.get_tool_versions(tool)?;
        let installed: Vec<ToolVersion> = self
            .rsdk_home
            .installed_versions(tool)?
            .collect::<Vec<_>>();

        // Start from the API list, then append any installed versions that
        // are no longer advertised (e.g. older releases dropped upstream).
        let mut seen: std::collections::HashSet<String> = all.iter().cloned().collect();
        let mut versions: Vec<String> = all;
        for tv in &installed {
            if !seen.contains(&tv.version) {
                seen.insert(tv.version.clone());
                versions.push(tv.version.clone());
            }
        }

        self.versions = versions
            .into_iter()
            .map(|v| {
                let installed_tv = installed.iter().find(|tv| tv.version == v);
                let starred = installed_tv.is_some();
                let mut item = Item::new(v, starred);
                if let Some(tv) = installed_tv {
                    item.is_current = tv.is_current();
                    item.is_default = tv.is_default();
                }
                item
            })
            .collect();
        sort_items(&mut self.versions);
        self.versions_state.select(Some(0));
        Ok(())
    }

    // ── Modal ───────────────────────────────────────────────────────────────
    fn open_action_modal(&mut self, tool: &str, version: &str, installed: bool) {
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

    // ── Rendering ───────────────────────────────────────────────────────────
    fn render(&self, f: &mut Frame) {
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
            vec![Span::styled(
                &self.status_msg,
                Style::default().fg(C_STAR),
            )]
        } else {
            let pane = match self.active {
                Pane::Left => "L",
                Pane::Right => "R",
            };
            vec![
                Span::styled(format!("[{pane}] "), Style::default().fg(C_ACCENT)),
                Span::styled(
                    "↑↓ navigate  ←→ drill/back  Tab pane  Enter select  type-to-search  Esc quit",
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
            ModalState::Actions {
                items,
                state,
                ..
            } => {
                // Height = items + top/bottom border (2) + small padding (2).
                let h = (items.len() as u16) + 4;
                let area = centered_rect_fixed(40, h, f.area());
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
            ModalState::Installing { tool, version, progress, .. } => {
                let area = centered_rect(60, 25, f.area());
                f.render_widget(Clear, area);
                let block = Block::default()
                    .title(Span::styled(
                        format!(" Installing {tool} {version} "),
                        Style::default()
                            .fg(C_PROGRESS)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(C_PROGRESS));

                let p = progress.lock().map(|p| p.clone()).unwrap_or(Progress::new());
                let pct = if p.total > 0 {
                    (p.bytes * 100 / p.total) as u32
                } else {
                    0
                };
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
                f.render_widget(Paragraph::new(lines).block(block).alignment(Alignment::Center), area);
            }
            ModalState::ConfirmDefault { tool, version, state } => {
                let area = centered_rect(55, 30, f.area());
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
                let items = vec![
                    Item::new(format!("Yes — make {tool} {version} the default"), false),
                    Item::new("No — keep current default".to_string(), false),
                ];
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
                let area = centered_rect(50, 20, f.area());
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

// ── Helpers ─────────────────────────────────────────────────────────────────
/// Returns a centered rect of width% and height% of `r`.
fn centered_rect(width_pct: u16, height_pct: u16, r: Rect) -> Rect {
    let pop = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_pct) / 2),
            Constraint::Percentage(height_pct),
            Constraint::Percentage((100 - height_pct) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_pct) / 2),
            Constraint::Percentage(width_pct),
            Constraint::Percentage((100 - width_pct) / 2),
        ])
        .split(pop[1])[1]
}

/// Centered rect with absolute width (chars) and height (lines).
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let h = height.min(r.height);
    let w = width.min(r.width);
    let y = r.y + (r.height.saturating_sub(h)) / 2;
    let x = r.x + (r.width.saturating_sub(w)) / 2;
    Rect { x, y, width: w, height: h }
}

pub fn run(rsdk_home: RsdkHome) -> Result<()> {
    let mut terminal = init()?;
    let mut app = App::new(rsdk_home);
    let result = app.run(&mut terminal);
    if let Err(e) = restore() {
        eprintln!("Failed to restore terminal: {e}");
    }
    result
}
