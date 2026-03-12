use crate::app::parse_and_process_markdown_with_config;
use crate::config::AppConfig;
use crate::utils::{detect_language_with_extensions, highlight_code_html};
use serde_json::Value;
use std::collections::HashMap;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HtmlRenderOptions {
    pub render_links: bool,
    pub convert_html: bool,
    pub syntax_theme: String,
    pub syntax_extensions: HashMap<String, String>,
}

impl Default for HtmlRenderOptions {
    fn default() -> Self {
        Self {
            render_links: true,
            convert_html: true,
            syntax_theme: "github_light".to_string(),
            syntax_extensions: HashMap::new(),
        }
    }
}

#[derive(Default)]
struct HtmlContext {
    link_definitions: HashMap<String, (String, Option<String>)>,
    footnotes: HashMap<String, String>,
}

impl HtmlContext {
    fn collect(&mut self, node: &Value) {
        match node["type"].as_str() {
            Some("definition") => {
                let identifier = node["identifier"].as_str().unwrap_or("").to_string();
                let url = node["url"].as_str().unwrap_or("").to_string();
                let title = node["title"].as_str().map(str::to_string);
                self.link_definitions.insert(identifier, (url, title));
            }
            Some("footnoteDefinition") => {
                let identifier = node["identifier"].as_str().unwrap_or("").to_string();
                self.footnotes.insert(identifier, node_text(node));
            }
            _ => {}
        }

        if let Some(children) = node["children"].as_array() {
            for child in children {
                self.collect(child);
            }
        }
    }
}

pub fn render_markdown_to_html(content: &str, options: &HtmlRenderOptions) -> io::Result<String> {
    let config = AppConfig {
        render_links: options.render_links,
        convert_html: options.convert_html,
        syntax_theme: options.syntax_theme.clone(),
        syntax_extensions: options.syntax_extensions.clone(),
        ..AppConfig::default()
    };
    let ast = parse_and_process_markdown_with_config(content, &config, false)?;

    let mut context = HtmlContext::default();
    context.collect(&ast);

    let mut html = String::new();
    render_node(&ast, options, &context, &mut html)?;

    if !context.footnotes.is_empty() {
        html.push_str("<section class=\"see-footnotes\"><h2>Footnotes</h2><ol>");
        let mut entries: Vec<_> = context.footnotes.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (identifier, content) in entries {
            html.push_str("<li id=\"fn-");
            html.push_str(&escape_html_attr(identifier));
            html.push_str("\">");
            html.push_str(&escape_html(content));
            html.push_str("</li>");
        }
        html.push_str("</ol></section>");
    }

    Ok(html)
}

pub fn render_code_to_html(
    content: &str,
    language: Option<&str>,
    options: &HtmlRenderOptions,
) -> io::Result<String> {
    let language = language.unwrap_or("txt");
    let highlighted = highlight_code_html(content, language, &options.syntax_theme)
        .unwrap_or_else(|_| escape_html(content));

    Ok(format!(
        "<pre class=\"see-code-block\"><code class=\"language-{}\">{}</code></pre>",
        escape_html_attr(language),
        highlighted
    ))
}

pub fn render_file_to_html(path: impl AsRef<Path>, options: &HtmlRenderOptions) -> io::Result<String> {
    let path = path.as_ref();

    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "md" => render_markdown_to_html(&std::fs::read_to_string(path)?, options),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => Ok(format!(
            "<img src=\"{}\" alt=\"{}\" />",
            escape_html_attr(&path.to_string_lossy()),
            escape_html_attr(
                &path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
            )
        )),
        _ => {
            let content = std::fs::read_to_string(path)?;
            let language = detect_language_with_extensions(
                &path.to_string_lossy(),
                &options.syntax_extensions,
            );
            render_code_to_html(&content, Some(&language), options)
        }
    }
}

fn render_node(
    node: &Value,
    options: &HtmlRenderOptions,
    context: &HtmlContext,
    out: &mut String,
) -> io::Result<()> {
    match node["type"].as_str() {
        Some("root") => render_children(node, options, context, out)?,
        Some("heading") => {
            let level = node["depth"].as_u64().unwrap_or(1).clamp(1, 6);
            out.push_str(&format!("<h{level}>"));
            render_children(node, options, context, out)?;
            out.push_str(&format!("</h{level}>"));
        }
        Some("paragraph") => {
            out.push_str("<p>");
            render_children(node, options, context, out)?;
            out.push_str("</p>");
        }
        Some("text") => out.push_str(&escape_html(node["value"].as_str().unwrap_or(""))),
        Some("emphasis") => wrap_tag("em", node, options, context, out)?,
        Some("strong") => wrap_tag("strong", node, options, context, out)?,
        Some("delete") => wrap_tag("del", node, options, context, out)?,
        Some("inlineCode") => {
            out.push_str("<code>");
            out.push_str(&escape_html(node["value"].as_str().unwrap_or("")));
            out.push_str("</code>");
        }
        Some("code") => {
            let language = node["lang"].as_str().unwrap_or("txt");
            out.push_str(&render_code_to_html(
                node["value"].as_str().unwrap_or(""),
                Some(language),
                options,
            )?);
        }
        Some("blockquote") => {
            out.push_str("<blockquote>");
            render_children(node, options, context, out)?;
            out.push_str("</blockquote>");
        }
        Some("list") => {
            let tag = if node["ordered"].as_bool().unwrap_or(false) {
                "ol"
            } else {
                "ul"
            };
            out.push('<');
            out.push_str(tag);
            out.push('>');
            render_children(node, options, context, out)?;
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
        Some("listItem") => wrap_tag("li", node, options, context, out)?,
        Some("table") => render_table(node, options, context, out)?,
        Some("link") => render_link(node, options, context, out)?,
        Some("linkReference") => render_link_reference(node, options, context, out)?,
        Some("image") => {
            out.push_str("<img src=\"");
            out.push_str(&escape_html_attr(node["url"].as_str().unwrap_or("")));
            out.push_str("\" alt=\"");
            out.push_str(&escape_html_attr(node["alt"].as_str().unwrap_or("")));
            out.push_str("\" />");
        }
        Some("imageReference") => {
            out.push_str("<span class=\"see-image-reference\">![");
            out.push_str(&escape_html(node["alt"].as_str().unwrap_or("")));
            out.push_str("]</span>");
        }
        Some("footnoteReference") => {
            let identifier = node["identifier"].as_str().unwrap_or("");
            out.push_str("<sup><a href=\"#fn-");
            out.push_str(&escape_html_attr(identifier));
            out.push_str("\">");
            out.push_str(&escape_html(identifier));
            out.push_str("</a></sup>");
        }
        Some("html") => {
            let raw = node["value"].as_str().unwrap_or("");
            if options.convert_html {
                out.push_str(raw);
            } else {
                out.push_str(&escape_html(raw));
            }
        }
        Some("thematicBreak") => out.push_str("<hr />"),
        Some("definition") | Some("footnoteDefinition") => {}
        _ => render_children(node, options, context, out)?,
    }

    Ok(())
}

fn render_children(
    node: &Value,
    options: &HtmlRenderOptions,
    context: &HtmlContext,
    out: &mut String,
) -> io::Result<()> {
    if let Some(children) = node["children"].as_array() {
        for child in children {
            render_node(child, options, context, out)?;
        }
    }
    Ok(())
}

fn wrap_tag(
    tag: &str,
    node: &Value,
    options: &HtmlRenderOptions,
    context: &HtmlContext,
    out: &mut String,
) -> io::Result<()> {
    out.push('<');
    out.push_str(tag);
    out.push('>');
    render_children(node, options, context, out)?;
    out.push_str("</");
    out.push_str(tag);
    out.push('>');
    Ok(())
}

fn render_link(
    node: &Value,
    options: &HtmlRenderOptions,
    context: &HtmlContext,
    out: &mut String,
) -> io::Result<()> {
    if !options.render_links {
        return render_children(node, options, context, out);
    }

    out.push_str("<a href=\"");
    out.push_str(&escape_html_attr(node["url"].as_str().unwrap_or("")));
    out.push('"');
    if let Some(title) = node["title"].as_str() {
        out.push_str(" title=\"");
        out.push_str(&escape_html_attr(title));
        out.push('"');
    }
    out.push('>');
    render_children(node, options, context, out)?;
    out.push_str("</a>");
    Ok(())
}

fn render_link_reference(
    node: &Value,
    options: &HtmlRenderOptions,
    context: &HtmlContext,
    out: &mut String,
) -> io::Result<()> {
    let identifier = node["identifier"].as_str().unwrap_or("");
    if let Some((url, title)) = context.link_definitions.get(identifier) {
        let mut link = serde_json::json!({
            "type": "link",
            "url": url,
            "children": node["children"].clone(),
        });
        if let Some(title) = title {
            link["title"] = Value::String(title.clone());
        }
        render_link(&link, options, context, out)
    } else {
        render_children(node, options, context, out)
    }
}

fn render_table(
    node: &Value,
    options: &HtmlRenderOptions,
    context: &HtmlContext,
    out: &mut String,
) -> io::Result<()> {
    out.push_str("<table>");
    if let Some(rows) = node["children"].as_array() {
        for (row_index, row) in rows.iter().enumerate() {
            out.push_str("<tr>");
            if let Some(cells) = row["children"].as_array() {
                for cell in cells {
                    let tag = if row_index == 0 { "th" } else { "td" };
                    out.push('<');
                    out.push_str(tag);
                    out.push('>');
                    render_children(cell, options, context, out)?;
                    out.push_str("</");
                    out.push_str(tag);
                    out.push('>');
                }
            }
            out.push_str("</tr>");
        }
    }
    out.push_str("</table>");
    Ok(())
}

fn node_text(node: &Value) -> String {
    match node["type"].as_str() {
        Some("text") => node["value"].as_str().unwrap_or("").to_string(),
        _ => node["children"]
            .as_array()
            .map(|children| children.iter().map(node_text).collect::<Vec<_>>().join(""))
            .unwrap_or_default(),
    }
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_html_attr(input: &str) -> String {
    escape_html(input).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_markdown_heading_without_terminal_suffix() {
        let html = render_markdown_to_html("# Title", &HtmlRenderOptions::default()).unwrap();
        assert_eq!(html, "<h1>Title</h1>");
    }
}
