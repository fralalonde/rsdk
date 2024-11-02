mod api;
mod dir;
mod version;
mod shell;
mod http;
mod args;

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
        force: bool,
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

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let _ = ARGS.set(cli.clone());

    let dir = dir::RsdkDir::new()?;

    match &cli.command {
        Commands::Init {} => {
            let default_tools = dir.all_defaults()?;

            let path = env::var_os("PATH").unwrap_or_default();
            let mut paths: Vec<_> = env::split_paths(&path)
                // cleanup any hardwired tools from PATH (how would it get there anyway)
                .filter(|p| !p.starts_with(dir.tools()))
                .collect();

            // Append each default tool's `bin` directory to PATH
            for default_version in default_tools {
                paths.push(default_version.bin());
                shell::set_var(&default_version.home(), &default_version.path().to_string_lossy())?;
            }

            let new_path = env::join_paths(paths).expect("Failed to join paths");
            shell::set_var("PATH", &new_path.to_string_lossy())?;
        }
        Commands::Install { tool, version, force, default } => {
            let api = api::Api::new(&dir.cache(), *force);
            let version = match version {
                None => api.get_default_version(tool)?,
                Some(v) => v.clone(),
            };
            println!("Installing {tool} {version}");

            let temp_dir = dir.temp();
            let work_dir = temp_dir.join("work");

            let zip_file = api.get_cached_file(tool, &version)?;
            let cv = ToolVersion::new(&dir, tool, &version);
            debug!("file is {:?}", zip_file.to_string_lossy());
            cv.install_from_file(&zip_file, &work_dir, true)?;
            if *default || ask_default(tool, &version) {
                cv.set_default()?;
                cv.set_current()?;
            }
            println!("Installed {tool} {version}");
        }
        Commands::Uninstall { tool, version } => {
            let cv = ToolVersion::new(&dir, tool, version);
            cv.uninstall()?;
            println!("Uninstalled {tool} {version}");
        }
        Commands::List { tool } => {
            let api = api::Api::new(&dir.cache(), false);
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
                cv.set_default()?
            } else {
                eprintln!("{cv} is not installed")
            }
        }
        Commands::Use { tool, version } => {
            let cv = ToolVersion::new(&dir, tool, version);
            if cv.is_installed() {
                cv.set_current()?;
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
