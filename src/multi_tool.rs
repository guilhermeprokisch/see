use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::app;
use crate::render::{render_code_file, render_image_file, render_markdown};
use crate::utils::detect_language;

pub struct MultiTool {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl MultiTool {
    pub fn new() -> Self {
        let mut multi_tool = MultiTool {
            tools: HashMap::new(),
        };

        // Register default tools
        multi_tool.register_tool("markdown", Box::new(MarkdownTool));
        multi_tool.register_tool("code", Box::new(CodeTool));
        multi_tool.register_tool("image", Box::new(ImageTool));

        multi_tool
    }

    pub fn register_tool(&mut self, name: &str, tool: Box<dyn Tool>) {
        self.tools.insert(name.to_string(), tool);
    }

    pub fn visualize(
        &self,
        tool_names: &[String],
        content: &str,
        file_path: Option<&str>,
    ) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);

        for (index, tool_name) in tool_names.iter().enumerate() {
            if index > 0 {
                writeln!(stdout)?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                writeln!(stdout, "---")?;
                stdout.reset()?;
                writeln!(stdout)?;
            }

            if let Some(tool) = self.tools.get(tool_name) {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                writeln!(stdout, "Visualization with {}", tool_name)?;
                stdout.reset()?;
                writeln!(stdout)?;

                tool.visualize(content, file_path)?;
            } else {
                eprintln!("Unknown tool: {}", tool_name);
            }
        }

        Ok(())
    }
}

pub trait Tool {
    fn visualize(&self, content: &str, file_path: Option<&str>) -> io::Result<()>;
}

struct MarkdownTool;

impl Tool for MarkdownTool {
    fn visualize(&self, content: &str, _file_path: Option<&str>) -> io::Result<()> {
        let json = app::parse_and_process_markdown(content)?;
        render_markdown(&json)
    }
}

struct CodeTool;

impl Tool for CodeTool {
    fn visualize(&self, content: &str, file_path: Option<&str>) -> io::Result<()> {
        let language = file_path
            .map(|path| detect_language(path))
            .unwrap_or_else(|| "txt".to_string());
        render_code_file(content, &language)
    }
}

struct ImageTool;

impl Tool for ImageTool {
    fn visualize(&self, _content: &str, file_path: Option<&str>) -> io::Result<()> {
        if let Some(path) = file_path {
            render_image_file(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No file path provided for image rendering",
            ))
        }
    }
}

// Add this function to src/multi_tool.rs
pub fn determine_tool_names(file_path: &Path) -> Vec<String> {
    let extension = file_path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");

    match extension.to_lowercase().as_str() {
        "md" => vec!["markdown".to_string()],
        "jpg" | "jpeg" | "png" | "gif" | "bmp" => vec!["image".to_string()],
        _ => vec!["code".to_string()],
    }
}
