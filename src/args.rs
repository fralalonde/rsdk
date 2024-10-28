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

pub fn force() -> bool {
    ARGS.get().unwrap().force
}

pub fn offline() -> bool {
    ARGS.get().unwrap().offline
}

pub fn shell() -> Option<String> {
    ARGS.get().unwrap().shell.clone()
}

pub fn envout() -> Option<String> {
    ARGS.get().unwrap().envout.clone()
}