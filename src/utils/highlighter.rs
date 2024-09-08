use inkjet::constants::HIGHLIGHT_NAMES;
use inkjet::formatter::{Formatter, Theme};
use inkjet::tree_sitter_highlight::HighlightEvent;
use inkjet::{Highlighter, Language, Result as InkjetResult};
use std::io::{self, Write};

use std::cell::RefCell;
use std::fmt::Write as FmtWrite;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::utils::theme::create_theme;

pub fn highlight_code<W: Write>(code: &str, lang: &str, writer: &mut W) -> io::Result<()> {
    let mut highlighter = Highlighter::new();
    let language = Language::from_token(lang).unwrap_or(Language::Plaintext);
    let theme = create_theme();
    let formatter = TerminalFormatter::new(theme);

    for line in code.lines() {
        highlighter
            .highlight_to_writer(language, &formatter, line, writer)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    }

    Ok(())
}

struct TerminalFormatter {
    theme: Theme,
    stdout: RefCell<StandardStream>,
}

impl TerminalFormatter {
    fn new(theme: Theme) -> Self {
        Self {
            theme,
            stdout: RefCell::new(StandardStream::stdout(ColorChoice::Always)),
        }
    }

    fn color_from_hex(&self, hex: &str) -> Color {
        let rgb = color_from_hex(hex).unwrap_or((255, 255, 255));
        Color::Rgb(rgb.0, rgb.1, rgb.2)
    }
}

impl Formatter for TerminalFormatter {
    fn write<W>(&self, source: &str, _writer: &mut W, event: HighlightEvent) -> InkjetResult<()>
    where
        W: FmtWrite,
    {
        match event {
            HighlightEvent::Source { start, end } => {
                let text = &source[start..end];
                let mut stdout = self.stdout.borrow_mut();
                stdout.write_all(text.as_bytes())?;
                stdout.flush()?;
            }
            HighlightEvent::HighlightStart(highlight) => {
                let style_name = HIGHLIGHT_NAMES[highlight.0];
                let style = self.theme.get_style(style_name);
                let color = self.color_from_hex(&style.primary_color);
                self.stdout
                    .borrow_mut()
                    .set_color(ColorSpec::new().set_fg(Some(color)))?;
            }
            HighlightEvent::HighlightEnd => {
                self.stdout.borrow_mut().reset()?;
            }
        }
        Ok(())
    }
}

fn color_from_hex(hex: &str) -> Option<(u8, u8, u8)> {
    if hex.len() != 7 || !hex.starts_with('#') {
        return None;
    }

    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

    Some((r, g, b))
}
