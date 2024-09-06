use include_dir::{include_dir, Dir};
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;

pub static IMAGE_FOLDER: OnceLock<String> = OnceLock::new();
pub static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
pub static NO_IMAGES: AtomicBool = AtomicBool::new(false);
pub static DOCS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/docs");
