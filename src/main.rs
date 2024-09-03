extern crate lazy_static;

use dirs::home_dir;
use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::RwLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use tempfile::TempDir;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use url::Url;

static IMAGE_FOLDER: OnceLock<String> = OnceLock::new();
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static NO_IMAGES: AtomicBool = AtomicBool::new(false);
static DOCS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/docs");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub theme: String,
    pub code_highlight_theme: String,
    pub max_image_width: Option<u32>,
    pub max_image_height: Option<u32>,
    pub render_images: bool,
    pub render_links: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            theme: "default".to_string(),
            code_highlight_theme: "Solarized (dark)".to_string(),
            max_image_width: Some(40),
            max_image_height: Some(13),
            render_images: true,
            render_links: true,
        }
    }
}

// Global storage
lazy_static! {
    static ref CURRENT_HEADING_LEVEL: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    static ref CONTENT_INDENT_LEVEL: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    static ref LIST_STACK: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));
    static ref ORDERED_LIST_STACK: Arc<Mutex<Vec<bool>>> = Arc::new(Mutex::new(Vec::new()));
    static ref LINK_DEFINITIONS: Mutex<HashMap<String, (String, Option<String>)>> =
        Mutex::new(HashMap::new());
    static ref FOOTNOTES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref GLOBAL_CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

fn load_config() {
    let config_dir = if cfg!(target_os = "macos") {
        home_dir()
            .map(|path| path.join(".config"))
            .unwrap_or_else(|| PathBuf::from("~/.config"))
    } else {
        dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"))
    };

    let config_path = config_dir.join("smd").join("config.toml");

    let config = if config_path.exists() {
        match fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(parsed_config) => parsed_config,
                Err(e) => {
                    eprintln!(
                        "Failed to parse config file: {}. Using default configuration.",
                        e
                    );
                    Config::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read config file: {}. Using default configuration.",
                    e
                );
                Config::default()
            }
        }
    } else {
        eprintln!(
            "Config file not found at {:?}. Using default configuration.",
            config_path
        );
        Config::default()
    };

    let mut global_config = GLOBAL_CONFIG.write().unwrap();
    *global_config = config;
}

fn get_config() -> Config {
    GLOBAL_CONFIG.read().unwrap().clone()
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut debug_mode = false;
    let mut file_path = None;

    // Parse command-line arguments
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
                return generate_default_config();
            }
            _ => file_path = Some(arg),
        }
    }

    // Load the configuration
    load_config();

    DEBUG_MODE.store(debug_mode, Ordering::Relaxed);
    NO_IMAGES.store(get_config().render_images, Ordering::Relaxed);

    let content = if let Some(path) = file_path {
        // Read from file
        fs::read_to_string(path)?
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    // Create a temporary directory for images
    let temp_dir = TempDir::new()?;
    let image_folder = temp_dir.path().to_str().unwrap().to_string();

    // Set the global image folder
    IMAGE_FOLDER.set(image_folder).unwrap();

    let ast = markdown::to_mdast(&content, &markdown::ParseOptions::gfm()).unwrap();
    let mut json: Value = serde_json::from_str(&serde_json::to_string(&ast).unwrap()).unwrap();

    process_definitions(&json);
    process_footnotes(&json);
    modify_heading_ast(&mut json);
    modify_list_item_ast(&mut json);

    render_markdown(&json)?;

    LINK_DEFINITIONS.lock().unwrap().clear();
    Ok(())
}

fn render_node(node: &Value) -> io::Result<()> {
    match node["type"].as_str() {
        Some("root") => render_children(node)?,
        Some("heading") => render_heading(node)?,
        Some("paragraph") => render_paragraph(node)?,
        Some("text") => render_text(node)?,
        Some("code") => render_code(node)?,
        Some("table") => render_table(node)?,
        Some("list") => render_list(node)?,
        Some("listItem") => render_list_item(node)?,
        Some("blockquote") => render_blockquote(node)?,
        Some("thematicBreak") => render_thematic_break()?,
        Some("link") => render_link(node)?,
        Some("image") => render_image(node)?,
        Some("emphasis") => render_emphasis(node)?,
        Some("strong") => render_strong(node)?,
        Some("delete") => render_delete(node)?,
        Some("inlineCode") => render_inline_code(node)?,
        Some("footnoteReference") => render_footnote_reference(node)?,
        Some("imageReference") => render_image_reference(node)?,
        Some("definition") => render_definition(node)?,
        Some("linkReference") => render_link_reference(node)?,
        _ => {
            if DEBUG_MODE.load(Ordering::Relaxed) {
                println!("{}Unsupported node type: {:?}", get_indent(), node["type"]);
            }
        }
    }
    Ok(())
}

fn modify_list_item_ast(node: &mut Value) {
    if node["type"] == "listItem" {
        if let Some(children) = node["children"].as_array_mut() {
            if children.len() == 1 && children[0]["type"] == "paragraph" {
                // Replace the listItem's children with the paragraph's children
                node["children"] = children[0]["children"].clone();
            }
        }
    }

    // Recursively modify children
    if let Some(children) = node["children"].as_array_mut() {
        for child in children {
            modify_list_item_ast(child);
        }
    }
}

fn modify_heading_ast(node: &mut Value) {
    if node["type"] == "heading" {
        if let Some(children) = node["children"].as_array_mut() {
            if let Some(last_child) = children.last_mut() {
                if last_child["type"] == "text" {
                    if let Some(text) = last_child["value"].as_str() {
                        last_child["value"] = Value::String(format!("{}:", text));
                    }
                }
            }
        }
    }

    // Recursively modify children
    if let Some(children) = node["children"].as_array_mut() {
        for child in children {
            modify_heading_ast(child);
        }
    }
}

fn render_markdown(ast: &Value) -> io::Result<()> {
    render_node(ast)?;
    render_footnotes()?;
    Ok(())
}

fn render_children(node: &Value) -> io::Result<()> {
    if let Some(children) = node["children"].as_array() {
        for child in children {
            render_node(child)?;
        }
    }
    Ok(())
}

fn render_heading(node: &Value) -> io::Result<()> {
    let level = node["depth"].as_u64().unwrap_or(1) as usize;
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    let color = match level {
        1 => Color::Cyan,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        _ => Color::White,
    };

    println!();
    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    print!("{}", get_heading_indent(level));
    render_children(node)?;
    stdout.reset()?;
    println!();

    if let Ok(mut current_heading_level) = CURRENT_HEADING_LEVEL.lock() {
        *current_heading_level = level;
    }
    if let Ok(mut content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
        *content_indent_level = level;
    }

    Ok(())
}

fn render_text(node: &Value) -> io::Result<()> {
    let text = node["value"].as_str().unwrap_or("");
    let words: Vec<&str> = text.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        if let Some(emoji) = parse_emoji(word) {
            print!("{}", emoji);
        } else {
            print!("{}", word);
        }
    }
    Ok(())
}

fn parse_emoji(word: &str) -> Option<String> {
    if word.len() >= 2 && word.starts_with(':') && word.ends_with(':') {
        let emoji_name = &word[1..word.len() - 1];
        if let Some(emoji) = emojis::get_by_shortcode(emoji_name) {
            return Some(emoji.as_str().to_string());
        }
    }
    None
}

fn render_code(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let code = node["value"].as_str().unwrap_or("");
    let lang = node["lang"].as_str().unwrap_or("txt");

    // Load these once at the start of your program
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let config = get_config();
    let syntax = ps
        .find_syntax_by_extension(lang)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &ts.themes[&config.code_highlight_theme]);

    // Print language in italic gray # TODO: Make this optional
    // stdout.set_color(
    //     ColorSpec::new()
    //         .set_fg(Some(Color::Ansi256(242)))
    //         .set_italic(true),
    // )?;
    // println!("{}{}", get_indent(), lang);
    // stdout.reset()?;

    // Add extra indentation for code content
    let code_indent = get_indent() + "  ";

    for line in LinesWithEndings::from(code) {
        let highlighted = match h.highlight_line(line, &ps) {
            Ok(h) => h,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
        };

        print!("{}", code_indent);
        for (style, text) in highlighted.iter() {
            let color = style_to_termcolor(style);
            stdout.set_color(ColorSpec::new().set_fg(color))?;
            write!(stdout, "{}", text)?;
        }
        stdout.reset()?;
    }
    println!();

    Ok(())
}

fn style_to_termcolor(style: &Style) -> Option<Color> {
    if style.foreground.a == 0 {
        None
    } else {
        Some(Color::Rgb(
            style.foreground.r,
            style.foreground.g,
            style.foreground.b,
        ))
    }
}

fn render_table(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if let Some(children) = node["children"].as_array() {
        let mut column_widths = Vec::new();

        // Calculate column widths
        for row in children {
            if let Some(cells) = row["children"].as_array() {
                for (i, cell) in cells.iter().enumerate() {
                    let content = cell["children"][0]["value"].as_str().unwrap_or("").len();
                    if i >= column_widths.len() {
                        column_widths.push(content);
                    } else if content > column_widths[i] {
                        column_widths[i] = content;
                    }
                }
            }
        }

        // Render table
        for (i, row) in children.iter().enumerate() {
            if let Some(cells) = row["children"].as_array() {
                print!("{}", get_indent());
                for (j, cell) in cells.iter().enumerate() {
                    let content = cell["children"][0]["value"].as_str().unwrap_or("");

                    // Set color for header row and first column
                    if i == 0 {
                        stdout
                            .set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                    } else if j == 0 {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                    } else {
                        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                    }

                    print!("{:<width$}", content, width = column_widths[j]);

                    if j < cells.len() - 1 {
                        print!("  "); // Add two spaces between columns
                    }
                }
                println!();
                stdout.reset()?;
            }
        }
    }

    Ok(())
}

fn render_list(node: &Value) -> io::Result<()> {
    let is_ordered = node["ordered"].as_bool().unwrap_or(false);
    if let Ok(mut list_stack) = LIST_STACK.lock() {
        list_stack.push(0);
    }
    if let Ok(mut ordered_list_stack) = ORDERED_LIST_STACK.lock() {
        ordered_list_stack.push(is_ordered);
    }
    render_children(node)?;
    if let Ok(mut list_stack) = LIST_STACK.lock() {
        list_stack.pop();
    }
    if let Ok(mut ordered_list_stack) = ORDERED_LIST_STACK.lock() {
        ordered_list_stack.pop();
    }
    Ok(())
}

fn render_list_item(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    print!("{}", get_indent());

    {
        let mut list_stack = LIST_STACK.lock().unwrap();
        let ordered_list_stack = ORDERED_LIST_STACK.lock().unwrap();

        if let Some(index) = list_stack.last_mut() {
            *index += 1;
            if *ordered_list_stack.last().unwrap_or(&false) {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                print!("{:2}. ", *index);
            } else {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                print!("• ");
            }
        } else {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
            print!("• ");
        }
    }
    stdout.reset()?;

    if let Some(checked) = node["checked"].as_bool() {
        render_task_list_item_checkbox(checked)?;
    }

    if let Ok(mut content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
        *content_indent_level += 1;
    }

    render_children(node)?;

    if let Ok(mut content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
        *content_indent_level -= 1;
    }

    println!(); // Add a newline after each list item
    Ok(())
}

fn render_task_list_item_checkbox(checked: bool) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if checked {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        print!("  ");
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        print!("  ");
    }
    stdout.reset()?;
    Ok(())
}

fn render_paragraph(node: &Value) -> io::Result<()> {
    // Only print indent if it's not inside a list item
    if let Ok(list_stack) = LIST_STACK.lock() {
        if list_stack.is_empty() {
            print!("{}", get_indent());
        }
    }
    render_children(node)?;
    println!();
    Ok(())
}

fn get_indent() -> String {
    if let Ok(content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
        "  ".repeat(*content_indent_level)
    } else {
        String::new() // Return empty string if lock fails
    }
}

fn render_thematic_break() -> io::Result<()> {
    // TODO: I don't like rulers in the terminal, maybe it can be optional?
    // let mut stdout = StandardStream::stdout(ColorChoice::Always);
    // stdout.set_color(ColorSpec::new().set_fg(Some(Color::Black)))?;
    //
    // let width = 80; // You can adjust this value or make it dynamic based on terminal width
    // let line = "─".repeat(width);
    //
    // println!("{}{}", get_indent(), line);
    //
    // stdout.reset()?;
    Ok(())
}

fn render_link(node: &Value) -> io::Result<()> {
    let config = get_config();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let url = node["url"].as_str().unwrap_or("");

    if config.render_links {
        render_children(node)?;
    } else {
        // Add a space before the link reference
        print!(" ");
        // Start OSC 8 hyperlink
        print!("\x1B]8;;{}\x1B\\", url);

        stdout.set_color(
            ColorSpec::new()
                .set_fg(Some(Color::Blue))
                .set_underline(true),
        )?;

        render_children(node)?;

        stdout.reset()?;

        // End OSC 8 hyperlink
        print!("\x1B]8;;\x1B\\");

        // Add a space after the link reference
        print!(" ");
    }

    Ok(())
}

fn render_image(node: &Value) -> io::Result<()> {
    let config = get_config();
    if !config.render_images {
        println!("[Image: {}]", node["alt"].as_str().unwrap_or(""));
        return Ok(());
    }

    let url = node["url"].as_str().unwrap_or("");

    let local_path = if Url::parse(url).is_ok() {
        match download_image(url) {
            Ok(path) => path,
            Err(_) => return Ok(()), // Silently ignore download errors
        }
    } else {
        PathBuf::from(url)
    };

    if !local_path.exists() {
        return Ok(()); // Silently ignore if the file doesn't exist
    }

    let viuer_config = viuer::Config {
        absolute_offset: false,
        width: config.max_image_width,
        height: config.max_image_height,
        ..Default::default()
    };

    if let Err(_) = viuer::print_from_file(&local_path, &viuer_config) {
        // Silently ignore rendering errors
    }

    Ok(())
}

fn download_image(url: &str) -> io::Result<PathBuf> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download image: HTTP {}", response.status()),
        ));
    }

    let content = response
        .bytes()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Generate a filename based on the hash of the content
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    let filename = format!("{:x}.jpg", hash); // Assuming JPG, adjust as needed

    let image_folder = IMAGE_FOLDER.get().expect("Image folder not set");
    let path = Path::new(image_folder).join(filename);
    fs::write(&path, content)?;

    Ok(path)
}

fn render_emphasis(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_italic(true))?;
    render_children(node)?;
    stdout.reset()?;
    Ok(())
}

fn render_strong(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_bold(true))?;
    render_children(node)?;
    stdout.reset()?;
    Ok(())
}

fn render_delete(node: &Value) -> io::Result<()> {
    if let Some(children) = node["children"].as_array() {
        for child in children {
            if child["type"] == "text" {
                if let Some(text) = child["value"].as_str() {
                    for c in text.chars() {
                        print!("{}\u{0336}", c);
                    }
                }
            } else {
                render_node(child)?;
            }
        }
    }
    Ok(())
}

fn render_inline_code(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)).set_bold(true))?;
    print!(" {} ", node["value"].as_str().unwrap_or(""));
    stdout.reset()?;
    Ok(())
}

fn render_image_reference(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
    print!(
        "![{}][{}]",
        node["alt"].as_str().unwrap_or(""),
        node["identifier"].as_str().unwrap_or("")
    );
    stdout.reset()?;
    Ok(())
}

fn get_heading_indent(level: usize) -> String {
    "  ".repeat(level - 1)
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

fn process_definitions(node: &Value) {
    if let Some("definition") = node["type"].as_str() {
        render_definition(node).unwrap();
    }

    if let Some(children) = node["children"].as_array() {
        for child in children {
            process_definitions(child);
        }
    }
}

fn render_link_reference(node: &Value) -> io::Result<()> {
    let identifier = node["identifier"].as_str().unwrap_or("");
    let definitions = LINK_DEFINITIONS.lock().unwrap();

    if let Some((url, title)) = definitions.get(identifier) {
        // Create a temporary link node
        let link_node = json!({
            "type": "link",
            "url": url,
            "title": title,
            "children": node["children"].clone()
        });

        // Render as a regular link
        render_link(&link_node)?;
    } else {
        // If definition is not found, render as plain text
        render_children(node)?;
    }

    Ok(())
}

fn render_definition(node: &Value) -> io::Result<()> {
    let identifier = node["identifier"].as_str().unwrap_or("");
    let url = node["url"].as_str().unwrap_or("");
    let title = node["title"].as_str().map(|s| s.to_string());

    let mut definitions = LINK_DEFINITIONS.lock().unwrap();
    definitions.insert(identifier.to_string(), (url.to_string(), title));

    Ok(())
}

fn process_footnotes(node: &Value) {
    if let Some("footnoteDefinition") = node["type"].as_str() {
        let identifier = node["identifier"].as_str().unwrap_or("");
        let mut content = String::new();
        if let Some(children) = node["children"].as_array() {
            for child in children {
                content.push_str(&node_to_string(child));
            }
        }
        FOOTNOTES
            .lock()
            .unwrap()
            .insert(identifier.to_string(), content);
    }

    if let Some(children) = node["children"].as_array() {
        for child in children {
            process_footnotes(child);
        }
    }
}

fn node_to_string(node: &Value) -> String {
    let mut content = String::new();
    if let Some("text") = node["type"].as_str() {
        content.push_str(node["value"].as_str().unwrap_or(""));
    } else if let Some(children) = node["children"].as_array() {
        for child in children {
            content.push_str(&node_to_string(child));
        }
    }
    content
}

// Add a new function to render all footnotes
fn render_footnotes() -> io::Result<()> {
    let footnotes = FOOTNOTES.lock().unwrap();
    if footnotes.is_empty() {
        return Ok(());
    }

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_italic(true))?;
    println!("Footnotes:");
    for (identifier, content) in footnotes.iter() {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_italic(true))?;
        print!("{}: ", identifier);
        stdout.reset()?;
        println!("{}", content);
    }
    println!();

    Ok(())
}

// Modify the render_footnote_reference function
fn render_footnote_reference(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let identifier = node["identifier"].as_str().unwrap_or("");
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)).set_italic(true))?;
    print!(" [^{}]", identifier);
    stdout.reset()?;

    // Store the footnote content
    if let Some(children) = node["children"].as_array() {
        let content = children
            .iter()
            .filter_map(|child| child["value"].as_str())
            .collect::<Vec<&str>>()
            .join(" ");
        FOOTNOTES
            .lock()
            .unwrap()
            .insert(identifier.to_string(), content);
    }

    Ok(())
}

fn print_version() -> io::Result<()> {
    println!("smd version {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

#[derive(Debug, PartialEq)]
enum AdmonitionType {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

impl AdmonitionType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "NOTE" => Some(AdmonitionType::Note),
            "TIP" => Some(AdmonitionType::Tip),
            "IMPORTANT" => Some(AdmonitionType::Important),
            "WARNING" => Some(AdmonitionType::Warning),
            "CAUTION" => Some(AdmonitionType::Caution),
            _ => None,
        }
    }

    fn color(&self) -> Color {
        match self {
            AdmonitionType::Note => Color::Cyan,
            AdmonitionType::Tip => Color::Green,
            AdmonitionType::Important => Color::Magenta,
            AdmonitionType::Warning => Color::Yellow,
            AdmonitionType::Caution => Color::Red,
        }
    }

    fn icon(&self) -> &str {
        match self {
            AdmonitionType::Note => " ",
            AdmonitionType::Tip => " ",
            AdmonitionType::Important => " ",
            AdmonitionType::Warning => " ",
            AdmonitionType::Caution => " ",
        }
    }
}

fn parse_admonition(node: &Value) -> Option<(AdmonitionType, String)> {
    if node["type"] != "blockquote" {
        return None;
    }

    if let Some(children) = node["children"].as_array() {
        if let Some(first_child) = children.first() {
            if first_child["type"] == "paragraph" {
                if let Some(paragraph_children) = first_child["children"].as_array() {
                    if let Some(text_node) = paragraph_children.first() {
                        if text_node["type"] == "text" {
                            if let Some(text) = text_node["value"].as_str() {
                                if text.trim().starts_with("[!") && text.contains("]") {
                                    let end = text.find("]").unwrap();
                                    let admonition_str = &text[2..end];
                                    if let Some(admonition) =
                                        AdmonitionType::from_str(admonition_str)
                                    {
                                        let content = text[end + 1..].trim().to_string()
                                            + &paragraph_children[1..]
                                                .iter()
                                                .filter_map(|node| node["value"].as_str())
                                                .collect::<Vec<_>>()
                                                .join(" ");
                                        return Some((admonition, content));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

fn render_admonition(admonition_type: AdmonitionType, content: &str) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(
        ColorSpec::new()
            .set_fg(Some(admonition_type.color()))
            .set_bold(true),
    )?;
    print!(
        "{} {}: ",
        admonition_type.icon(),
        format!("{:?}", admonition_type)
    );
    stdout.reset()?;

    println!("{}", content);
    Ok(())
}

fn render_blockquote(node: &Value) -> io::Result<()> {
    if let Some((admonition_type, content)) = parse_admonition(node) {
        render_admonition(admonition_type, &content)
    } else {
        // Existing blockquote rendering logic
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
        print!("{}> ", get_indent());

        if let Ok(mut content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
            *content_indent_level += 1;
        }

        render_children(node)?;

        if let Ok(mut content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
            *content_indent_level -= 1;
        }

        stdout.reset()?;
        Ok(())
    }
}

fn generate_default_config() -> io::Result<()> {
    let config_dir = if cfg!(target_os = "macos") {
        home_dir()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "Could not find home directory")
            })?
            .join(".config")
    } else {
        dirs::config_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "Could not find config directory")
        })?
    };

    let smd_config_dir = config_dir.join("smd");

    // Create all directories in the path if they don't exist
    fs::create_dir_all(&smd_config_dir)?;

    let config_path = smd_config_dir.join("config.toml");

    let default_config = Config::default();
    let toml = toml::to_string_pretty(&default_config).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to serialize config: {}", e),
        )
    })?;

    fs::write(&config_path, toml)?;

    println!("Default configuration file created at {:?}", config_path);
    Ok(())
}
