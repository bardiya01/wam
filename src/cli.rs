use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Web App manager
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to a config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new web-app
    Add,

    /// Remove a web-app
    Remove,

    /// List web-apps
    List,

    /// Sync all configured web-apps
    Sync,
}
