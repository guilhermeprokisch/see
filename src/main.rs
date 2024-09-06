mod app;
mod cli;
mod config;
mod constants;
mod render;
mod utils;

use crate::app::{parse_and_process_content, read_content, AppState};
use crate::cli::parse_args;
use crate::render::render_markdown;
use std::env;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let (debug_mode, file_path) = parse_args(&args)?;
    let _app_state = AppState::new(debug_mode)?;
    let content = read_content(file_path)?;
    let json = parse_and_process_content(&content)?;
    render_markdown(&json)?;
    Ok(())
}
