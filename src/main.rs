
mod api;
mod error;
mod dir;
mod download;
mod version;

use std::fs;
use std::sync::OnceLock;
use clap::{Parser, Subcommand};
use log::{info, error, debug};
use env_logger;
use crate::version::CandidateVersion;

/// CLI Struct for command-line arguments
#[derive(Parser, Clone)]
#[command(name = "rsdk", version = "0.1", about = "Rust SDK Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    verbose: bool,

    #[arg(short, long)]
    force: bool,

    #[arg(short, long)]
    shell: Option<String>
}

pub static ARGS: OnceLock<Cli> = OnceLock::new();

/// Subcommands enum
#[derive(Subcommand, Clone)]
enum Commands {
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
        Commands::Install { candidate, version } => {
            let api = api::Api::new(&dir.cache());
            let version = match version {
                None => api.get_default_version(candidate).await?,
                Some(v) => v.clone(),
            };

            let tempdir = tempfile::tempdir()?;
            let zipfile = tempdir.path().join("zipfile.zip");
            let work_dir = tempdir.path().join("work");

            api.get_file2(candidate, &version, &zipfile).await?;
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
