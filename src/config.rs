use dirs::home_dir;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub max_image_width: Option<u32>,
    pub max_image_height: Option<u32>,
    pub render_images: bool,
    pub render_links: bool,
    pub render_table_borders: bool,
    pub show_line_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            max_image_width: Some(40),
            max_image_height: Some(13),
            render_images: true,
            render_links: true,
            render_table_borders: false,
            show_line_numbers: true,
        }
    }
}

lazy_static! {
    static ref GLOBAL_CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

pub fn load_config() {
    let config_dir = if cfg!(target_os = "macos") {
        home_dir()
            .map(|path| path.join(".config"))
            .unwrap_or_else(|| PathBuf::from("~/.config"))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
    };

    let config_path = config_dir.join("smd").join("config.toml");

    let config = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(parsed_config) => parsed_config,
                Err(e) => {
                    eprintln!(
                        "Failed to parse config file: {}. Using default configuration.",
                        e
                    );
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read config file: {}. Using default configuration.",
                    e
                );
                Config::default()
            }
        }
    } else {
        eprintln!(
            "Config file not found at {:?}. Using default configuration.",
            config_path
        );
        Config::default()
    };

    let mut global_config = GLOBAL_CONFIG.write().unwrap();
    *global_config = config;
}

pub fn get_config() -> Config {
    GLOBAL_CONFIG.read().unwrap().clone()
}

pub fn generate_default_config() -> std::io::Result<()> {
    let config_dir = if cfg!(target_os = "macos") {
        home_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not find home directory",
                )
            })?
            .join(".config")
    } else {
        dirs::config_dir().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find config directory",
            )
        })?
    };

    let smd_config_dir = config_dir.join("smd");
    fs::create_dir_all(&smd_config_dir)?;

    let config_path = smd_config_dir.join("config.toml");

    let default_config = Config::default();
    let toml = toml::to_string_pretty(&default_config).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to serialize config: {}", e),
        )
    })?;

    fs::write(&config_path, toml)?;

    println!("Default configuration file created at {:?}", config_path);
    Ok(())
}
