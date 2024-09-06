use std::env;
use std::io;
use std::process::Command;

use crate::config;
use crate::constants::DOCS_DIR;

pub fn parse_args(args: &[String]) -> io::Result<(bool, Option<String>)> {
    let mut debug_mode = false;
    let mut file_path = None;
    for arg in &args[1..] {
        match arg.as_str() {
            "--debug" => debug_mode = true,
            "--help" => {
                return render_help();
            }
            "--version" => {
                return print_version();
            }
            "--generate-config" => {
                return config::generate_default_config().map(|_| (false, None));
            }
            _ => file_path = Some(arg.clone()),
        }
    }
    Ok((debug_mode, file_path))
}

fn render_help() -> io::Result<(bool, Option<String>)> {
    if let Some(file) = DOCS_DIR.get_file("main.md") {
        let content = file
            .contents_utf8()
            .unwrap_or("Help content not available.");
        let temp_dir = tempfile::TempDir::new()?;
        let temp_file = temp_dir.path().join("help.md");
        std::fs::write(&temp_file, content)?;
        Command::new(env::current_exe()?).arg(temp_file).status()?;
        Ok((false, None))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Help file not found",
        ))
    }
}

fn print_version() -> io::Result<(bool, Option<String>)> {
    println!("smd version {}", env!("CARGO_PKG_VERSION"));
    Ok((false, None))
}
