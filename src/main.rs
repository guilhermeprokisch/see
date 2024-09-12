// File: src/main.rs

use crate::config::initialize_app;
use crate::multi_tool::{determine_tool_names, MultiTool};
use std::path::Path;

mod app;
mod config;
mod constants;
mod directory_tree;
mod multi_tool;
mod render;
mod utils;

fn main() -> std::io::Result<()> {
    let (config, file_paths) = initialize_app()?;

    if config.debug_mode {
        eprintln!("Debug mode enabled");
        eprintln!("Configuration: {:?}", config);
    }

    let multi_tool = MultiTool::new();

    match file_paths {
        Some(paths) => {
            for path in paths {
                let path = Path::new(&path);

                if path.is_dir() {
                    directory_tree::handle_directory(path)?;
                } else {
                    println!("\nFile: {}", path.display());
                    let tool_names = determine_tool_names(path);

                    if tool_names.contains(&"image".to_string()) {
                        // For images, we don't need to read the content
                        multi_tool.visualize(&tool_names, "", path.to_str())?;
                    } else {
                        // For other files, read the content as before
                        match app::read_content(Some(path.to_str().unwrap().to_string())) {
                            Ok(content) => {
                                multi_tool.visualize(&tool_names, &content, path.to_str())?;
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
            // Handle stdin input (assuming it's always Markdown)
            let content = app::read_content(None)?;
            multi_tool.visualize(
                &["markdown".to_string(), "code".to_string()],
                &content,
                None,
            )?;
        }
    }

    Ok(())
}
