mod api;
mod home;
mod version;
mod shell;
mod http;
mod args;
mod http_utils;
mod cache;
mod api_decode;
mod extract;

use std::{env, fs, io};
use std::io::Write;
use anyhow::bail;
use clap::{Parser, Subcommand};
use log::debug;
use crate::args::{Cli, ARGS};
use crate::version::ToolVersion;

/// Subcommands enum
#[derive(Subcommand, Clone)]
enum Commands {
    Init {},
    Install {
        tool: String,
        version: Option<String>,
        #[arg(short, long)]
        default: bool,
    },
    Uninstall {
        tool: String,
        version: String,
    },
    List {
        tool: Option<String>,
    },
    Installed {
        tool: Option<String>,
    },
    Default {
        tool: String,
        version: String,
    },
    Use {
        tool: String,
        version: String,
    },
    Flush {},
}

const RUST_LOG: &str = "RUST_LOG";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _ = ARGS.set(cli.clone());

    if args::debug() {
        env::set_var(RUST_LOG , "debug");
        env::set_var("RUST_BACKTRACE" , "1");
    } else if env::var(RUST_LOG ).is_err() {
        env::set_var(RUST_LOG , "info");
    }

    env_logger::init();

    let rsdk_home = home::RsdkHomeDir::new()?;

    match &cli.command {
        Commands::Init {} => {
            let default_tools = rsdk_home.all_defaults()?;
            let mut paths = vec![];

            // Append each default tool's `bin` directory to PATH
            for default_version in default_tools {
                // prepend default tools path to have precedence over system packages
                paths.push(default_version.bin());
                debug!("setting env var {:?} to {:?}", default_version.home(), default_version.path());
                shell::set_var(&default_version.home(), &default_version.path().to_string_lossy())?;
            }

            let path = env::var_os("PATH").unwrap_or_default();
            env::split_paths(&path)
                // cleanup any hardwired RSDK tools from PATH (how did it get there anyway?)
                .filter(|p| !p.starts_with(rsdk_home.tools()))
                .for_each(|p| paths.push(p));

            let new_path = env::join_paths(paths)?;
            debug!("updating PATH to {:?}", new_path);
            shell::set_var("PATH", &new_path.to_string_lossy())?;
        }
        Commands::Install { tool, version, default } => {
            let api = api::Api::new(&rsdk_home.cache());
            let version = match version {
                None => api.get_default_version(tool)?,
                Some(v) => v.clone(),
            };

            let cv = ToolVersion::new(&rsdk_home, tool, &version);
            if cv.is_installed() {
                bail!("{} is already installed", cv)
            }

            println!("Installing {tool} {version}");
            let temp_dir = rsdk_home.temp();
            let work_dir = temp_dir.join("work");

            let archive = api.get_cached_file(tool, &version)?;
            debug!("archive is {:?}", archive.file_path());
            cv.install_from_file(&archive, &work_dir, true)?;
            if *default || ask_default(tool, &version) {
                cv.make_default()?;
                cv.make_current()?;
            }
            println!("Installed {tool} {version}");
        }
        Commands::Uninstall { tool, version } => {
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
                        // debug!("deleting default symlink");
                        // fs::remove_file(dir.default_symlink_path(tool))?;
                    }
                    if cv.is_current() {
                        new_cv.make_current()?;
                    }
                    println!("{} version {} is the new default", new_cv.tool, new_cv.version)
                }
            }
        }
        Commands::List { tool } => {
            let api = api::Api::new(&rsdk_home.cache());
            match tool {
                Some(c) => {
                    for v in api.get_tool_versions(c)? {
                        println!("{v}");
                    }
                }
                None => {
                    for v in api.get_tools()? {
                        println!("{v}");
                    }
                }
            }
        }
        Commands::Installed { tool } => {
            for cv in rsdk_home.all_installed()? {
                if let Some(tool) = tool {
                    if cv.tool.eq(tool) {
                        println!("{cv}");
                    }
                } else {
                    println!("{cv}");
                }
            }
        }
        Commands::Default { tool, version } => {
            let cv = ToolVersion::new(&rsdk_home, tool, version);
            if cv.is_installed() {
                cv.make_default()?
            } else {
                eprintln!("{cv} is not installed")
            }
        }
        Commands::Use { tool, version } => {
            let cv = ToolVersion::new(&rsdk_home, tool, version);
            if cv.is_installed() {
                cv.make_current()?;
            } else {
                eprintln!("{cv} is not installed")
            }
        }
        Commands::Flush {} => {
            println!("Flushing cache");
            fs::remove_dir_all(rsdk_home.cache())?;
            fs::create_dir_all(rsdk_home.cache())?
        }
    }
    Ok(())
}

pub fn ask_default(tool: &str, version: &str) -> bool {
    print!("Do you want to make {tool} {version} the default? (Y/n): ");
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
