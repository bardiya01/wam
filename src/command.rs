use crate::{
    cli::{Cli, Commands},
    config::{AppStatus, Config},
    files::add_app_entry,
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

    add_app_entry(config, name, url, AppStatus::Enabled, cli)?;

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

    // Default to https if no protocol/scheme is provided
    let candidate = if !trimmed.contains("://") {
        format!("https://{trimmed}")
    } else {
        trimmed.to_string()
    };

    let mut parsed = Url::parse(&candidate)?;

    // Optional: Strip default ports (80 for HTTP, 443 for HTTPS)
    if (parsed.scheme() == "https" && parsed.port() == Some(443))
        || (parsed.scheme() == "http" && parsed.port() == Some(80))
    {
        let _ = parsed.set_port(None);
    }

    // Optional: Normalize empty query strings or empty fragments
    if parsed.query() == Some("") {
        parsed.set_query(None);
    }
    if parsed.fragment() == Some("") {
        parsed.set_fragment(None);
    }

    Ok(parsed)
}
