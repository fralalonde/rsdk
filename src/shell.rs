use std::fs::OpenOptions;
use std::{env, io};
use log::{debug, warn};
use crate::{args, tool_version};
use std::io::Write;
use std::path::PathBuf;
use anyhow::bail;

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

pub fn current_tool_version(tool: &str) -> anyhow::Result<String> {
    let env_home = tool_version::home_env(tool);
    let current_path = env::var(&env_home)
        .map(PathBuf::from)
        .map_err(|_| anyhow::anyhow!("No environment variable '{env_home}' found for tool '{tool}'"))?;

    if !current_path.exists() {
        bail!("Path '{:?}' (from env '{env_home}') does not exist", current_path);
    }
    if !current_path.is_dir() {
        bail!("Path '{:?}' (from env '{env_home}') is not a directory", current_path);
    }

    let version = current_path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Path '{:?}' (from env '{env_home}') is empty", current_path))?
        .to_string_lossy()
        .into_owned();

    Ok(version)
}