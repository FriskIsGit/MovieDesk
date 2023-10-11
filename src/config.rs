use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    pub include_adult: bool,
    pub enable_cache: bool,
}

impl Config {
    pub fn read_config(path: &str) -> Config {
        let contents: String = fs::read_to_string(path).expect("File not found");
        let a_str: &str = contents.as_str();
        let config: Config = serde_json::from_str(a_str).unwrap();
        config
    }
}
