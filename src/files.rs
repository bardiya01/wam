use url::Url;

use crate::{
    cli::Cli,
    config::{AppConfig, AppStatus, Config, save_config},
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub fn add_app_entry(
    config: &mut Config,
    name: &str,
    url: String,
    status: AppStatus,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    config.apps.insert(
        name.to_string(),
        AppConfig {
            url: url.clone(),
            status,
        },
    );

    get_metadata(config, name, &url)?;

    // 4. Generate the .desktop file
    save_config(config, cli)?;

    Ok(())
}

pub fn get_metadata(config: &Config, name: &str, url: &str) -> Result<(), Box<dyn Error>> {
    let icon_dir = config
        .settings
        .icon_file_dir
        .as_deref()
        .ok_or("Icon directory is not configured")?;

    let _ = save_icon(url, name, icon_dir)?;

    create_desktop_file(name, config)?;
    Ok(())
}

pub fn remove_app_entry(
    config: &mut Config,
    name: &String,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    config.apps.remove(name);

    remove_desktop_file(name, config)?;

    save_config(config, cli)?;

    Ok(())
}

fn save_icon(target_url: &str, name: &str, icon_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    // 1. Ensure target directory exists
    fs::create_dir_all(icon_dir)?;

    // 2. Extract domain from the URL
    let parsed = Url::parse(target_url)?;
    let domain = parsed.host_str().ok_or("Invalid domain in URL")?;

    // 3. Fetch icon bytes
    let icon_endpoint = format!("https://www.google.com/s2/favicons?domain={domain}&sz=128");
    let image_bytes = reqwest::blocking::get(&icon_endpoint)?.bytes()?;

    // 4. Save to destination path: icon_dir / <name>.png
    let output_path = icon_dir.join(format!("{name}.png"));
    fs::write(&output_path, image_bytes)?;

    Ok(output_path)
}

pub fn toggle_desktop_file(
    name: &str,
    config: &mut Config,
    cli: &Cli,
) -> Result<(), Box<dyn Error>> {
    match config.apps[name].status {
        AppStatus::Enabled => {
            remove_desktop_file(name, config)?;
            println!("Removed {}'s .desktop file", name);

            if let Some(app) = config.apps.get_mut(name) {
                app.status = AppStatus::Disabled;
            }
        }
        AppStatus::Disabled => {
            create_desktop_file(name, config)?;
            println!("Added {}'s .desktop file", name);

            if let Some(app) = config.apps.get_mut(name) {
                app.status = AppStatus::Enabled;
            }
        }
    }

    save_config(config, cli)?;

    Ok(())
}

fn remove_desktop_file(name: &str, config: &Config) -> Result<(), Box<dyn Error>> {
    let desktop_dir = config
        .settings
        .desktop_file_dir
        .as_deref()
        .ok_or("Desktop directory is not configured")?;

    let desktop_file_path = desktop_dir.join(format!("{name}.desktop"));

    fs::remove_file(desktop_file_path)?;

    Ok(())
}

fn create_desktop_file(name: &str, config: &Config) -> Result<(), Box<dyn Error>> {
    let app = config
        .apps
        .get(name)
        .ok_or_else(|| format!("App '{name}' not found in configuration"))?;

    let icon_dir = config
        .settings
        .icon_file_dir
        .as_deref()
        .ok_or("Icon directory is not configured")?;

    let desktop_dir = config
        .settings
        .desktop_file_dir
        .as_deref()
        .ok_or("Desktop directory is not configured")?;

    let command_template = config
        .settings
        .custom_command
        .as_deref()
        .ok_or("No browser command template configured")?;

    // Replace {app} placeholder with target URL
    let exec = command_template.replace("{app}", &app.url);
    let icon_path = icon_dir.join(format!("{name}.png"));
    let desktop_file_path = desktop_dir.join(format!("{name}.desktop"));

    let content = format!(
        "[Desktop Entry]
Version=1.0
Name={name}
Exec={exec}
Icon={}
Terminal=false
Type=Application
Categories=Network;WebBrowser;
",
        icon_path.display()
    );

    fs::write(&desktop_file_path, content)?;
    Ok(())
}
