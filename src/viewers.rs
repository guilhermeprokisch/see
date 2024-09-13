use crate::app;
use crate::config::get_config;
use crate::render::{render_code_file, render_image_file, render_markdown};
use crate::utils::detect_language;
use devicons::{icon_for_file, File, Theme};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct ViewerManager {
    viewers: HashMap<String, Box<dyn Viewer>>,
}

impl ViewerManager {
    pub fn new() -> Self {
        let mut viewer_manager = ViewerManager {
            viewers: HashMap::new(),
        };
        // Register default viewers
        viewer_manager.register_viewer("markdown", Box::new(MarkdownViewer));
        viewer_manager.register_viewer("code", Box::new(CodeViewer));
        viewer_manager.register_viewer("image", Box::new(ImageViewer));
        viewer_manager
    }

    pub fn register_viewer(&mut self, name: &str, viewer: Box<dyn Viewer>) {
        self.viewers.insert(name.to_string(), viewer);
    }

    pub fn visualize(
        &self,
        viewer_names: &[String],
        content: &str,
        file_path: Option<&str>,
    ) -> io::Result<()> {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        let config = get_config();

        // Display file name if available and show_filename is true
        if config.show_filename {
            if let Some(path) = file_path {
                let file_name = Path::new(path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();

                let file = File::new(Path::new(path));
                let icon = icon_for_file(&file, Some(Theme::Dark));
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
                writeln!(stdout)?;
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                writeln!(stdout, "{}  {}", icon.icon, file_name)?;
                stdout.reset()?;
                writeln!(stdout)?;
            }
        }

        for (index, viewer_name) in viewer_names.iter().enumerate() {
            if index > 0 {
                writeln!(stdout)?;
            }
            if let Some(viewer) = self.viewers.get(viewer_name) {
                viewer.visualize(content, file_path)?;
            } else {
                eprintln!("Unknown viewer: {}", viewer_name);
            }
        }
        Ok(())
    }
}

pub trait Viewer {
    fn visualize(&self, content: &str, file_path: Option<&str>) -> io::Result<()>;
}

struct MarkdownViewer;

impl Viewer for MarkdownViewer {
    fn visualize(&self, content: &str, _file_path: Option<&str>) -> io::Result<()> {
        let json = app::parse_and_process_markdown(content)?;
        render_markdown(&json)
    }
}

struct CodeViewer;

impl Viewer for CodeViewer {
    fn visualize(&self, content: &str, file_path: Option<&str>) -> io::Result<()> {
        let language = file_path
            .map(|path| detect_language(path))
            .unwrap_or_else(|| "txt".to_string());
        render_code_file(content, &language)
    }
}

struct ImageViewer;

impl Viewer for ImageViewer {
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

pub fn determine_viewer(file_path: &Path) -> Vec<String> {
    let extension = file_path
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();
    match extension.as_str() {
        "md" => vec!["markdown".to_string()],
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => vec!["image".to_string()],
        _ => vec!["code".to_string()],
    }
}
