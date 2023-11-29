use serde::Deserialize;
use serde::Serialize;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub access_token: String,
    pub include_adult: bool,
    pub enable_cache: bool,
    pub load_on_startup: bool,
    pub save_on_exit: bool,
    pub autosave: bool,
    pub browser_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            access_token: "<Replace this with your TMDB access token>".to_string(),
            include_adult: false,
            enable_cache: false,
            load_on_startup: true,
            save_on_exit: true,
            autosave: false,
            browser_name: "firefox".to_string(),
        }
    }
}

impl Config {
    pub fn save(&self, path: &str) {
        let Ok(json_string) = serde_json::to_string_pretty(self) else {
            eprintln!("ERROR: Tries to serialize the data but something went wrong.");
            return;
        };

        if let Err(reason) = fs::write(path, json_string) {
            eprintln!("ERROR: Writing json to a file failed because of this: {reason}");
        }
    }

    pub fn load(path: &str) -> Config {
        if let Ok(contents) = fs::read_to_string(path) {
            serde_json::from_str(&contents).expect("Erroneous config file")
        } else {
            eprintln!("ERROR: Failed to load config file, generating default config.");
            let config = Self::default();

            let contents = serde_json::to_string(&config).unwrap();
            let _ = fs::write(path, contents);

            config
        }
    }

    pub fn validate_access_token(&self) -> bool {
        if self.access_token.len() != 211 {
            return false
        }
        let mut dots = 0;
        for byte in self.access_token.bytes() {
            if byte == b'.' {
                dots += 1
            }
        }
        return dots == 2
    }
}
