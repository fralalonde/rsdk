mod sdkman_client;
mod rsdk_home;
mod tool_version;
mod shell;
mod http_client;
mod args;
mod http_utils;
mod cache;
mod sdkman_decode;
mod archive;
mod rcfile;
mod tui;

use std::{env, fs, io};
use std::io::Write;
use eyre::bail;
use clap::{CommandFactory, Parser};
use log::{debug};
use crate::args::{Cli, Command, EnvSubcommand, ARGS};
use crate::tool_version::ToolVersion;
use crate::tui::App;

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

                // Append each default tool's `bin` directory to PATH
                for default_version in default_tools {
                    // prepend default tools path to have precedence over system packages
                    paths.push(default_version.bin());
                    debug!("setting env var {:?} to {:?}", default_version.home(), default_version.path());
                    shell::set_env_var_after_exit(&default_version.home(), &default_version.path().to_string_lossy())?;
                }

                let path = env::var_os("PATH").unwrap_or_default();
                env::split_paths(&path)
                    // cleanup any hardwired RSDK tools from PATH (how did it get there anyway?)
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
                        if *default || ask_default(&tv.tool, &tv.version) {
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

                if cv.is_default() {
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
                        if cv.is_default() {
                            new_cv.make_default()?;
                        }
                        if cv.is_current() {
                            new_cv.make_current()?;
                        }
                        println!("{} version {} is the new default", new_cv.tool, new_cv.version)
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
                for tv in rsdk_home.all_installed()? {
                    if let Some(tool) = tool {
                        if tv.tool.eq(tool) {
                            println!("{tv}");
                        }
                    } else {
                        println!("{tv}");
                    }
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
                            println!("{:?}", version);
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
                    for tv in rsdk_home.all_installed()? {
                        if tv.tool.eq(tool) && tv.is_current() {
                            println!("{tv}");
                            return Ok(());
                        }
                    }
                    bail!("no current version of tool '{}'", tool);
                } else {
                    for tv in rsdk_home.all_installed()? {
                        if tv.is_current() {
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
                        eprintln!("'tool {tv}' is not installed")
                    }
                } else {
                    let version = shell::current_tool_version(tool)?;
                    println!("{:?}", version);
                }
            }
            Command::Flush {} => {
                println!("Flushing cache");
                fs::remove_dir_all(rsdk_home.cache())?;
                fs::create_dir_all(rsdk_home.cache())?
            }
            Command::Tui => {
                color_eyre::install()?;
                let mut terminal = tui::init()?;
                let app_result = App::default().run(&mut terminal);
                if let Err(err) = tui::restore() {
                    eprintln!(
                        "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
                        err
                    );
                }
                app_result?
            }
        }
    } else {
        Cli::command().print_help().unwrap();
        println!();
    }
    Ok(())
}

pub fn ask_default(tool: &str, version: &str) -> bool {
    print!("Do you want to make {tool} {version} the new default? (Y/n): ");
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let response = input.trim().to_lowercase();
            response.is_empty() || matches!(response.as_str(), "y" | "yes")
        }
        Err(_) => true,
    }
}
