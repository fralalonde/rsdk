mod utils;
mod api;
mod error;
mod dir;
mod download;

// use std::io::{Cursor};
use clap::{Parser, Subcommand};
use log::{info, error, debug};
use env_logger;

/// CLI Struct for command-line arguments
#[derive(Parser)]
#[command(name = "rsdk", version = "0.1", about = "Rust SDK Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Verbose mode (-v for verbose)
    #[arg(short, long)]
    verbose: bool,
}

/// Subcommands enum
#[derive(Subcommand)]
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
        candidate:  Option<String>,
    },
    Use {
        candidate: String,
        version: String,
    },
    Default {
        candidate: String,
        version: String,
    },
    Current {
        candidate: Option<String>,
    },
    Flush {
        data_type: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    if cli.verbose {
        debug!("Verbose mode enabled");
    }

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
            dir.install_from_zip(candidate, &version, &zipfile, &work_dir, true)?;
            dir.set_default(candidate, &version)?;
            println!("Installed");
        }
        Commands::Uninstall { candidate, version } => {
            dir.uninstall(candidate, &version)?;
            println!("Uninstalled");
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
        Commands::Use { candidate, version } => {
            if let Err(e) = utils::test_candidate_present(candidate) {
                error!("Error: {}", e);
            }
            info!("Using candidate: {}, version: {}", candidate, version);
            // Call use_candidate_version logic here
        }
        Commands::Default { candidate, version } => {
            if let Err(e) = utils::test_candidate_present(candidate) {
                error!("Error: {}", e);
            }
            info!("Setting default version for candidate: {}, version: {}", candidate, version);
            // Call set_default_version logic here
        }
        Commands::Current { candidate } => {
            match candidate {
                Some(c) => info!("Showing current version for candidate: {}", c),
                None => info!("Showing current versions for all candidates"),
            }
            // Call show_current_version logic here
        }
        Commands::Flush { data_type } => {
            info!("Flushing cache of type: {}", data_type);
            // Call clear_cache logic here
        }
    }
    Ok(())
}
