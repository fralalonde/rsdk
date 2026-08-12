//! Modal state machine: action selection, install progress, post-install
//! confirmation, and transient done/error notices.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use color_eyre::Result;
use ratatui::widgets::ListState;

use rsdk::tool_version::ToolVersion;

use crate::tui::Item;

/// Action modal: pops when a version is selected.
#[derive(Debug, Clone)]
pub(crate) enum ModalState {
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
        done: InstallDone,
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
pub(crate) struct Progress {
    pub(crate) bytes: u64,
    pub(crate) total: u64,
}

impl Progress {
    pub(crate) fn new() -> Self {
        Self { bytes: 0, total: 0 }
    }
}

/// Result of an install worker: the installed `ToolVersion` plus whether it
/// was a new install (as opposed to already present).
pub(crate) type InstallResult = Result<(ToolVersion, bool)>;
/// Shared slot where the install worker parks its outcome for the UI thread.
pub(crate) type InstallDone = Arc<Mutex<Option<InstallResult>>>;
