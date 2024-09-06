use serde_json::Value;
use std::fs;
use std::io::{self, Read};
use std::sync::atomic::Ordering;
use tempfile::TempDir;

use crate::config::{get_config, load_config};
use crate::constants::{DEBUG_MODE, IMAGE_FOLDER, NO_IMAGES};
use crate::utils::ast;

pub struct AppState {
    _temp_dir: TempDir,
}

impl AppState {
    pub fn new(debug_mode: bool) -> io::Result<Self> {
        load_config();
        DEBUG_MODE.store(debug_mode, Ordering::Relaxed);
        NO_IMAGES.store(!get_config().render_images, Ordering::Relaxed);

        // Create a temporary directory for images
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
        Some(path) => fs::read_to_string(path),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer)
        }
    }
}

pub fn parse_and_process_content(content: &str) -> io::Result<Value> {
    let ast = markdown::to_mdast(content, &markdown::ParseOptions::gfm())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let mut json: Value = serde_json::from_str(&serde_json::to_string(&ast).unwrap())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    ast::process_definitions(&json);
    ast::process_footnotes(&json);
    ast::modify_heading_ast(&mut json);
    ast::modify_list_item_ast(&mut json);

    Ok(json)
}
