//! Background install worker: progress/cancel plumbing and completion
//! polling from the UI thread.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::thread;

use ratatui::widgets::ListState;

use rsdk::tool_version::ToolVersion;

use crate::tui::{ModalState, Progress};

use super::App;

impl App {
    pub(super) fn spawn_install(&mut self, tool: &str, version: &str) {
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

    pub(super) fn check_install_done(&mut self) {
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
                self.modal = Some(ModalState::Done {
                    msg,
                    is_error: true,
                });
            }
        }
    }
}
