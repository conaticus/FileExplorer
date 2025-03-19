use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub background_color: String,
    pub font_size: i64,
    pub enable: bool
}

pub fn load_config(filename: &str) -> Result<Config, String> {
    if !Path::new(filename).exists() {
        return Err(format!("Config file '{}' not found!", filename));
    }

    let config_data = fs::read_to_string(filename).map_err(|e| format!("Failed to read config: {}", e))?;
    let config: Config = serde_json::from_str(&config_data).map_err(|e| format!("Invalid JSON format: {}", e))?;

    Ok(config)
}