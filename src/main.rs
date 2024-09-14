use crate::config::initialize_app;
use crate::viewers::{determine_viewer, ViewerManager};
use std::io::{self, IsTerminal};
use std::path::Path;

mod app;
mod config;
mod constants;
mod directory_tree;
mod render;
mod utils;
mod viewers;

use base64::{engine::general_purpose, Engine as _};
use std::fs;

fn main() -> std::io::Result<()> {
    let (config, file_paths) = initialize_app()?;
    if config.debug_mode {
        eprintln!("Debug mode enabled");
        eprintln!("Configuration: {:?}", config);
    }

    let viewer_manager = ViewerManager::new();

    match &file_paths {
        Some(paths) if !paths.is_empty() => {
            for path in paths {
                let path = Path::new(path);
                if path.is_dir() {
                    directory_tree::handle_directory(path)?;
                } else {
                    let viewer = determine_viewer(path);
                    if viewer.contains(&"image".to_string()) {
                        let img_data = fs::read(path)?;
                        let b64 = general_purpose::STANDARD.encode(&img_data);
                        let extension = path.extension().unwrap().to_str().unwrap();
                        let content = format!("data:image/{};base64,{}", extension, b64);

                        if !io::stdout().is_terminal() {
                            let file_name = path.file_name().unwrap().to_str().unwrap();
                            println!("![{}]({})", file_name, content);
                        } else {
                            viewer_manager.visualize(&viewer, &content, None)?;
                        }
                    } else {
                        let content = app::read_content(Some(path.to_string_lossy().into_owned()))?;
                        if !io::stdout().is_terminal() {
                            print!("{}", content);
                        } else {
                            viewer_manager.visualize(
                                &viewer,
                                &content,
                                Some(path.to_str().unwrap()),
                            )?;
                        }
                    }
                }
            }
        }
        _ => {
            let content = app::read_content(None)?;
            if !io::stdout().is_terminal() {
                print!("{}", content);
            } else {
                viewer_manager.visualize(&["markdown".to_string()], &content, None)?;
            }
        }
    }

    Ok(())
}
