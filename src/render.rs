use htmd::HtmlToMarkdown;
use lazy_static::lazy_static;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use base64::{engine::general_purpose, Engine as _};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use url::Url;

use crate::config::get_config;
use crate::constants::DEBUG_MODE;
use crate::utils::download_image;
use crate::utils::highlight_code;
use crate::utils::shared;

lazy_static! {
    static ref CURRENT_HEADING_LEVEL: Mutex<usize> = Mutex::new(0);
    static ref CONTENT_INDENT_LEVEL: Mutex<usize> = Mutex::new(0);
    static ref LIST_STACK: Mutex<Vec<usize>> = Mutex::new(Vec::new());
    static ref ORDERED_LIST_STACK: Mutex<Vec<bool>> = Mutex::new(Vec::new());
    static ref LINK_DEFINITIONS: Mutex<HashMap<String, (String, Option<String>)>> =
        Mutex::new(HashMap::new());
}

pub fn render_markdown(ast: &Value) -> io::Result<()> {
    render_node(ast)?;
    render_footnotes()?;
    Ok(())
}

fn render_node(node: &Value) -> io::Result<()> {
    let config = get_config();

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
        Some("html") => {
            if config.convert_html {
                render_html(node)?
            }
        }
        _ => {
            if DEBUG_MODE.load(Ordering::Relaxed) {
                println!("{}Unsupported node type: {:?}", get_indent(), node["type"]);
            }
        }
    }
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
    let config = get_config();
    let level = node["depth"].as_u64().unwrap_or(1) as usize;
    let mut stdout = get_stdout();

    let color = match level {
        1 => Color::Cyan,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        _ => Color::White,
    };

    println!();
    if config.use_colors {
        stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    }
    print!("{}", get_heading_indent(level));
    render_children(node)?;
    if config.use_colors {
        stdout.reset()?;
    }
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
    let code = node["value"].as_str().unwrap_or("");
    let lang = node["lang"].as_str().unwrap_or("txt");

    let indent = get_indent();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    for line in code.lines() {
        write!(stdout, "{}", indent)?;
        if let Err(e) = highlight_code(line, lang, &mut stdout) {
            // If highlighting fails, fall back to plain text
            writeln!(stdout, "{}", line)?;
            eprintln!(
                "Error highlighting code: {}. Falling back to plain text for this line.",
                e
            );
        }
        stdout.reset()?;
        writeln!(stdout)?;
    }

    writeln!(stdout)?;
    Ok(())
}

fn render_table(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let config = get_config();

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
                // Top border for the first row
                if i == 0 && config.render_table_borders {
                    print_horizontal_border(&column_widths, "┌", "┬", "┐")?;
                }

                print!("{}", get_indent());

                if config.render_table_borders {
                    print!("│ ");
                }

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
                    stdout.reset()?;

                    if config.render_table_borders {
                        if j < cells.len() - 1 {
                            print!(" │ ");
                        } else {
                            print!(" │");
                        }
                    } else if j < cells.len() - 1 {
                        print!("  "); // Add two spaces between columns
                    }
                }

                println!();

                // Print horizontal line after header and between rows
                if config.render_table_borders {
                    print_horizontal_border(&column_widths, "├", "┼", "┤")?;
                }
            }
        }

        // Print bottom border
        if config.render_table_borders {
            print_horizontal_border(&column_widths, "└", "┴", "┘")?;
        }
    }

    Ok(())
}

fn print_horizontal_border(
    column_widths: &[usize],
    left: &str,
    middle: &str,
    right: &str,
) -> io::Result<()> {
    print!("{}", get_indent());
    print!("{}", left);
    for (i, width) in column_widths.iter().enumerate() {
        print!("{}", "─".repeat(width + 2)); // +2 for the padding spaces
        if i < column_widths.len() - 1 {
            print!("{}", middle);
        }
    }
    println!("{}", right);
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

pub fn render_image(node: &Value) -> io::Result<()> {
    let config = get_config();
    if !config.render_images {
        println!("[Image: {}]", node["alt"].as_str().unwrap_or(""));
        return Ok(());
    }

    let url = node["url"].as_str().unwrap_or("");

    if url.starts_with("data:image") {
        // Handle base64 encoded image
        let parts: Vec<&str> = url.split(',').collect();
        if parts.len() == 2 {
            let b64_data = parts[1];
            let decoded = general_purpose::STANDARD
                .decode(b64_data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            // Create a temporary file
            let mut temp_file = tempfile::NamedTempFile::new()?;
            temp_file.write_all(&decoded)?;
            let temp_path = temp_file.into_temp_path();

            render_image_file(temp_path.to_str().unwrap())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid base64 image data",
            ))
        }
    } else {
        render_image_file(url)
    }
}

pub fn render_image_file(path: &str) -> io::Result<()> {
    let config = get_config();
    if !config.render_images {
        println!("[Image: {}]", path);
        return Ok(());
    }

    let local_path = if Url::parse(path).is_ok() {
        match download_image(path) {
            Ok(path) => path,
            Err(_) => return Ok(()), // Silently ignore download errors
        }
    } else {
        PathBuf::from(path)
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

    if let Err(e) = viuer::print_from_file(&local_path, &viuer_config) {
        // Silently ignore errors when rendering images
        if config.debug_mode {
            eprintln!("Error rendering image: {}", e);
        }
    }

    Ok(())
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

fn render_link_reference(node: &Value) -> io::Result<()> {
    let identifier = node["identifier"].as_str().unwrap_or("");

    if let Some((url, title)) = shared::get_link_definition(identifier) {
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

fn render_footnotes() -> io::Result<()> {
    let footnotes = shared::FOOTNOTES.lock().unwrap();
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
        shared::set_footnote(identifier.to_string(), content);
    }

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

pub fn render_code_file(content: &str, mut language: &str) -> io::Result<()> {
    let mut stdout = get_stdout();
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();
    let max_line_num_width = line_count.to_string().len();
    let config = get_config();

    for (i, line) in lines.iter().enumerate() {
        if config.show_line_numbers && config.use_colors {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
            write!(stdout, "{:>width$} │ ", i + 1, width = max_line_num_width)?;
            stdout.reset()?;
        }

        if !config.use_colors {
            language = "txt";
        }

        if let Err(e) = highlight_code(line, language, &mut stdout) {
            // If highlighting fails, fall back to plain text
            writeln!(stdout, "{}", line)?;
            eprintln!(
                "Error highlighting code: {}. Falling back to plain text for this line.",
                e
            );
        }
        writeln!(stdout)?;
    }

    Ok(())
}

pub fn get_indent() -> String {
    if let Ok(content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
        "  ".repeat(*content_indent_level)
    } else {
        String::new() // Return empty string if lock fails
    }
}

fn get_stdout() -> Box<dyn WriteColor> {
    let config = get_config();
    if config.use_colors {
        Box::new(StandardStream::stdout(ColorChoice::Always))
    } else {
        Box::new(StandardStream::stdout(ColorChoice::Never))
    }
}

fn render_html(node: &Value) -> io::Result<()> {
    if let Some(html_content) = node["value"].as_str() {
        let converter = HtmlToMarkdown::new();
        match converter.convert(html_content) {
            Ok(markdown) => {
                // Parse the resulting markdown
                let md_ast = markdown::to_mdast(&markdown, &markdown::ParseOptions::gfm())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

                let md_json: Value = serde_json::from_str(&serde_json::to_string(&md_ast).unwrap())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

                // Render the markdown AST
                render_node(&md_json)?;
            }
            Err(e) => {
                eprintln!("Error converting HTML to Markdown: {}", e);
                // Fallback to rendering raw HTML
                println!("{}", html_content);
            }
        }
    }
    Ok(())
}
