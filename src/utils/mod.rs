pub mod ast;
pub mod shared;

mod detect_language;
mod git;
mod highlighter;
mod images;
mod theme;

// pub use emoji::parse_emoji;
pub use detect_language::detect_language;
pub use git::show_git_diff;
pub use highlighter::highlight_code;
pub use images::download_image;
