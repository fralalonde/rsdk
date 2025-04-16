use std::sync::OnceLock;
use clap::{Parser, Subcommand};

/// CLI Struct for command-line arguments
#[derive(Parser, Clone)]
#[command(name = "rsdk", version = "0.1", about = "Rust SDK Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

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

// pub fn offline() -> bool {
//     ARGS.get().unwrap().offline
// }

pub fn shell() -> Option<String> {
    ARGS.get().unwrap().shell.clone()
}

pub fn envout() -> Option<String> {
    ARGS.get().unwrap().envout.clone()
}

/// Subcommands enum
#[derive(Subcommand, Clone)]
pub enum Command {
    #[command(about = "Initialize rsdk in current shell")]
    Init,

    #[command(about = "Download and install a tool")]
    Install {
        tool: String,
        version: Option<String>,
        #[arg(short, long)]
        default: bool,
    },

    #[command(about = "Uninstall a specific version of a tool")]
    Uninstall {
        tool: String,
        version: String,
    },

    #[command(about = "Alias for uninstall")]
    Remove {
        tool: String,
        version: String,
    },

    #[command(about = "Show the currently active version of a tool")]
    Current {
        tool: Option<String>,
    },

    #[command(about = "Manage tool-specific environment variables")]
    Env {
        #[command(subcommand)]
        command: Option<EnvSubcommand>,
    },

    #[command(about = "List available tools or versions")]
    List {
        tool: Option<String>,
    },

    #[command(about = "List installed tools or versions")]
    Installed {
        tool: Option<String>,
    },

    #[command(about = "Set or show the default version for a tool")]
    Default {
        tool: Option<String>,
        version: Option<String>,
    },

    #[command(about = "Temporarily use a specific tool version")]
    Use {
        tool: String,
        version: Option<String>,
    },

    #[command(about = "Flush internal caches")]
    Flush {},

    #[command(about = "List installed tools or versions")]
    Tui,
}

#[derive(Subcommand, Clone)]
pub enum EnvSubcommand {
    #[command(about = "Save current tool versions to env")]
    Init,

    #[command(about = "Install a tool in env or change its version")]
    Install,

    #[command(about = "Revert current tools to default version (env is untouched)")]
    Clear,
}