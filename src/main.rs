
mod api;
mod error;
mod dir;
mod version;
mod shell;
mod http;
mod args;

use std::ffi::OsString;
use std::{env, fs};
use clap::{Parser, Subcommand};
use env_logger;
use log::debug;
use crate::args::{Cli, ARGS};
use crate::version::CandidateVersion;

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

fn main() -> anyhow::Result<()> {
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
                None => api.get_default_version(candidate)?,
                Some(v) => v.clone(),
            };

            let temp_dir = dir.temp();
            // let temp_dir = tempfile::tempdir_in(dir.temp())?;
            // let zipfile = tempdir.path().join("zipfile.zip");
            let work_dir = temp_dir.join("work");

            let zip_file = api.get_cached_file(candidate, &version)?;
            let cv = CandidateVersion::new(&dir,candidate, &version);
            debug!("file is {:?}", zip_file.to_string_lossy());
            cv.install_from_file(&zip_file, &work_dir, true)?;
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
                    for v in api.get_candidate_versions(c)? {
                        println!("{v}");
                    }
                }
                None => {
                    for v in api.get_candidates()? {
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
