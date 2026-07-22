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
use crate::utils::line_number_color;
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
    reset_render_state();
    render_node(ast)?;
    render_footnotes()?;
    Ok(())
}

fn reset_render_state() {
    if let Ok(mut current_heading_level) = CURRENT_HEADING_LEVEL.lock() {
        *current_heading_level = 0;
    }
    if let Ok(mut content_indent_level) = CONTENT_INDENT_LEVEL.lock() {
        *content_indent_level = 0;
    }
    if let Ok(mut list_stack) = LIST_STACK.lock() {
        list_stack.clear();
    }
    if let Ok(mut ordered_list_stack) = ORDERED_LIST_STACK.lock() {
        ordered_list_stack.clear();
    }
    if let Ok(mut definitions) = LINK_DEFINITIONS.lock() {
        definitions.clear();
    }
    if let Ok(mut definitions) = shared::LINK_DEFINITIONS.lock() {
        definitions.clear();
    }
    if let Ok(mut footnotes) = shared::FOOTNOTES.lock() {
        footnotes.clear();
    }
}

fn render_node(node: &Value) -> io::Result<()> {
    let config = get_config();

    match node["type"].as_str() {
        Some("root") => render_children(node)?,
        Some("yaml") | Some("toml") => render_frontmatter(node)?,
        Some("heading") => render_heading(node)?,
        Some("paragraph") => render_paragraph(node)?,
        Some("text") => render_text(node)?,
        Some("code") => render_code(node)?,
        Some("table") => render_table(node)?,
        Some("list") => render_list(node)?,
        Some("listItem") => render_list_item(node)?,
        Some("blockquote") => render_blockquote(node)?,
        Some("thematicBreak") => render_thematic_break()?,
        Some("break") => render_break()?,
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

/// Render a YAML/TOML frontmatter block as a compact metadata header.
///
/// The parser hands us the raw block between the `---` fences. We render each
/// flat `key: value` line as an aligned pair — keys in a muted color, and a
/// few well-known keys (`status`, `priority`) get their value colored by
/// meaning. Lines we can't split on `:` (nested/multiline YAML) are printed
/// verbatim so nothing is silently dropped.
fn render_frontmatter(node: &Value) -> io::Result<()> {
    let config = get_config();
    let raw = node["value"].as_str().unwrap_or("");
    let mut stdout = get_stdout();

    let entries: Vec<(&str, &str)> = raw
        .lines()
        .filter(|line| !line.trim().is_empty())
        // YAML uses `key: value`; TOML frontmatter (`+++`) uses `key = value`.
        .map(|line| match line.split_once(':').or_else(|| line.split_once('=')) {
            Some((key, value)) => (key.trim(), value.trim()),
            None => (line.trim(), ""),
        })
        .collect();

    if entries.is_empty() {
        return Ok(());
    }

    let key_width = entries
        .iter()
        .map(|(key, _)| key.chars().count())
        .max()
        .unwrap_or(0);

    println!();
    for (key, value) in &entries {
        print!("{}", get_indent());

        // Dim left gutter bar marks the block as a distinct metadata region
        // without drawing a horizontal rule across the terminal.
        if config.use_colors {
            stdout.set_color(ColorSpec::new().set_dimmed(true))?;
        }
        print!("│ ");
        if config.use_colors {
            stdout.reset()?;
        }

        // Keys are dimmed (not a heading color) so metadata reads as chrome and
        // never gets mistaken for an H1, which is bold cyan.
        if config.use_colors {
            stdout.set_color(ColorSpec::new().set_dimmed(true))?;
        }
        print!("{:<width$}", key, width = key_width);
        if config.use_colors {
            stdout.reset()?;
        }

        if value.is_empty() {
            println!();
            continue;
        }

        print!("   ");
        let value_color = frontmatter_value_color(key, value);
        if config.use_colors {
            let mut spec = ColorSpec::new();
            match value_color {
                Some(color) => {
                    spec.set_fg(Some(color)).set_bold(true);
                }
                None if key.eq_ignore_ascii_case("title") => {
                    spec.set_bold(true);
                }
                None => {}
            }
            stdout.set_color(&spec)?;
        }
        print!("{}", value);
        if config.use_colors {
            stdout.reset()?;
        }
        println!();
    }
    println!();

    Ok(())
}

/// Pick a semantic color for a few well-known frontmatter values so task-style
/// metadata reads at a glance. Returns `None` for keys/values we don't
/// recognize, leaving them in the default color.
fn frontmatter_value_color(key: &str, value: &str) -> Option<Color> {
    let normalized = value.trim().trim_matches('"').to_ascii_lowercase();
    match key.to_ascii_lowercase().as_str() {
        "status" => match normalized.as_str() {
            "done" | "completed" | "complete" | "closed" => Some(Color::Green),
            "in_progress" | "in-progress" | "doing" | "active" | "wip" => Some(Color::Blue),
            "blocked" | "waiting" | "on_hold" | "on-hold" => Some(Color::Red),
            "pending" | "todo" | "open" | "backlog" => Some(Color::Yellow),
            _ => None,
        },
        "priority" => match normalized.as_str() {
            "high" | "urgent" | "critical" | "p0" | "p1" => Some(Color::Red),
            "medium" | "med" | "normal" | "p2" => Some(Color::Yellow),
            "low" | "minor" | "p3" | "p4" => Some(Color::Green),
            _ => None,
        },
        _ => None,
    }
}

fn render_text(node: &Value) -> io::Result<()> {
    let text = node["value"].as_str().unwrap_or("");

    // A newline inside a text value is a soft line break. Split on it so each
    // source line stays on its own line (as pagers like glow render markdown)
    // instead of letting format_text collapse it into a single space.
    for (line_idx, line) in text.split('\n').enumerate() {
        if line_idx > 0 {
            render_break()?;
        }
        print!("{}", format_text(line));
    }
    Ok(())
}

/// Render a single text line to a string, normalizing internal whitespace runs
/// to single spaces while preserving the whitespace at the boundaries of the
/// node. Soft line breaks are handled by the caller (`render_text`), which
/// splits on newlines before calling this.
///
/// Preserving the boundary whitespace keeps inline spans like **bold** or
/// *italic* separated from adjacent text. The parser splits
/// "The **controller** runs" into the text node "The ", a strong node, and the
/// text node " runs"; collapsing those boundary spaces would glue the words
/// together ("Thecontrollerruns").
fn format_text(text: &str) -> String {
    let leading = text.starts_with(char::is_whitespace);
    let trailing = text.ends_with(char::is_whitespace);

    let mut out = String::with_capacity(text.len());
    if leading {
        out.push(' ');
    }

    let mut has_content = false;
    for word in text.split_whitespace() {
        if has_content {
            out.push(' ');
        }
        has_content = true;
        if let Some(emoji) = parse_emoji(word) {
            out.push_str(&emoji);
        } else {
            out.push_str(word);
        }
    }

    // Only emit a trailing space when there was actual content; for a
    // whitespace-only node the leading space above already represents it.
    if trailing && has_content {
        out.push(' ');
    }

    out
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

/// Visible text of a table cell, applying the same inline transforms used when
/// rendering (emoji/whitespace normalization for text, raw value for code) so
/// column widths line up with what is actually printed. Formatting wrappers
/// (strong/emphasis/link/…) contribute their inner text rather than being
/// dropped.
fn cell_display_text(node: &Value) -> String {
    match node["type"].as_str() {
        Some("text") => format_text(node["value"].as_str().unwrap_or("")),
        Some("inlineCode") => node["value"].as_str().unwrap_or("").to_string(),
        _ => {
            let mut out = String::new();
            if let Some(children) = node["children"].as_array() {
                for child in children {
                    out.push_str(&cell_display_text(child));
                }
            }
            out
        }
    }
}

/// Render a table cell's full inline content through the normal renderers so
/// bold/italic/code/links are preserved. The base color is re-applied before
/// each child so plain-text segments keep the cell color even after an
/// emphasis/strong span resets the terminal state.
fn render_table_cell(cell: &Value, base: &ColorSpec) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    if let Some(children) = cell["children"].as_array() {
        for child in children {
            stdout.set_color(base)?;
            render_node(child)?;
        }
    }
    stdout.reset()?;
    Ok(())
}

fn render_table(node: &Value) -> io::Result<()> {
    let config = get_config();

    if let Some(children) = node["children"].as_array() {
        let mut column_widths = Vec::new();

        // Calculate column widths
        for row in children {
            if let Some(cells) = row["children"].as_array() {
                for (i, cell) in cells.iter().enumerate() {
                    let content = cell_display_text(cell).chars().count();
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
                    // Base color for the header row and first column.
                    let mut base = ColorSpec::new();
                    if i == 0 {
                        base.set_fg(Some(Color::Red)).set_bold(true);
                    } else if j == 0 {
                        base.set_fg(Some(Color::Cyan));
                    } else {
                        base.set_fg(Some(Color::White));
                    }

                    // Render the cell's full inline content (bold/italic/code/
                    // links), then right-pad to the column width. Padding is
                    // printed uncolored, which is fine since no background is set.
                    let visible = cell_display_text(cell).chars().count();
                    render_table_cell(cell, &base)?;
                    for _ in visible..column_widths[j] {
                        print!(" ");
                    }

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

fn render_break() -> io::Result<()> {
    // Soft and hard line breaks: end the current line and re-apply the
    // paragraph indent so the continuation aligns (unless inside a list item,
    // matching render_paragraph).
    println!();
    if let Ok(list_stack) = LIST_STACK.lock() {
        if list_stack.is_empty() {
            print!("{}", get_indent());
        }
    }
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

    if !config.render_links {
        render_children(node)?;
    } else {
        // No surrounding spaces here: adjacent text nodes carry their own
        // boundary whitespace (see format_text), so padding here would double
        // the spaces around the link.
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
    // No surrounding spaces here: adjacent text nodes carry their own boundary
    // whitespace (see format_text), so padding here would double the spaces.
    print!("{}", node["value"].as_str().unwrap_or(""));
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
    println!();
    print!(
        "{} {}: ",
        admonition_type.icon(),
        format!("{:?}", admonition_type),
    );
    stdout.reset()?;
    stdout.set_color(
        ColorSpec::new()
            .set_fg(Some(admonition_type.color()))
            .set_italic(true),
    )?;
    println!("{}", content);
    println!();
    stdout.reset()?;

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
            let number_color = line_number_color().unwrap_or(Color::Cyan);
            stdout.set_color(ColorSpec::new().set_fg(Some(number_color)))?;
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

#[cfg(test)]
mod tests {
    use super::format_text;

    #[test]
    fn preserves_space_before_inline_span() {
        // Text node that precedes a **bold** span keeps its trailing space.
        assert_eq!(format_text("The "), "The ");
    }

    #[test]
    fn preserves_space_after_inline_span() {
        // Text node that follows a **bold** span keeps its leading space.
        assert_eq!(format_text(" runs"), " runs");
    }

    #[test]
    fn preserves_both_boundary_spaces() {
        assert_eq!(format_text(" and decides "), " and decides ");
    }

    #[test]
    fn normalizes_internal_whitespace() {
        assert_eq!(format_text("foo   bar\nbaz"), "foo bar baz");
    }

    #[test]
    fn preserves_boundaries_while_normalizing_internal() {
        assert_eq!(format_text("  foo   bar  "), " foo bar ");
    }

    #[test]
    fn whitespace_only_node_collapses_to_single_space() {
        assert_eq!(format_text("   "), " ");
    }

    #[test]
    fn empty_node_stays_empty() {
        assert_eq!(format_text(""), "");
    }

    #[test]
    fn no_boundary_whitespace_is_untouched() {
        assert_eq!(format_text("controller"), "controller");
    }

    #[test]
    fn expands_emoji_shortcodes() {
        assert_eq!(format_text("hello :smile:"), "hello 😄");
    }
}
