use crate::config::get_config;
use hyperpolyglot::detect;
use std::collections::HashMap;
use std::path::Path;

pub fn detect_language(path: &str) -> String {
    let config = get_config();
    detect_language_with_extensions(path, &config.syntax_extensions)
}

pub fn detect_language_with_extensions(
    path: &str,
    syntax_extensions: &HashMap<String, String>,
) -> String {
    let path = Path::new(path);

    if let Some(language) = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| language_for_extension(ext, syntax_extensions))
    {
        return language;
    }

    // Use hyperpolyglot for language detection
    match detect(path) {
        Ok(Some(detection)) => {
            let lang = detection.language().to_lowercase();
            match lang.as_str() {
                "shell" => "bash".to_string(),
                _ => lang,
            }
        }
        Ok(None) | Err(_) => {
            // Fallback to extension-based detection if hyperpolyglot fails
            path.extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| fallback_language_for_extension(ext))
                .unwrap_or("txt")
                .to_string()
        }
    }
}

fn language_for_extension(
    extension: &str,
    syntax_extensions: &HashMap<String, String>,
) -> Option<String> {
    let normalized_extension = normalize_extension(extension);

    syntax_extensions
        .get(&normalized_extension)
        .cloned()
        .or_else(|| fallback_language_for_extension(&normalized_extension).map(str::to_string))
}

fn fallback_language_for_extension(extension: &str) -> Option<&'static str> {
    match normalize_extension(extension).as_str() {
        "md" => Some("md"),
        "rs" => Some("rust"),
        "py" => Some("python"),
        "js" => Some("javascript"),
        "html" => Some("html"),
        "css" => Some("css"),
        "json" => Some("json"),
        "c" => Some("c"),
        "cc" | "cpp" | "cxx" | "ino" => Some("cpp"),
        _ => None,
    }
}

fn normalize_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_detect_shell_script() {
        let path = "./utils/script.sh";
        assert_eq!(
            detect_language_with_extensions(path, &HashMap::new()),
            "bash"
        )
    }

    #[test]
    fn test_detect_cpp_fallback_extensions() {
        assert_eq!(fallback_language_for_extension("cpp"), Some("cpp"));
        assert_eq!(fallback_language_for_extension("cxx"), Some("cpp"));
        assert_eq!(fallback_language_for_extension("ino"), Some("cpp"));
    }

    #[test]
    fn test_custom_extension_override() {
        let mut syntax_extensions = HashMap::new();
        syntax_extensions.insert("pde".to_string(), "cpp".to_string());

        assert_eq!(
            detect_language_with_extensions("./sketch.pde", &syntax_extensions),
            "cpp"
        );
    }
}
