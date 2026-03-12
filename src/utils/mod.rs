pub mod ast;
pub mod shared;

mod detect_language;
mod highlighter;
mod images;

// pub use emoji::parse_emoji;
pub use detect_language::detect_language;
pub use highlighter::highlight_code;
pub use highlighter::line_number_color;
pub use images::download_image;
