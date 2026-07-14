//! End-to-end checks that YAML frontmatter renders as a metadata header rather
//! than being misparsed. Without the frontmatter construct enabled, a closing
//! `---` turns the block into a setext heading, so `id: "001"` etc. rendered as
//! one bold heading line. These tests pin the metadata-block behavior.

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

const TASK_DOC: &str = "---\nid: \"001\"\ntitle: \"My first task\"\nstatus: pending\npriority: medium\n---\n\n# My First Task\n\nBody text.\n";

#[test]
fn frontmatter_keys_render_on_their_own_lines() {
    let rendered = render("frontmatter_keys", TASK_DOC);
    // Each key sits on its own line with its value, not collapsed into one
    // heading line the way the setext-heading misparse produced.
    for expected in ["id", "title", "status", "priority"] {
        assert!(
            rendered.lines().any(|line| {
                // Each metadata line is `│ <key> ...`; strip the gutter first.
                line.trim_start()
                    .trim_start_matches('│')
                    .trim_start()
                    .starts_with(expected)
            }),
            "expected a metadata line for {expected:?}, got: {rendered:?}"
        );
    }
    assert!(
        rendered.contains("pending"),
        "status value should be shown, got: {rendered:?}"
    );
}

#[test]
fn frontmatter_fences_are_not_rendered() {
    // The `---` fences must be consumed by the parser, never printed as text.
    let rendered = render("frontmatter_fences", TASK_DOC);
    assert!(
        !rendered.contains("---"),
        "frontmatter fences should not appear in output, got: {rendered:?}"
    );
}

#[test]
fn frontmatter_does_not_swallow_following_heading() {
    // Regression: the closing `---` used to fold the body's first heading into
    // the frontmatter. The real document heading must still render.
    let rendered = render("frontmatter_heading", TASK_DOC);
    assert!(
        rendered.contains("My First Task"),
        "document heading should still render, got: {rendered:?}"
    );
}
