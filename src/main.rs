use crate::{cli::Cli, command::handle_command, config::load_config};
use clap::Parser;
use std::error::Error;

mod cli;
mod command;
mod config;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let config = load_config(&cli)?;

    if let Some(command) = &cli.command {
        handle_command(command, &config)?;
    }

    Ok(())
}
