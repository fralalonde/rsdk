//! ratatui-based interactive browser for tools and versions — the `rsdk tui`
//! command. Split across submodules: [`theme`] (palette + widget styles),
//! [`list`] (list items), [`descriptions`] (SDKMAN list text parser),
//! [`modal`] (modal state), and [`app`] (the application itself).

mod app;
mod descriptions;
mod list;
mod modal;
mod theme;

// Private import (not a re-export): `App` is `pub(super)` in `app`, visible
// here as the parent module without widening its visibility.
use app::App;
pub(crate) use list::{filter_items, sort_items, Item};
pub(crate) use modal::{ModalState, Progress};

use std::io::{self, Stdout};

use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use rsdk::rsdk_home::RsdkHome;

/// The terminal backend used by the whole TUI.
pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

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

pub fn run(rsdk_home: RsdkHome) -> Result<()> {
    let mut terminal = init()?;
    let mut app = App::new(rsdk_home);
    let result = app.run(&mut terminal);
    if let Err(e) = restore() {
        eprintln!("Failed to restore terminal: {e}");
    }
    result
}
