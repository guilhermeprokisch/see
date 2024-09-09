use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::app::{AppState, APP_STATE};
use crate::constants::DOCS_DIR;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub max_image_width: Option<u32>,
    pub max_image_height: Option<u32>,
    pub render_images: bool,
    pub render_links: bool,
    pub render_table_borders: bool,
    pub show_line_numbers: bool,
    pub debug_mode: bool,
}

impl AppConfig {
    pub fn load_with_defaults() -> Self {
        let config_path = Self::get_config_path();

        if config_path.exists() {
            Self::load_from_file(&config_path).unwrap_or_else(|_| Self::hardcoded_defaults())
        } else {
            Self::hardcoded_defaults()
        }
    }

    fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
    }

    fn hardcoded_defaults() -> Self {
        AppConfig {
            max_image_width: Some(40),
            max_image_height: Some(13),
            render_images: true,
            render_links: true,
            render_table_borders: false,
            show_line_numbers: true,
            debug_mode: false,
        }
    }

    fn get_config_path() -> PathBuf {
        let config_dir = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .map(|d| d.join(".config"))
                .unwrap_or_else(|| PathBuf::from("."))
        } else {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
        };
        config_dir.join("see").join("config.toml")
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::load_with_defaults()
    }
}

pub fn initialize_app() -> io::Result<(AppConfig, Option<PathBuf>)> {
    let (config, file_path) = parse_cli_args()?;

    let state = AppState::new(config.clone())?;
    APP_STATE.set(state).map_err(|_| {
        io::Error::new(io::ErrorKind::AlreadyExists, "AppState already initialized")
    })?;

    Ok((config, file_path))
}

fn parse_bool(value: Option<&str>) -> bool {
    match value {
        Some(v) => match v.to_lowercase().as_str() {
            "true" | "1" => true,
            "false" | "0" => false,
            _ => true, // Default to true if the value is not recognized
        },
        None => true, // Default to true if no value is provided
    }
}

fn parse_u32(value: Option<&str>) -> Option<u32> {
    value.and_then(|v| v.parse().ok())
}

fn parse_cli_args() -> io::Result<(AppConfig, Option<PathBuf>)> {
    let args: Vec<String> = env::args().collect();
    let mut config = AppConfig::default();
    let mut file_path = None;
    let mut i = 1;

    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with("--") {
            let parts: Vec<&str> = arg[2..].split('=').collect();
            match parts[0] {
                "debug" => config.debug_mode = parse_bool(parts.get(1).map(|s| *s)),
                "max-image-width" => config.max_image_width = parse_u32(parts.get(1).map(|s| *s)),
                "max-image-height" => config.max_image_height = parse_u32(parts.get(1).map(|s| *s)),
                "render-images" => config.render_images = parse_bool(parts.get(1).map(|s| *s)),
                "render-links" => config.render_links = parse_bool(parts.get(1).map(|s| *s)),
                "render-table_borders" => {
                    config.render_table_borders = parse_bool(parts.get(1).map(|s| *s))
                }
                "show-line-numbers" => {
                    config.show_line_numbers = parse_bool(parts.get(1).map(|s| *s))
                }
                "config" => {
                    if let Some(path) = parts.get(1) {
                        if let Ok(file_config) = AppConfig::load_from_file(Path::new(path)) {
                            config = file_config;
                        }
                    }
                }
                "help" => {
                    render_help()?;
                    std::process::exit(0);
                }
                "version" => {
                    println!("see version {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                "generate-config" => {
                    if let Err(e) = generate_default_config() {
                        eprintln!("Error generating default config: {}", e);
                        std::process::exit(1);
                    }
                    std::process::exit(0);
                }
                _ => {
                    eprintln!("Unknown option: {}", arg);
                    render_help()?;
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Invalid command-line argument",
                    ));
                }
            }
        } else {
            file_path = Some(PathBuf::from(arg));
        }
        i += 1;
    }

    Ok((config, file_path))
}

fn render_help() -> io::Result<()> {
    if let Some(file) = DOCS_DIR.get_file("main.md") {
        let content = file
            .contents_utf8()
            .unwrap_or("Help content not available.");
        let temp_dir = tempfile::TempDir::new()?;
        let temp_file = temp_dir.path().join("help.md");
        std::fs::write(&temp_file, content)?;
        Command::new(env::current_exe()?).arg(temp_file).status()?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Help file not found",
        ))
    }
}

pub fn generate_default_config() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::default();
    let config_path = AppConfig::get_config_path();

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    config.save_to_file(&config_path)?;
    println!("Default configuration file created at {:?}", config_path);
    Ok(())
}
