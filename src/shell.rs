use std::fs::OpenOptions;
use std::io;
use log::warn;
use crate::{args};
use std::io::Write;

pub fn set_var(name: &str, value: &str) -> io::Result<()> {
    // Get shell and envout settings from ARGS
    if let Some(shell) = args::shell() {
        if let Some(envout) = args::envout() {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&envout)?;

            // Emit shell-specific environment variable instructions
            match shell.as_str() {
                "powershell" => {
                    writeln!(file, "$env:{} = '{}'", name, value)?;
                }
                "bash" | "zsh" => {
                    writeln!(file, "export {}=\"{}\"", name, value)?;
                }
                "fish" => {
                    writeln!(file, "set -x {} '{}'", name, value)?;
                }
                _ => {
                    warn!("Unsupported shell specified: {}", shell);
                }
            }
        } else {
            warn!("--shell specified but no --envout provided");
        }
    }
    Ok(())
}