use lumis::languages::Language;
use lumis::{themes, write_highlight, TerminalBuilder};
use std::io::{self, Write};
use std::str::FromStr;

use crate::config::get_config;

pub fn highlight_code<W: Write>(code: &str, lang: &str, writer: &mut W) -> io::Result<()> {
    let config = get_config();
    let theme = themes::get(&config.syntax_theme)
        .or_else(|_| themes::get("github_light"))
        .map_err(|e| io::Error::other(e.to_string()))?;
    let formatter = TerminalBuilder::new()
        .lang(Language::from_str(lang).unwrap_or(Language::PlainText))
        .theme(Some(theme))
        .build()
        .map_err(|e| io::Error::other(e.to_string()))?;

    write_highlight(writer, code, formatter).map_err(|e| io::Error::other(e.to_string()))
}
