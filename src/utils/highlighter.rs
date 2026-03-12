use lumis::languages::Language;
use lumis::themes::Theme;
use lumis::{formatter::Formatter, themes, write_highlight, HtmlInlineBuilder, TerminalBuilder};
use std::io::{self, Write};
use std::str::FromStr;
use termcolor::Color;

use crate::config::get_config;

pub fn highlight_code<W: Write>(code: &str, lang: &str, writer: &mut W) -> io::Result<()> {
    let theme = selected_theme().map_err(|e| io::Error::other(e.to_string()))?;
    let formatter = TerminalBuilder::new()
        .lang(Language::from_str(lang).unwrap_or(Language::PlainText))
        .theme(Some(theme))
        .build()
        .map_err(|e| io::Error::other(e.to_string()))?;

    write_highlight(writer, code, formatter).map_err(|e| io::Error::other(e.to_string()))
}

pub fn line_number_color() -> Option<Color> {
    let theme = selected_theme().ok()?;
    let style = theme
        .highlights
        .get("comment")
        .or_else(|| theme.highlights.get("punctuation"))
        .or_else(|| theme.highlights.get("variable"))
        .or_else(|| theme.highlights.values().find(|style| style.fg.is_some()))?;

    style.fg.as_deref().and_then(color_from_hex)
}

fn selected_theme() -> Result<Theme, lumis::themes::ThemeError> {
    let config = get_config();
    selected_theme_by_name(&config.syntax_theme)
}

fn selected_theme_by_name(name: &str) -> Result<Theme, lumis::themes::ThemeError> {
    themes::get(name).or_else(|_| themes::get("github_light"))
}

pub fn highlight_code_html(code: &str, lang: &str, theme_name: &str) -> io::Result<String> {
    let theme = selected_theme_by_name(theme_name).map_err(|e| io::Error::other(e.to_string()))?;
    let formatter = HtmlInlineBuilder::new()
        .lang(Language::from_str(lang).unwrap_or(Language::PlainText))
        .theme(Some(theme))
        .build()
        .map_err(|e| io::Error::other(e.to_string()))?;

    let mut output = Vec::new();
    formatter
        .format(code, &mut output)
        .map_err(|e| io::Error::other(e.to_string()))?;

    String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn color_from_hex(hex: &str) -> Option<Color> {
    if hex.len() != 7 || !hex.starts_with('#') {
        return None;
    }

    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}
