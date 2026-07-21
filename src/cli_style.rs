//! Minimal CLI styling mirroring the TUI palette (`src/tui.rs`), built on
//! the popular `colored` crate so call sites stay terse (e.g.
//! `cli_style::star("*")`, `cli_style::accent(&tool)`).
//!
//! Colors match the TUI: cyan accent, green star, yellow current,
//! magenta default, red error, dim gray.
//!
//! `colored` honors `NO_COLOR` and non-TTY output automatically (it disables
//! itself unless stdout is a terminal and NO_COLOR is unset), so the helpers
//! need no extra guards. Helpers return `String` so they compose freely with
//! plain strings in `format!`/`println!` and ternaries.

use colored::{Color, Colorize};

pub fn accent(s: &str) -> String {
    s.cyan().to_string()
}

pub fn star(s: &str) -> String {
    s.green().to_string()
}

/// Current version / marker (yellow, like C_CURRENT in the TUI).
pub fn current(s: &str) -> String {
    s.yellow().to_string()
}

/// Default tool/version (magenta, like C_DEFAULT in the TUI).
pub fn default_(s: &str) -> String {
    s.magenta().to_string()
}

pub fn error(s: &str) -> String {
    s.red().to_string()
}

/// Muted helper text (gray).
pub fn dim(s: &str) -> String {
    s.truecolor(120, 120, 120).to_string()
}

/// Informational label (e.g. "Flushing cache", "Tool").
pub fn info(s: &str) -> String {
    s.color(Color::Cyan).to_string()
}
