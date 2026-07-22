//! Regression test for #90: table cells that contain inline formatting
//! (bold/italic/code) used to render only the first child's plain text, so any
//! emphasized text — and any text after it — was silently dropped.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Render `markdown` with the real `see` binary and return the visible text,
/// with ANSI/OSC escape sequences stripped.
fn render(name: &str, markdown: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(format!("{name}.md"));
    fs::write(&path, markdown).expect("write input markdown");

    let output = Command::new(env!("CARGO_BIN_EXE_see"))
        .env("SEE_FORCE_INTERACTIVE", "1")
        .arg("--pager=false")
        .arg(&path)
        .output()
        .expect("run see");
    assert!(output.status.success(), "see exited with {:?}", output.status);

    strip_ansi(&String::from_utf8_lossy(&output.stdout))
}

/// Remove ANSI CSI sequences (`ESC [ ... m`) and OSC 8 hyperlinks so assertions
/// can look at the plain text.
fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                for f in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&f) {
                        break;
                    }
                }
            }
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

const TABLE: &str = "| Test | Table |\n| --- | --- |\n| Test **row** | **data** |\n| *Another* test | _another_ data |\n";

#[test]
fn table_cells_keep_bold_and_italic_text() {
    let rendered = render("table_formatting", TABLE);

    // Text that lives inside emphasis/strong spans (and text that follows one
    // in the same cell) must survive.
    for expected in ["Test row", "data", "Another test", "another data"] {
        assert!(
            rendered.contains(expected),
            "expected {expected:?} in rendered table, got: {rendered:?}"
        );
    }
}

#[test]
fn table_cell_starting_with_bold_is_not_empty() {
    // `**data**` has a strong node as its first (and only) child. The old code
    // read `children[0].value`, which is empty for a strong node, dropping the
    // whole cell.
    let rendered = render("table_leading_bold", "| A | B |\n| --- | --- |\n| **x** | y |\n");
    assert!(
        rendered.contains('x'),
        "leading-bold cell should render its text, got: {rendered:?}"
    );
}
