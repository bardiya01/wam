use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Web App manager
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to a config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Add a new web-app
    Add {
        /// Name of the app to add
        #[arg(long)]
        name: Option<String>,

        /// URL of the app to add
        #[arg(long)]
        url: Option<String>,
    },

    /// Remove a web-app
    Remove {
        #[arg(long)]
        name: Option<String>,

        /// Automatically confirm deletion without prompting
        #[arg(short = 'y')]
        yes: bool,
    },

    /// List web-apps
    List,

    /// Sync all configured web-apps
    Sync,

    /// Toggle the .desktop file
    Toggle {
        #[arg(long)]
        name: Option<String>,
    },
}
