use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    pub static ref LINK_DEFINITIONS: Mutex<HashMap<String, (String, Option<String>)>> =
        Mutex::new(HashMap::new());
    pub static ref FOOTNOTES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn get_link_definition(identifier: &str) -> Option<(String, Option<String>)> {
    LINK_DEFINITIONS.lock().unwrap().get(identifier).cloned()
}

pub fn set_link_definition(identifier: String, url: String, title: Option<String>) {
    LINK_DEFINITIONS
        .lock()
        .unwrap()
        .insert(identifier, (url, title));
}

pub fn set_footnote(identifier: String, content: String) {
    FOOTNOTES.lock().unwrap().insert(identifier, content);
}
