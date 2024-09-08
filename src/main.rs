use crate::app::{parse_and_process_markdown, read_content};
use crate::config::initialize_app;
use crate::render::{render_code_file, render_markdown};
use crate::utils::detect_language;

mod app;
mod config;
mod constants;
mod render;
mod utils;

fn main() -> std::io::Result<()> {
    // Parse CLI args, and set up the app state
    let (config, file_path) = initialize_app()?;

    if config.debug_mode {
        println!("Debug mode enabled");
        println!("Configuration: {:?}", config);
    }

    if let Some(path) = file_path {
        let content = read_content(Some(path.to_str().unwrap().to_string()))?;
        let language = detect_language(path.to_str().unwrap());

        if language == "md" {
            let json = parse_and_process_markdown(&content)?;
            render_markdown(&json)?;
        } else {
            render_code_file(&content, &language)?;
        }
    } else {
        // Handle stdin input (assuming it's always Markdown)
        let content = read_content(None)?;
        let json = parse_and_process_markdown(&content)?;
        render_markdown(&json)?;
    }

    Ok(())
}
