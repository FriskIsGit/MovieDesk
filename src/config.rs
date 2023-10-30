use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::fs::read_to_string;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api_key: String,
    pub include_adult: bool,
    pub enable_cache: bool,
    pub browser_name: String,
}

impl Config {
    pub fn read_config(path: &str) -> Config {
        let contents = read_to_string(path).expect("File not found");
        serde_json::from_str(&contents).expect("Erroneous config file")
    }
}
