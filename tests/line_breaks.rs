//! End-to-end checks that line breaks within a paragraph render on separate
//! lines: soft breaks (a single source newline) and hard breaks (two trailing
//! spaces or a backslash). Regression test for #84, where soft breaks were
//! silently collapsed into the surrounding text.

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

#[test]
fn soft_line_breaks_stay_on_separate_lines() {
    // Single newlines within one paragraph (soft breaks). Previously these were
    // collapsed, gluing the lines together.
    let rendered = render(
        "soft_breaks",
        "**Date:** 2026-06-22\n**Work item:** here\n**Design:** there\n",
    );
    assert!(
        rendered.contains("Date: 2026-06-22\n"),
        "first line should end at the soft break, got: {rendered:?}"
    );
    assert!(
        rendered.contains("Work item: here\n"),
        "second line should be on its own line, got: {rendered:?}"
    );
    assert!(
        rendered.contains("Design: there"),
        "third line should be on its own line, got: {rendered:?}"
    );
    assert!(
        !rendered.contains("2026-06-22Work item"),
        "soft-broken lines must not be glued together, got: {rendered:?}"
    );
}

#[test]
fn hard_line_breaks_stay_on_separate_lines() {
    // Two trailing spaces before the newline is a hard break.
    let rendered = render("hard_breaks", "line one  \nline two\n");
    assert!(
        rendered.contains("line one\nline two"),
        "hard break should split the two lines, got: {rendered:?}"
    );
}
