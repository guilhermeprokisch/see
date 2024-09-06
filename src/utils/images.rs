use crate::constants::IMAGE_FOLDER;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn download_image(url: &str) -> io::Result<PathBuf> {
    let client = Client::new();
    let response = client
        .get(url)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download image: HTTP {}", response.status()),
        ));
    }

    let content = response
        .bytes()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Generate a filename based on the hash of the content
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let hash = hasher.finalize();
    let filename = format!("{:x}.jpg", hash); // Assuming JPG, adjust as needed

    let image_folder = IMAGE_FOLDER.get().expect("Image folder not set");
    let path = Path::new(image_folder).join(filename);
    fs::write(&path, content)?;

    Ok(path)
}
