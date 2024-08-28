use serde_json::Value;
use std::env;
use std::fs;
use std::io::Write;
use std::io::{self};
use std::process;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

static mut CURRENT_HEADING_LEVEL: usize = 0;
static mut CONTENT_INDENT_LEVEL: usize = 0;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Error reading file {}: {}", file_path, error);
            process::exit(1);
        }
    };

    let ast = markdown::to_mdast(&content, &markdown::ParseOptions::gfm()).unwrap();
    let json: Value = serde_json::from_str(&serde_json::to_string(&ast).unwrap()).unwrap();

    render_markdown(&json)?;

    Ok(())
}

fn render_markdown(ast: &Value) -> io::Result<()> {
    render_node(ast)
}

fn render_node(node: &Value) -> io::Result<()> {
    match node["type"].as_str() {
        Some("root") => render_children(node)?,
        Some("heading") => {
            println!("");
            render_heading(node)?
        }
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
        _ => println!("{}Unsupported node type: {:?}", get_indent(), node["type"]),
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

fn render_paragraph(node: &Value) -> io::Result<()> {
    print!("{}", get_indent());
    render_children(node)?;
    println!();
    Ok(())
}

fn render_text(node: &Value) -> io::Result<()> {
    print!("{}", node["value"].as_str().unwrap_or(""));
    Ok(())
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

    // Print language in italic gray
    stdout.set_color(
        ColorSpec::new()
            .set_fg(Some(Color::Ansi256(242)))
            .set_italic(true),
    )?;
    println!("{}{}", get_indent(), lang);
    stdout.reset()?;

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

    println!();
    Ok(())
}

fn render_list(node: &Value) -> io::Result<()> {
    render_children(node)?;
    println!();
    Ok(())
}

fn render_list_item(node: &Value) -> io::Result<()> {
    print!("{}* ", get_indent());
    unsafe {
        CONTENT_INDENT_LEVEL += 1;
    }
    render_children(node)?;
    unsafe {
        CONTENT_INDENT_LEVEL -= 1;
    }
    Ok(())
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
    println!();
    Ok(())
}

fn render_thematic_break() -> io::Result<()> {
    println!("{}---", get_indent());
    Ok(())
}

fn render_link(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(
        ColorSpec::new()
            .set_fg(Some(Color::Blue))
            .set_underline(true),
    )?;
    print!("[");
    render_children(node)?;
    print!("]({})", node["url"].as_str().unwrap_or(""));
    stdout.reset()?;
    Ok(())
}

fn render_image(node: &Value) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
    print!("![");
    render_children(node)?;
    print!("]({})", node["url"].as_str().unwrap_or(""));
    stdout.reset()?;
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

fn render_children(node: &Value) -> io::Result<()> {
    if let Some(children) = node["children"].as_array() {
        for child in children {
            render_node(child)?;
        }
    }
    Ok(())
}

fn get_heading_indent(level: usize) -> String {
    "  ".repeat(level - 1)
}

fn get_indent() -> String {
    unsafe { "  ".repeat(CONTENT_INDENT_LEVEL) }
}
