mod app;
mod cli;
mod config;
mod constants;
mod render;
mod utils;

use crate::app::{parse_and_process_content, read_content, AppState};
use crate::cli::parse_args;
use crate::render::{render_code_file, render_markdown};
use std::env;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (debug_mode, file_path) = parse_args(&args)?;
    let _app_state = AppState::new(debug_mode)?;

    if let Some(path) = file_path {
        let content = read_content(Some(path.clone()))?;
        let extension = Path::new(&path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match extension {
            "md" => {
                let json = parse_and_process_content(&content)?;
                render_markdown(&json)?;
            }
            _ => {
                render_code_file(&content, extension)?;
            }
        }
    } else {
        // Handle stdin input (assuming it's always Markdown)
        let content = read_content(None)?;
        let json = parse_and_process_content(&content)?;
        render_markdown(&json)?;
    }

    Ok(())
}
