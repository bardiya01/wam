use crate::{
    cli::{Cli, Commands},
    config::{AppStatus, Config},
    files::{add_app_entry, get_metadata, remove_app_entry, toggle_desktop_file},
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
        Commands::Add { name, url } => handle_add(config, cli, name, url)?,
        Commands::Remove { name, yes } => handle_remove(config, cli, name, *yes)?,
        Commands::List => handle_list(config)?,
        Commands::Sync => handle_sync(config)?,
        Commands::Toggle { name } => handle_toggle(config, cli, name)?,
    }
    Ok(())
}

fn handle_add(
    config: &mut Config,
    cli: &Cli,
    name: &Option<String>,
    url: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut name_list = Vec::new();
    let mut url_list = Vec::new();

    for app in &config.apps {
        name_list.push(app.0.clone());
        url_list.push(app.1.url.clone());
    }

    let url = match url {
        Some(url) => url,
        None => &prompt_input("Enter app url: ")?,
    };
    let url = standardize_url(url)?.to_string();
    if name_list.contains(&url) {
        println!("WARNING: an app with this url already {} exists.", url);
    }

    let name = match name {
        Some(name) => name,
        None => &prompt_input("Enter app name: ")?,
    };
    if name_list.contains(name) {
        println!("WARNING: an app already named {} exists.", name);
    }

    add_app_entry(config, name, url, AppStatus::Enabled, cli)?;
    println!("{} with url {} added.", name, config.apps[name].url);

    Ok(())
}

fn handle_remove(
    config: &mut Config,
    cli: &Cli,
    name: &Option<String>,
    yes: bool,
) -> Result<(), Box<dyn Error>> {
    let mut name_list = Vec::new();
    for app in &config.apps {
        name_list.push(app.0.clone());
    }

    let name = match name {
        Some(name) => name,
        None => &prompt_input("Enter app name to delete: ")?,
    };

    if name_list.contains(name) {
        if yes
            || prompt_input(&format!(
                "Remove {} with url {}? (y/N)",
                name, config.apps[name].url
            ))? == 'y'.to_string()
        {
            remove_app_entry(config, name, cli)?;
            println!("{} removed.", name);
        }
    } else {
        println!("App named {} does not exist.", name);
    }

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

fn handle_toggle(
    config: &mut Config,
    cli: &Cli,
    name: &Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut name_list = Vec::new();
    for app in &config.apps {
        name_list.push(app.0.clone());
    }

    let name = match name {
        Some(name) => name,
        None => &prompt_input("Enter app name to delete: ")?,
    };

    toggle_desktop_file(name, config, cli)?;

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
