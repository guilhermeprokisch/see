//! End-to-end checks that inline spans render with exactly one space around
//! them — covering both the dropped-space bug around **bold**/*italic* and the
//! double-space regression around `code` spans and links.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Render `markdown` with the real `see` binary and return the visible text,
/// with ANSI/OSC escape sequences stripped. `name` keeps the temp input file
/// unique so tests running in parallel don't clobber each other's input.
fn render(name: &str, markdown: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(format!("{name}.md"));
    fs::write(&path, markdown).expect("write input markdown");

    let output = Command::new(env!("CARGO_BIN_EXE_see"))
        // Force the rich markdown viewer even though stdout is a pipe, and turn
        // the pager off so the rendered output lands on stdout directly.
        .env("SEE_FORCE_INTERACTIVE", "1")
        .arg("--pager=false")
        .arg(&path)
        .output()
        .expect("run see");
    assert!(
        output.status.success(),
        "see exited with {:?}",
        output.status
    );

    strip_ansi(&String::from_utf8_lossy(&output.stdout))
}

/// Remove ANSI CSI sequences (`ESC [ ... m`) and OSC 8 hyperlinks
/// (`ESC ] 8 ; ; ... ST`) so assertions can look at the plain text.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.next() {
            // CSI: consume until a final byte in the range 0x40..=0x7e.
            Some('[') => {
                for f in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&f) {
                        break;
                    }
                }
            }
            // OSC: consume until the string terminator (ESC \) or BEL.
            Some(']') => {
                while let Some(f) = chars.next() {
                    if f == '\u{07}' {
                        break;
                    }
                    if f == '\u{1b}' {
                        if matches!(chars.peek(), Some('\\')) {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

#[test]
fn inline_code_and_links_render_with_single_spaces() {
    let rendered = render(
        "code_and_links",
        "Use `code` now and a [link](http://example.com) here.\n",
    );
    assert!(
        rendered.contains("Use code now and a link here."),
        "expected single spaces around inline code and link, got: {rendered:?}"
    );
    assert!(
        !rendered.contains("  "),
        "rendered output should not contain double spaces, got: {rendered:?}"
    );
}

#[test]
fn bold_and_italic_keep_surrounding_spaces() {
    let rendered = render(
        "bold_and_italic",
        "The **controller** runs and decides *what* next.\n",
    );
    assert!(
        rendered.contains("The controller runs and decides what next."),
        "expected spaces preserved around bold/italic spans, got: {rendered:?}"
    );
}
