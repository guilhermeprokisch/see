use reqwest::blocking::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use tempfile::TempDir;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use url::Url;
use viuer::Config;

static mut CURRENT_HEADING_LEVEL: usize = 0;
static mut CONTENT_INDENT_LEVEL: usize = 0;
static mut LIST_STACK: Vec<usize> = Vec::new();
static mut ORDERED_LIST_STACK: Vec<bool> = Vec::new();
static IMAGE_FOLDER: OnceLock<String> = OnceLock::new();
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static NO_IMAGES: AtomicBool = AtomicBool::new(false);

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut debug_mode = false;
    let mut no_images = false;
    let mut file_path = None;

    // Parse command-line arguments
    for arg in &args[1..] {
        match arg.as_str() {
            "--debug" => debug_mode = true,
            "--no-images" => no_images = true,
            "--help" => {
                return render_help();
            }
            _ => file_path = Some(arg),
        }
    }

    DEBUG_MODE.store(debug_mode, Ordering::Relaxed);
    NO_IMAGES.store(no_images, Ordering::Relaxed);

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

    modify_heading_ast(&mut json);
    modify_list_item_ast(&mut json);

    render_markdown(&json)?;

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
    render_node(ast)
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

    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    print!("{}", get_heading_indent(level));
    render_children(node)?;
    stdout.reset()?;
    println!();

    unsafe {
        CURRENT_HEADING_LEVEL = level;
        CONTENT_INDENT_LEVEL = level;
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

    let syntax = ps
        .find_syntax_by_extension(lang)
        .unwrap_or_else(|| ps.find_syntax_plain_text());
    let mut h = HighlightLines::new(syntax, &ts.themes["Solarized (dark)"]);

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
    unsafe {
        LIST_STACK.push(0);
        ORDERED_LIST_STACK.push(is_ordered);
        // Don't increase CONTENT_INDENT_LEVEL here
    }
    render_children(node)?;
    unsafe {
        LIST_STACK.pop();
        ORDERED_LIST_STACK.pop();
        // No need to decrease CONTENT_INDENT_LEVEL
    }
    Ok(())
}

fn render_list_item(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    print!("{}", get_indent());

    unsafe {
        if let (Some(index), Some(is_ordered)) = (LIST_STACK.last_mut(), ORDERED_LIST_STACK.last())
        {
            *index += 1;
            if *is_ordered {
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
        stdout.reset()?;
    }

    if let Some(checked) = node["checked"].as_bool() {
        render_task_list_item_checkbox(checked)?;
    }

    unsafe {
        CONTENT_INDENT_LEVEL += 1;
    }

    render_children(node)?;

    unsafe {
        CONTENT_INDENT_LEVEL -= 1;
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
    unsafe {
        if LIST_STACK.is_empty() {
            print!("{}", get_indent());
        }
    }
    render_children(node)?;
    println!();
    Ok(())
}

fn get_indent() -> String {
    unsafe { "  ".repeat(CONTENT_INDENT_LEVEL) }
}

fn render_blockquote(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
    print!("{}> ", get_indent());
    unsafe {
        CONTENT_INDENT_LEVEL += 1;
    }
    render_children(node)?;
    unsafe {
        CONTENT_INDENT_LEVEL -= 1;
    }
    stdout.reset()?;
    Ok(())
}

fn render_thematic_break() -> io::Result<()> {
    println!("{}---", get_indent());
    Ok(())
}

fn render_link(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let url = node["url"].as_str().unwrap_or("");

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

    Ok(())
}

fn render_image(node: &Value) -> io::Result<()> {
    if NO_IMAGES.load(Ordering::Relaxed) {
        // If --no-images is set, just print the image alt text
        println!("[Image: {}]", node["alt"].as_str().unwrap_or(""));
        return Ok(());
    }

    let url = node["url"].as_str().unwrap_or("");

    let local_path = if Url::parse(url).is_ok() {
        download_image(url)?
    } else {
        PathBuf::from(url)
    };

    if !local_path.exists() {
        println!("Image file not found: {:?}", local_path);
        return Ok(());
    }

    // Attempt to render the image using viuer
    let config = Config {
        absolute_offset: false,
        width: Some(40),
        height: Some(13),
        ..Default::default()
    };

    match viuer::print_from_file(local_path, &config) {
        Ok(_) => return Ok(()),
        Err(e) => println!("Failed to render image: {}", e),
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

fn render_footnote_reference(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
    print!("[^{}]", node["identifier"].as_str().unwrap_or(""));
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

fn render_definition(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
    println!(
        "{}[{}]: {}",
        get_indent(),
        node["identifier"].as_str().unwrap_or(""),
        node["url"].as_str().unwrap_or("")
    );
    if let Some(title) = node["title"].as_str() {
        println!("{}  \"{}\"", get_indent(), title);
    }
    stdout.reset()?;
    Ok(())
}

fn get_heading_indent(level: usize) -> String {
    "  ".repeat(level - 1)
}

fn render_help() -> io::Result<()> {
    let help_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("main.md");

    if !help_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Help file not found",
        ));
    }

    Command::new(env::current_exe()?).arg(help_path).status()?;

    Ok(())
}
