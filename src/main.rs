
mod api;
mod error;
mod dir;
mod download;
mod version;
mod shell;

use std::ffi::OsString;
use std::{env, fs};
use std::sync::OnceLock;
use clap::{Parser, Subcommand};
use env_logger;
use crate::version::CandidateVersion;

/// CLI Struct for command-line arguments
#[derive(Parser, Clone)]
#[command(name = "rsdk", version = "0.1", about = "Rust SDK Manager")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    force: bool,

    #[arg(short, long)]
    shell: Option<String>,

    #[arg(short, long)]
    envout: Option<String>,

    #[arg(short, long)]
    offline: bool,
}

pub static ARGS: OnceLock<Cli> = OnceLock::new();

/// Subcommands enum
#[derive(Subcommand, Clone)]
enum Commands {
    Attach {
    },
    Install {
        candidate: String,
        version: Option<String>,
    },
    Uninstall {
        candidate: String,
        version: String,
    },
    List {
        candidate: Option<String>,
    },
    Default {
        candidate: String,
        version: String,
    },
    Use {
        candidate: String,
        version: String,
    },
    Flush {
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();
    let _ = ARGS.set(cli.clone());

    let dir = dir::RsdkDir::new()?;

    // Example command handling for 'version'
    match &cli.command {
        Commands::Attach {} => {
            let default_candidates = dir.all_defaults()?;

            let path = env::var_os("PATH").unwrap_or_else(OsString::new);
            let mut paths: Vec<_> = env::split_paths(&path)
                // cleanup any hardwired candidate from PATH (how would it get there anyway)
                .filter(|p| !p.starts_with(&dir.candidates()))
                .collect();

            // Append each default candidate's `bin` directory to PATH if it exists
            for default_version in default_candidates {
                paths.push(default_version.bin());
                shell::set_var(&default_version.home(), &default_version.path().to_string_lossy())?;
            }

            let new_path = env::join_paths(paths).expect("Failed to join paths");
            shell::set_var("PATH", &new_path.to_string_lossy())?;
        }
        Commands::Install { candidate, version } => {
            let api = api::Api::new(&dir.cache());
            let version = match version {
                None => api.get_default_version(candidate).await?,
                Some(v) => v.clone(),
            };

            let tempdir = tempfile::tempdir()?;
            let zipfile = tempdir.path().join("zipfile.zip");
            let work_dir = tempdir.path().join("work");

            api.get_file(candidate, &version, &zipfile).await?;
            let cv = CandidateVersion::new(&dir,candidate, &version);
            cv.install_from_zip(&zipfile, &work_dir, true)?;
            cv.set_default()?;
            cv.make_current()?;
            println!("Installed {candidate} {version}");
        }
        Commands::Uninstall { candidate, version } => {
            let cv = CandidateVersion::new(&dir,candidate, &version);
            cv.uninstall()?;
            println!("Uninstalled {candidate} {version}");
        }
        Commands::List { candidate } => {
            let api = api::Api::new(&dir.cache());
            match candidate {
                Some(c) => {
                    for v in api.get_candidate_versions(c).await? {
                        println!("{v}");
                    }
                }
                None => {
                    for v in api.get_candidates().await? {
                        println!("{v}");
                    }
                }
            }
        }
        Commands::Default { candidate, version } => {
            let cv = CandidateVersion::new(&dir, candidate, version);
            cv.set_default()?
        }
        Commands::Use { candidate, version } => {
            let cv = CandidateVersion::new(&dir, candidate, version);
            cv.set_default()?
        }
        Commands::Flush { } => {
            println!("Flushing cache");
            fs::remove_dir_all(dir.cache())?;
            fs::create_dir_all(dir.cache())?
        }
    }
    Ok(())
}
