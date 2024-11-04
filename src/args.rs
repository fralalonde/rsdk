use std::sync::OnceLock;
use clap::Parser;
use crate::Commands;

/// CLI Struct for command-line arguments
#[derive(Parser, Clone)]
#[command(name = "rsdk", version = "0.1", about = "Rust SDK Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    debug: bool,

    #[arg(short, long)]
    shell: Option<String>,

    #[arg(short, long)]
    envout: Option<String>,

    // #[arg(short, long)]
    // offline: bool,

    #[arg(long)]
    insecure: bool,
}

pub static ARGS: OnceLock<Cli> = OnceLock::new();

pub fn debug() -> bool {
    ARGS.get().unwrap().debug
}

pub fn insecure() -> bool {
    ARGS.get().unwrap().insecure
}
//
// pub fn offline() -> bool {
//     ARGS.get().unwrap().offline
// }

pub fn shell() -> Option<String> {
    ARGS.get().unwrap().shell.clone()
}

pub fn envout() -> Option<String> {
    ARGS.get().unwrap().envout.clone()
}