use std::fs::OpenOptions;
use std::io;
use log::{debug, warn};
use crate::args;
use std::io::Write;

pub fn set_env_var_after_exit(name: &str, value: &str) -> io::Result<()> {
    if let Some(shell) = args::shell() {
        if let Some(envout) = args::envout() {
            let mut file = OpenOptions::new()
                .append(true)
                .open(&envout)?;

            debug!("setting {} to {}", name, value);

            // Emit shell-specific environment variable instructions
            let set_cmd = match shell.as_str() {
                "powershell" => format!("$env:{name} = '{value}'"),
                "bash" | "zsh" | "sh" => format!("export {name}=\"{value}\""),
                "fish" => format!("set -gx {name} '{value}'"),
                _ => {
                    warn!("Unsupported shell specified: {}", shell);
                    "".to_string()
                }
            };
            if args::debug() {
                writeln!(file, "echo eval: {set_cmd}")?;
            }
            writeln!(file, "{set_cmd}")?;
        } else {
            warn!("--shell specified but no --envout provided, env vars will not be set");
        }
    }
    Ok(())
}
