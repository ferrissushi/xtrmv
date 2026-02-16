use std::collections::HashMap;
use std::fs;

pub fn load_filetype(path: &str) -> HashMap<String, String> {
    let data = fs::read_to_string(path).expect("Failed to read filetype.json");
    let map: HashMap<String, String> =
        serde_json::from_str(&data).expect("Failed to parse filetype.json");
    map
}

pub fn get_filetype(filename: &str, map: &HashMap<String, String>) -> String {
    let ext = filename.split('.').next_back().unwrap();
    map.get(ext).unwrap_or(&"Unknown".to_string()).to_string()
}

pub fn load_keywords(filetype: &str) -> Vec<String> {
    let path = format!("src/{}.json", filetype);
    let data = match fs::read_to_string(&path) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let map: HashMap<String, Vec<String>> = match serde_json::from_str(&data) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };
    map.get("keywords").cloned().unwrap_or_default()
}
