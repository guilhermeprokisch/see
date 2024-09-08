use hyperpolyglot::detect;
use std::path::Path;

pub fn detect_language(path: &str) -> String {
    let path = Path::new(path);

    // First, check if it's a Markdown file by extension
    if let Some(extension) = path.extension() {
        if extension == "md" {
            return "md".to_string();
        }
    }

    // Use hyperpolyglot for language detection
    match detect(path) {
        Ok(Some(detection)) => detection.language().to_lowercase(),
        Ok(None) | Err(_) => {
            // Fallback to extension-based detection if hyperpolyglot fails
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| match ext {
                    "rs" => "rust",
                    "py" => "python",
                    "js" => "javascript",
                    "html" => "html",
                    "css" => "css",
                    "json" => "json",
                    _ => "txt",
                })
                .unwrap_or("txt")
                .to_string()
        }
    }
}
