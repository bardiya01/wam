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

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    pub browser: String,
    pub custom_command: String,
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

pub fn load_config(cli: &Cli) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(resolve_config_file(cli)?)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
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
