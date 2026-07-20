mod tui;

use std::{env, fs, io};
use std::io::Write;
use eyre::bail;
use clap::{CommandFactory, Parser};
use log::debug;
use rsdk::args::{Cli, Command, EnvSubcommand, ARGS};
use rsdk::{args, rcfile, rsdk_home, shell, sdkman_client, tool_version::ToolVersion};

const RUST_LOG: &str = "RUST_LOG";
const RUST_BACKTRACE: &str = "RUST_BACKTRACE";

#[allow(clippy::collapsible_else_if)]
fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    let _ = ARGS.set(cli.clone());

    if args::debug() {
        env::set_var(RUST_LOG, "debug");
        env::set_var(RUST_BACKTRACE, "1");
    } else if env::var(RUST_LOG).is_err() {
        env::set_var(RUST_LOG, "info");
    }

    env_logger::init();

    let rsdk_home = rsdk_home::RsdkHome::new()?;

    if let Some(command) = &cli.command {
        match command {
            Command::Init => {
                let default_tools = rsdk_home.all_defaults()?;
                let mut paths = vec![];

                // Add each default tool's stable `current/bin` to PATH. The
                // `current` symlink is what `use` / `env` flip, so PATH never
                // needs to be rewritten after this (same model as SDKMAN).
                for default_version in default_tools {
                    let current_bin = default_version.path().parent()
                        .map(|tool_dir| tool_dir.join("current").join("bin"))
                        .expect("tool version path has a tool dir parent");
                    paths.push(current_bin);
                    debug!("setting env var {:?} to {:?}", default_version.home(), default_version.path());
                    shell::set_env_var_after_exit(&default_version.home(), &default_version.path().to_string_lossy())?;
                }

                let path = env::var_os("PATH").unwrap_or_default();
                env::split_paths(&path)
                    // drop any rsdk-managed tool dirs already on PATH to avoid duplicates
                    .filter(|p| !p.starts_with(rsdk_home.tools()))
                    .for_each(|p| paths.push(p));

                let new_path = env::join_paths(paths)?;
                debug!("updating PATH to {:?}", new_path);
                shell::set_env_var_after_exit("PATH", &new_path.to_string_lossy())?;
            }
            Command::Install { tool, version, default } => {
                let (tv, new_install) = ToolVersion::install(&rsdk_home, tool, version)?;
                if !new_install {
                    println!("Tool {} version {} was already installed", tv.tool, tv.version);
                }

                let vv: Vec<_> = rsdk_home.installed_versions(tool)?.collect();
                match vv.len() {
                    0 => panic!("just installed {} version {} yet no versions detected?!", tv.tool, tv.version),
                    1 => {
                        tv.make_default()?;
                        tv.make_current()?;
                    }
                    _ => {
                        if *default || ask(&format!("Do you want to make {tool} {} the new default? (Y/n): ", tv.version), true) {
                            tv.make_default()?;
                            tv.make_current()?;
                        }
                    }
                }
                if new_install {
                    println!("Installed {} {}", tv.tool, tv.version);
                }
            }
            Command::Uninstall { tool, version } | Command::Remove { tool, version } => {
                let cv = ToolVersion::new(&rsdk_home, tool, version);

                let was_default = cv.is_default();
                let was_current = cv.is_current();

                if was_default {
                    debug!("deleting default symlink to deleted version");
                    fs::remove_file(rsdk_home.default_symlink_path(tool))?;
                }

                cv.uninstall()?;
                println!("Uninstalled {tool} {version}");

                let vv: Vec<_> = rsdk_home.installed_versions(tool)?.collect();
                match vv.len() {
                    0 => {
                        debug!("deleted last tool version, deleting tool dir too");
                        fs::remove_dir_all(rsdk_home.tool_dir(tool))?
                    }
                    _ => {
                        let new_cv = &vv[0];
                        if was_default {
                            new_cv.make_default()?;
                        }
                        if was_current {
                            new_cv.make_current()?;
                        }
                        if was_default {
                            println!("{} version {} is the new default", new_cv.tool, new_cv.version)
                        }
                    }
                }
            }
            Command::List { tool } => {
                let api = sdkman_client::SdkManClient::new(&rsdk_home.cache());
                if let Some(tool) = tool {
                    println!("{}", api.get_tool_versions_text(tool)?)
                } else {
                    println!("{}", api.get_tools_list_text()?)
                }
            }
            Command::Installed { tool } => {
                let mut installed: Vec<ToolVersion> = rsdk_home
                    .all_installed()?
                    .filter(|tv| tool.as_ref().is_none_or(|t| tv.tool.eq(t)))
                    .collect();
                installed.sort_by(|a, b| a.tool.cmp(&b.tool).then(a.version.cmp(&b.version)));

                // Mark the current version with `*` and align the version column
                // by padding tool names to the widest (like sdkman).
                let width = installed.iter().map(|tv| tv.tool.len()).max().unwrap_or(0);
                for tv in &installed {
                    let marker = if tv.is_current() { "*" } else { " " };
                    println!("{marker} {:width$} {}", tv.tool, tv.version, width = width);
                }
            }
            Command::Env { command } => {
                if let Some(command) = command {
                    match command {
                        EnvSubcommand::Init => rcfile::env_init(&rsdk_home)?,
                        EnvSubcommand::Install => rcfile::env_install(&rsdk_home)?,
                        EnvSubcommand::Clear => rcfile::env_clear(&rsdk_home)?,
                    }
                } else {
                    rcfile::env_apply(&rsdk_home)?;
                }
            }
            Command::Default { tool, version } => {
                if let Some(tool) = tool {
                    if let Some(version) = version {
                        let cv = ToolVersion::new(&rsdk_home, tool, version);
                        if cv.is_installed() {
                            cv.make_default()?
                        } else {
                            bail!("tool '{cv}' is not installed")
                        }
                    } else {
                        if let Some(version) = rsdk_home.default_version(tool)? {
                            println!("{version}");
                        } else {
                            bail!("no default version set for tool '{}'", tool);
                        }
                    }
                } else {
                    for tv in rsdk_home.all_installed()? {
                        if tv.is_default() {
                            println!("{tv}");
                        }
                    }
                }
            }
            Command::Current { tool } => {
                if let Some(tool) = tool {
                    match current_tv(&rsdk_home, tool)? {
                        Some(tv) => println!("{tv}"),
                        None => bail!("no current version of tool '{}'", tool),
                    }
                } else {
                    for tool in installed_tools(&rsdk_home)? {
                        if let Some(tv) = current_tv(&rsdk_home, &tool)? {
                            println!("{tv}");
                        }
                    }
                }
            }
            Command::Use { tool, version } => {
                if let Some(version) = version {
                    let tv = ToolVersion::new(&rsdk_home, tool, version);
                    if tv.is_installed() {
                        tv.make_current()?;
                    } else {
                        // SDKMAN offers to install a missing version on `use`.
                        if ask(&format!("{tool} {version} is not installed, install it now? (Y/n): "), true) {
                            let (tv, _) = ToolVersion::install(&rsdk_home, tool, &Some(version.clone()))?;
                            tv.make_current()?;
                            println!("Installed {} {}", tv.tool, tv.version);
                        } else {
                            eprintln!("'{tool} {version}' is not installed");
                        }
                    }
                } else {
                    match current_tv(&rsdk_home, tool)? {
                        Some(tv) => println!("{tv}"),
                        None => bail!("no current version of tool '{}'", tool),
                    }
                }
            }
            Command::Flush {} => {
                println!("Flushing cache");
                fs::remove_dir_all(rsdk_home.cache())?;
                fs::create_dir_all(rsdk_home.cache())?
            }
            Command::Tui => {
                color_eyre::install()?;
                let result = tui::run(rsdk_home);
                if let Err(err) = tui::restore() {
                    eprintln!(
                        "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
                        err
                    );
                }
                result?
            }
        }
    } else {
        Cli::command().print_help().unwrap();
        println!();
    }
    Ok(())
}

/// Prompt the user with `prompt`, returning `default` when they just press
/// enter or when input can't be read.
pub fn ask(prompt: &str, default: bool) -> bool {
    print!("{prompt}");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let response = input.trim().to_lowercase();
            if response.is_empty() {
                default
            } else {
                matches!(response.as_str(), "y" | "yes")
            }
        }
        Err(_) => default,
    }
}

/// Resolve the current version of `tool`, materializing the `current` symlink
/// if it is missing (e.g. installs that predate the symlink model, where only
/// `default` or `*_HOME` was set). Returns `None` if nothing is current.
fn current_tv(home: &rsdk_home::RsdkHome, tool: &str) -> color_eyre::Result<Option<ToolVersion>> {
    let Some(path) = home.resolve_current(tool) else {
        return Ok(None);
    };
    let version = path
        .file_name()
        .map(|v| v.to_string_lossy().into_owned())
        .expect("current version path has a version component");
    let tv = ToolVersion::new(home, tool, &version);
    // Converge legacy state onto the symlink model.
    if !home.current_symlink_path(tool).exists() {
        tv.make_current()?;
    }
    Ok(Some(tv))
}

/// Names of all tools that have at least one installed version.
fn installed_tools(home: &rsdk_home::RsdkHome) -> color_eyre::Result<Vec<String>> {
    let mut tools: Vec<String> = home
        .all_installed()?
        .map(|tv| tv.tool)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    tools.sort();
    Ok(tools)
}
