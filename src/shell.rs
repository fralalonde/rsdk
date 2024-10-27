use std::fs::OpenOptions;
use std::io;
use log::warn;
use crate::ARGS;
use std::io::Write;

pub fn set_var(name: &str, value: &str) -> io::Result<()> {
    // Get shell and envout settings from ARGS
    if let Some(shell) = &ARGS.get().unwrap().shell {
        if let Some(envout) = &ARGS.get().unwrap().envout {
            // Open the envout file for appending (without truncating existing content)
            let mut file = OpenOptions::new()
                .write(true)
                .append(true) // Use append mode to add new lines without overwriting
                .open(envout)
                .expect(&format!("Failed to create envout file {envout}"));

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