use crate::config::initialize_app;
use crate::viewers::{determine_viewer, ViewerManager};
use std::path::Path;

mod app;
mod config;
mod constants;
mod directory_tree;
mod render;
mod utils;
mod viewers;

fn main() -> std::io::Result<()> {
    let (config, file_paths) = initialize_app()?;
    if config.debug_mode {
        eprintln!("Debug mode enabled");
        eprintln!("Configuration: {:?}", config);
    }

    let viewer_manager = ViewerManager::new();

    match file_paths {
        Some(paths) => {
            for path in paths {
                let path = Path::new(&path);
                if path.is_dir() {
                    directory_tree::handle_directory(path)?;
                } else {
                    let viewer = determine_viewer(path);
                    if viewer.contains(&"image".to_string()) {
                        viewer_manager.visualize(&viewer, "", path.to_str())?;
                    } else {
                        match app::read_content(Some(path.to_str().unwrap().to_string())) {
                            Ok(content) => {
                                viewer_manager.visualize(&viewer, &content, path.to_str())?;
                            }
                            Err(e) => {
                                eprintln!("Error reading file {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        None => {
            let content = app::read_content(None)?;
            viewer_manager.visualize(
                &["markdown".to_string(), "code".to_string()],
                &content,
                None,
            )?;
        }
    }

    Ok(())
}
