use crate::{cli::Cli, command::handle_command, config::load_config};
use clap::Parser;
use std::error::Error;

mod cli;
mod command;
mod config;
mod files;

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let mut config = load_config(&cli)?;

    println!("{:?}", config);

    if let Some(command) = &cli.command {
        handle_command(command, &mut config, &cli)?;
    }

    Ok(())
}
