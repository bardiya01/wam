use crate::{cli::Commands, config::Config};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::{
    error::Error,
    io::{self, Write},
};
use url::{ParseError, Url};

pub fn handle_command(command: &Commands, config: &Config) -> Result<(), Box<dyn Error>> {
    match command {
        Commands::Add => handle_add(config)?,
    }
    Ok(())
}

fn handle_add(config: &Config) -> Result<(), Box<dyn Error>> {
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
        println!("WARNING: an app with url already {} exists.", url);
    }

    let name = prompt_input("Enter app name: ")?;
    if name_list.contains(&name) {
        println!("WARNING: an app already named {} exists.", name);
    }

    display_url_icon(&url)?;

    Ok(())
}

fn display_url_icon(target_url: &str) -> Result<(), Box<dyn Error>> {
    let parsed = Url::parse(target_url)?;
    let domain = parsed.host_str().ok_or("Invalid domain in URL")?;

    let icon_url = format!("https://www.google.com/s2/favicons?domain={domain}&sz=128");
    let image_bytes = reqwest::blocking::get(&icon_url)?.bytes()?;

    let encoded = BASE64.encode(&image_bytes);

    let chunk_size = 4096;
    let bytes = encoded.as_bytes();
    let total_chunks = bytes.len().div_ceil(chunk_size);

    for (i, chunk) in bytes.chunks(chunk_size).enumerate() {
        let is_last = i == total_chunks - 1;
        let m = if is_last { 0 } else { 1 };
        let chunk_str = std::str::from_utf8(chunk)?;

        if i == 0 {
            // First chunk sets control keys: action=Transmit & Display (a=T), format=PNG (f=100)
            print!("\x1b_Ga=T,f=100,m={m};{chunk_str}\x1b\\");
        } else {
            // Subsequent chunks only specify the continuation flag
            print!("\x1b_Gm={m};{chunk_str}\x1b\\");
        }
    }

    io::stdout().flush()?;
    println!();

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
