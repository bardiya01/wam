use crate::{
    cli::{Cli, Commands},
    config::{AppStatus, Config},
    files::{add_app_entry, get_metadata, remove_app_entry},
};
use std::{
    error::Error,
    io::{self, Write},
};
use url::{ParseError, Url};

pub fn handle_command(
    command: &Commands,
    config: &mut Config,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    match command {
        Commands::Add => handle_add(config, cli)?,
        Commands::Remove => handle_remove(config, cli)?,
        Commands::List => handle_list(config)?,
        Commands::Sync => handle_sync(config)?,
    }
    Ok(())
}

fn handle_add(config: &mut Config, cli: &Cli) -> Result<(), Box<dyn Error>> {
    let mut name_list = Vec::new();
    let mut url_list = Vec::new();

    for app in &config.apps {
        name_list.push(app.0.clone());
        url_list.push(app.1.url.clone());
    }

    let url = prompt_input("Enter app url: ")?;
    let url = standardize_url(&url)?.to_string();
    println!("{url}");
    if name_list.contains(&url) {
        println!("WARNING: an app with this url already {} exists.", url);
    }

    let name = prompt_input("Enter app name: ")?;
    if name_list.contains(&name) {
        println!("WARNING: an app already named {} exists.", name);
    }

    add_app_entry(config, &name, url, AppStatus::Enabled, cli)?;

    println!("{} added.", name);

    Ok(())
}

fn handle_remove(config: &mut Config, cli: &Cli) -> Result<(), Box<dyn Error>> {
    let mut name_list = Vec::new();
    for app in &config.apps {
        name_list.push(app.0.clone());
    }

    let name = prompt_input("Enter app name to delete: ")?;

    if name_list.contains(&name) {
        remove_app_entry(config, &name, cli)?;
    } else {
        println!("App named {} does not exist.", name);
    }

    println!("{} removed.", name);

    Ok(())
}

fn handle_list(config: &Config) -> Result<(), Box<dyn Error>> {
    let max_name_len = config.apps.keys().map(|k| k.len()).max().unwrap_or(0);
    for app in &config.apps {
        let icon = match app.1.status {
            AppStatus::Enabled => " ",
            AppStatus::Disabled => " ",
        };
        println!(
            "{} {}{} -> {}",
            icon,
            app.0,
            " ".repeat(max_name_len - app.0.len()),
            app.1.url
        );
    }
    Ok(())
}

fn handle_sync(config: &Config) -> Result<(), Box<dyn Error>> {
    for app in &config.apps {
        let _ = match app.1.status {
            AppStatus::Enabled => get_metadata(config, app.0, &app.1.url),
            _ => Ok(()),
        };
    }
    Ok(())
}

fn prompt_input(prompt: &str) -> io::Result<String> {
    print!("{prompt} ");
    io::stdout().flush()?; // Ensure prompt displays before reading input

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn standardize_url(input: &str) -> Result<Url, ParseError> {
    let trimmed = input.trim();

    let candidate = if !trimmed.contains("://") {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let mut parsed = Url::parse(&candidate)?;

    if parsed.query() == Some("") {
        parsed.set_query(None);
    }
    if parsed.fragment() == Some("") {
        parsed.set_fragment(None);
    }

    Ok(parsed)
}
