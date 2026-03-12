use serde_json::Value;
use std::io::{self, Read};
use std::sync::atomic::Ordering;
use std::sync::OnceLock;
use tempfile::TempDir;

use crate::config::{get_config, AppConfig};
use crate::constants::{DEBUG_MODE, IMAGE_FOLDER, NO_IMAGES};
use crate::utils::ast;

pub static APP_STATE: OnceLock<AppState> = OnceLock::new();

pub struct AppState {
    _temp_dir: TempDir,
}

impl AppState {
    pub fn new(config: AppConfig) -> io::Result<Self> {
        DEBUG_MODE.store(config.debug_mode, Ordering::Relaxed);
        NO_IMAGES.store(!config.render_images, Ordering::Relaxed);

        let temp_dir = TempDir::new()?;
        let image_folder = temp_dir.path().to_str().unwrap().to_string();
        IMAGE_FOLDER.set(image_folder).unwrap();

        Ok(AppState {
            _temp_dir: temp_dir,
        })
    }
}

pub fn read_content(file_path: Option<String>) -> io::Result<String> {
    match file_path {
        Some(path) => std::fs::read_to_string(path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

pub fn parse_and_process_markdown(content: &str) -> io::Result<Value> {
    let config = get_config();
    parse_and_process_markdown_with_config(content, config, true)
}

pub fn parse_and_process_markdown_with_config(
    content: &str,
    config: &AppConfig,
    apply_terminal_tweaks: bool,
) -> io::Result<Value> {
    let mut md_options = markdown::ParseOptions::gfm();
    if !config.render_links {
        md_options.constructs.autolink = false;
        md_options.constructs.gfm_autolink_literal = false;
    }
    let ast = markdown::to_mdast(content, &md_options)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut json: Value = serde_json::from_str(&serde_json::to_string(&ast).unwrap())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    ast::process_definitions(&json);
    ast::process_footnotes(&json);
    if apply_terminal_tweaks {
        ast::modify_heading_ast(&mut json);
        ast::modify_list_item_ast(&mut json);
    }

    Ok(json)
}
