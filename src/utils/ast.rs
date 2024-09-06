use crate::utils::shared;

use lazy_static::lazy_static;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref LINK_DEFINITIONS: Mutex<HashMap<String, (String, Option<String>)>> =
        Mutex::new(HashMap::new());
    static ref FOOTNOTES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub fn process_definitions(node: &Value) {
    if let Some("definition") = node["type"].as_str() {
        let identifier = node["identifier"].as_str().unwrap_or("");
        let url = node["url"].as_str().unwrap_or("");
        let title = node["title"].as_str().map(|s| s.to_string());

        shared::set_link_definition(identifier.to_string(), url.to_string(), title);
    }

    if let Some(children) = node["children"].as_array() {
        for child in children {
            process_definitions(child);
        }
    }
}

pub fn process_footnotes(node: &Value) {
    if let Some("footnoteDefinition") = node["type"].as_str() {
        let identifier = node["identifier"].as_str().unwrap_or("");
        let mut content = String::new();
        if let Some(children) = node["children"].as_array() {
            for child in children {
                content.push_str(&node_to_string(child));
            }
        }
        shared::set_footnote(identifier.to_string(), content);
    }

    if let Some(children) = node["children"].as_array() {
        for child in children {
            process_footnotes(child);
        }
    }
}

pub fn modify_heading_ast(node: &mut Value) {
    if node["type"] == "heading" {
        if let Some(children) = node["children"].as_array_mut() {
            if let Some(last_child) = children.last_mut() {
                if last_child["type"] == "text" {
                    if let Some(text) = last_child["value"].as_str() {
                        last_child["value"] = Value::String(format!("{}:", text));
                    }
                }
            }
        }
    }

    if let Some(children) = node["children"].as_array_mut() {
        for child in children {
            modify_heading_ast(child);
        }
    }
}

pub fn modify_list_item_ast(node: &mut Value) {
    if node["type"] == "listItem" {
        if let Some(children) = node["children"].as_array_mut() {
            if children.len() == 1 && children[0]["type"] == "paragraph" {
                node["children"] = children[0]["children"].clone();
            }
        }
    }

    if let Some(children) = node["children"].as_array_mut() {
        for child in children {
            modify_list_item_ast(child);
        }
    }
}

fn node_to_string(node: &Value) -> String {
    let mut content = String::new();
    if let Some("text") = node["type"].as_str() {
        content.push_str(node["value"].as_str().unwrap_or(""));
    } else if let Some(children) = node["children"].as_array() {
        for child in children {
            content.push_str(&node_to_string(child));
        }
    }
    content
}
