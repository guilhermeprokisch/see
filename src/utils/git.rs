use crate::render::get_indent;
use crate::utils::detect_language;
use std::cmp;
use std::io::{self, Write};
use std::process::Command;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn show_git_diff(file_path: &str) -> io::Result<()> {
    let output = Command::new("git")
        .args(&["diff", "--", file_path])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr),
        ));
    }

    let diff_output = String::from_utf8_lossy(&output.stdout);
    print_styled_diff(&diff_output, file_path)
}

fn print_styled_diff(diff: &str, file_path: &str) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let language = detect_language(file_path);
    let mut in_diff_block = false;
    let mut diff_block = Vec::new();
    let mut old_line_num = 0;
    let mut new_line_num = 0;

    for line in diff.lines() {
        if line.starts_with("@@") {
            if in_diff_block {
                render_diff_block(&diff_block, &language, old_line_num, new_line_num)?;
                diff_block.clear();
            }
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
            writeln!(&mut stdout, "{}{}", get_indent(), line)?;
            stdout.reset()?;
            in_diff_block = true;

            // Parse line numbers
            let parts: Vec<&str> = line.split(' ').collect();
            if parts.len() >= 3 {
                let old_range: Vec<&str> = parts[1][1..].split(',').collect();
                let new_range: Vec<&str> = parts[2][1..].split(',').collect();
                old_line_num = old_range[0].parse::<usize>().unwrap_or(1);
                new_line_num = new_range[0].parse::<usize>().unwrap_or(1);
            }
        } else if line.starts_with('+') || line.starts_with('-') || line.starts_with(' ') {
            diff_block.push(line.to_string());
        } else {
            if in_diff_block {
                render_diff_block(&diff_block, &language, old_line_num, new_line_num)?;
                diff_block.clear();
                in_diff_block = false;
            }
            writeln!(&mut stdout, "{}{}", get_indent(), line)?;
        }
    }

    if in_diff_block {
        render_diff_block(&diff_block, &language, old_line_num, new_line_num)?;
    }

    Ok(())
}

fn render_diff_block(
    block: &[String],
    language: &str,
    mut old_line_num: usize,
    mut new_line_num: usize,
) -> io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut old_lines = Vec::new();
    let mut new_lines = Vec::new();

    for line in block {
        if line.starts_with('-') {
            old_lines.push((old_line_num, line[1..].to_string()));
            old_line_num += 1;
        } else if line.starts_with('+') {
            new_lines.push((new_line_num, line[1..].to_string()));
            new_line_num += 1;
        } else {
            old_lines.push((old_line_num, line[1..].to_string()));
            new_lines.push((new_line_num, line[1..].to_string()));
            old_line_num += 1;
            new_line_num += 1;
        }
    }

    let max_lines = cmp::max(old_lines.len(), new_lines.len());
    let terminal_width = 120; // Adjust this based on your terminal width
    let half_width = terminal_width / 2 - 2;

    for i in 0..max_lines {
        // Left side (old)
        if i < old_lines.len() {
            let (line_num, content) = &old_lines[i];
            render_diff_line(&mut stdout, *line_num, content, Color::Red, half_width)?;
        } else {
            write!(&mut stdout, "{:width$}", "", width = half_width)?;
        }

        // Separator
        write!(&mut stdout, " │ ")?;

        // Right side (new)
        if i < new_lines.len() {
            let (line_num, content) = &new_lines[i];
            render_diff_line(&mut stdout, *line_num, content, Color::Green, half_width)?;
        }

        writeln!(&mut stdout)?;
    }

    Ok(())
}

fn render_diff_line(
    stdout: &mut StandardStream,
    line_num: usize,
    content: &str,
    color: Color,
    width: usize,
) -> io::Result<()> {
    let bg_color = match color {
        Color::Red => Color::Rgb(255, 200, 200),
        Color::Green => Color::Rgb(200, 255, 200),
        _ => Color::White, // Default background color for unchanged lines
    };

    stdout.set_color(ColorSpec::new().set_bg(Some(bg_color)).set_fg(Some(color)))?;
    write!(stdout, "{:4} ", line_num)?;

    let mut content_width = 0;
    for (idx, c) in content.char_indices() {
        if content_width >= width - 5 {
            write!(stdout, "…")?;
            break;
        }
        write!(stdout, "{}", c)?;
        content_width += if c == '\t' { 4 } else { 1 };
    }

    // Pad the rest of the line
    if content_width < width - 5 {
        write!(stdout, "{:width$}", "", width = width - 5 - content_width)?;
    }

    stdout.reset()?;
    Ok(())
}
