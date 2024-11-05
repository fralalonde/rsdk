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

    let dir = home::RsdkHomeDir::new()?;

    match &cli.command {
        Commands::Init {} => {
            let default_tools = dir.all_defaults()?;
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
                .filter(|p| !p.starts_with(dir.tools()))
                .for_each(|p| paths.push(p));

            let new_path = env::join_paths(paths)?;
            debug!("updating PATH to {:?}", new_path);
            shell::set_var("PATH", &new_path.to_string_lossy())?;
        }
        Commands::Install { tool, version, default } => {
            let api = api::Api::new(&dir.cache());
            let version = match version {
                None => api.get_default_version(tool)?,
                Some(v) => v.clone(),
            };
            println!("Installing {tool} {version}");

            let temp_dir = dir.temp();
            let work_dir = temp_dir.join("work");

            let archive = api.get_cached_file(tool, &version)?;
            let cv = ToolVersion::new(&dir, tool, &version);
            debug!("archive is {:?}", archive.file_path());
            cv.install_from_file(&archive, &work_dir, true)?;
            if *default || ask_default(tool, &version) {
                cv.make_default()?;
                cv.make_current()?;
            }
            println!("Installed {tool} {version}");
        }
        Commands::Uninstall { tool, version } => {
            let cv = ToolVersion::new(&dir, tool, version);
            cv.uninstall()?;
            println!("Uninstalled {tool} {version}");
            // FIXME use dir
            // if cv.is_default() {
            //     debug!("deleting default symlink");
            //     fs::remove_file(dir.default_symlink_path(tool))?;
            // }
            if cv.is_current() {
                debug!("unsetting HOME and removing bin from PATH");
                // TODO
            }
            // TODO check for alternate installed versions
            // propose new default/current
            // OR delete tool dir if empty
        }
        Commands::List { tool } => {
            let api = api::Api::new(&dir.cache());
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
            for cv in dir.all_versions()? {
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
            let cv = ToolVersion::new(&dir, tool, version);
            if cv.is_installed() {
                cv.make_default()?
            } else {
                eprintln!("{cv} is not installed")
            }
        }
        Commands::Use { tool, version } => {
            let cv = ToolVersion::new(&dir, tool, version);
            if cv.is_installed() {
                cv.make_current()?;
            } else {
                eprintln!("{cv} is not installed")
            }
        }
        Commands::Flush {} => {
            println!("Flushing cache");
            fs::remove_dir_all(dir.cache())?;
            fs::create_dir_all(dir.cache())?
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
