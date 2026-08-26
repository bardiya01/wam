use serde::{Deserialize, Serialize};

use crate::cli::Cli;
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub settings: Settings,
    pub apps: HashMap<String, AppConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    #[serde(default)]
    pub browser: Option<String>,

    #[serde(default)]
    pub custom_command: Option<String>,

    #[serde(default)]
    pub desktop_file_dir: Option<PathBuf>,

    #[serde(default)]
    pub icon_file_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppStatus {
    Enabled,
    Disabled,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub url: String,
    pub status: AppStatus,
}

pub fn save_config(config: &Config, cli: &Cli) -> Result<(), Box<dyn Error>> {
    let toml_str = toml::to_string_pretty(config)?;
    fs::write(resolve_config_file(cli)?, toml_str)?;
    Ok(())
}

pub fn load_config(cli: &Cli) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(resolve_config_file(cli)?)?;
    let mut config: Config = toml::from_str(&content)?;

    // 1. Validate browser / command configuration
    resolve_browser_command(&mut config.settings)?;

    // 2. Populate directory paths with defaults and ensure they exist on disk
    config.settings.desktop_file_dir = Some(resolve_desktop_dir(
        config.settings.desktop_file_dir.as_ref(),
    )?);

    config.settings.icon_file_dir = Some(resolve_icon_dir(config.settings.icon_file_dir.as_ref())?);

    Ok(config)
}

/// Maps known browser identifiers to their standard web-app command template.
fn get_browser_preset(browser: &str) -> Option<&'static str> {
    match browser.trim().to_lowercase().as_str() {
        "helium" => Some("helium --app={app} --no-first-run --no-default-browser-check"),
        "chrome" | "google-chrome" => Some("google-chrome --app={app}"),
        "chromium" => Some("chromium --app={app}"),
        "brave" => Some("brave --app={app}"),
        "edge" | "msedge" | "microsoft-edge" => Some("microsoft-edge --app={app}"),
        "vivaldi" => Some("vivaldi --app={app}"),
        "firefox" => Some("firefox --new-window {app}"),
        "librewolf" => Some("librewolf --new-window {app}"),
        _ => None,
    }
}

/// Validates that either `browser` or `custom_command` is present,
/// enforces the `{app}` placeholder, and fills in missing commands from presets.
fn resolve_browser_command(settings: &mut Settings) -> Result<(), Box<dyn Error>> {
    match (&settings.custom_command, &settings.browser) {
        // Case 1: Custom command explicitly provided
        (Some(cmd), _) if !cmd.trim().is_empty() => {
            if !cmd.contains("{app}") {
                return Err(format!(
                    "Invalid custom_command '{}': must contain the '{{app}}' placeholder.",
                    cmd
                )
                .into());
            }
        }

        // Case 2: Browser preset specified, custom_command is missing
        (_, Some(browser)) if !browser.trim().is_empty() => {
            if let Some(preset) = get_browser_preset(browser) {
                settings.custom_command = Some(preset.to_string());
            } else {
                return Err(format!(
                    "Unknown browser '{}'. Supported presets: helium, chrome, chromium, brave, edge, vivaldi, firefox, librewolf. Alternatively, define 'custom_command'.",
                    browser
                )
                .into());
            }
        }

        // Case 3: Neither is defined or both are empty strings
        _ => {
            return Err(
                "Config error: Either 'browser' or 'custom_command' must be defined in [settings]."
                    .into(),
            );
        }
    }

    Ok(())
}

fn resolve_desktop_dir(custom_path: Option<&PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let target_dir = match custom_path {
        Some(dir) => dir.clone(),
        None => {
            if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
                PathBuf::from(xdg_data_home).join("applications")
            } else if let Some(home_dir) = env::var_os("HOME") {
                PathBuf::from(home_dir).join(".local/share/applications")
            } else {
                return Err("Neither XDG_DATA_HOME nor HOME environment variable is set".into());
            }
        }
    };

    fs::create_dir_all(&target_dir)?;
    Ok(target_dir)
}

fn resolve_icon_dir(custom_path: Option<&PathBuf>) -> Result<PathBuf, Box<dyn Error>> {
    let target_dir = match custom_path {
        Some(dir) => dir.clone(),
        None => {
            if let Some(xdg_cache_home) = env::var_os("XDG_CACHE_HOME") {
                PathBuf::from(xdg_cache_home).join("wam/icons")
            } else if let Some(home_dir) = env::var_os("HOME") {
                PathBuf::from(home_dir).join(".cache/wam/icons")
            } else {
                return Err("Neither XDG_CACHE_HOME nor HOME environment variable is set".into());
            }
        }
    };

    fs::create_dir_all(&target_dir)?;
    Ok(target_dir)
}

fn resolve_config_file(cli: &Cli) -> Result<PathBuf, Box<dyn Error>> {
    // --config override
    if let Some(config_path) = &cli.config {
        check_path(config_path)?;
        return Ok(config_path.clone());
    }

    let mut paths: Vec<PathBuf> = Vec::new();

    if let Some(wam_path) = env::var_os("WAM_CONFIG_PATH") {
        paths.push(PathBuf::from(wam_path));
    }
    if let Some(xdg_home) = env::var_os("XDG_CONFIG_HOME") {
        paths.push(PathBuf::from(xdg_home).join("wam/config.toml"));
    } else if let Some(home) = env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".config/wam/config.toml"));
    }

    if paths.is_empty() {
        return Err("Neither WAM_CONFIG_PATH, XDG_CONFIG_HOME, nor HOME are set".into());
    }

    for path in &paths {
        if check_path(path).is_ok() {
            return Ok(path.clone());
        }
    }

    for path in paths {
        if try_create_file(&path).is_ok() {
            return Ok(path);
        }
    }

    Err("Failed to resolve or create a configuration file".into())
}

fn try_create_file(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;

    Ok(())
}

fn check_path(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        return Err(format!("Config file not found at: {}", path.display()).into());
    }
    if !path.is_file() {
        return Err(format!("Config path is not a file: {}", path.display()).into());
    }
    Ok(())
}
