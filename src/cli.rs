use std::env;
use std::io;
use std::process::Command;

use crate::config;
use crate::constants::DOCS_DIR;

pub struct CliOptions {
    pub debug_mode: bool,
    pub file_path: Option<String>,
    pub show_line_numbers: bool,
}

pub enum ParseResult {
    Options(CliOptions),
    Help,
    Version,
    Config,
}

pub fn parse_args(args: &[String]) -> io::Result<ParseResult> {
    let mut options = CliOptions {
        debug_mode: false,
        file_path: None,
        show_line_numbers: false,
    };

    let mut args_iter = args.iter().skip(1);
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--debug" => options.debug_mode = true,
            "--line-numbers" => options.show_line_numbers = true,
            "--help" => return Ok(ParseResult::Help),
            "--version" => return Ok(ParseResult::Version),
            "--generate-config" => return Ok(ParseResult::Config),
            _ => options.file_path = Some(arg.clone()),
        }
    }

    Ok(ParseResult::Options(options))
}

fn render_help() -> io::Result<()> {
    if let Some(file) = DOCS_DIR.get_file("main.md") {
        let content = file
            .contents_utf8()
            .unwrap_or("Help content not available.");
        let temp_dir = tempfile::TempDir::new()?;
        let temp_file = temp_dir.path().join("help.md");
        std::fs::write(&temp_file, content)?;
        Command::new(env::current_exe()?).arg(temp_file).status()?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Help file not found",
        ))
    }
}

fn print_version() -> io::Result<()> {
    println!("smd version {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

pub fn handle_cli_result(result: ParseResult) -> io::Result<Option<CliOptions>> {
    match result {
        ParseResult::Options(options) => Ok(Some(options)),
        ParseResult::Help => {
            render_help()?;
            Ok(None)
        }
        ParseResult::Version => {
            print_version()?;
            Ok(None)
        }
        ParseResult::Config => {
            config::generate_default_config()?;
            Ok(None)
        }
    }
}
