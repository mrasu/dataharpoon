use crate::config::mcp_server_config::McpServerConfig;
use regex::Regex;
use serde::Deserialize;
use std::error::Error;
use std::{env, fs};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub mcp_servers: Vec<McpServerConfig>,
}

const CONFIG_FILES: [&str; 2] = ["data_harpoon_config.local.toml", "data_harpoon_config.toml"];

impl Default for Config {
    fn default() -> Self {
        Self {
            mcp_servers: Vec::new(),
        }
    }
}

impl Config {
    pub async fn load_config() -> Result<Config, Box<dyn Error>> {
        let filepath = if let Some(path) = Self::find_config_filepath() {
            path
        } else {
            return Ok(Self::default());
        };

        let config = Self::read_config_file(filepath.clone())
            .map_err(|e| format!("failed to parse {}: {}", filepath, e))?;

        Ok(config)
    }

    fn find_config_filepath() -> Option<String> {
        for file in CONFIG_FILES {
            match fs::exists(file) {
                Ok(true) => return Some(file.to_string()),
                Err(_) => return None,
                _ => (),
            };
        }

        None
    }

    fn read_config_file(filepath: String) -> Result<Config, Box<dyn Error>> {
        let file_content = fs::read_to_string(filepath)?;
        let content = file_content.as_str();

        // Replace `${FOO}` characters to environment variable.
        let reg = Regex::new(r"\$\{(\w+)}").unwrap();

        let content = reg
            .replace_all(content, |cap: &regex::Captures| {
                env::var(&cap[1]).unwrap_or("".to_string())
            })
            .to_string();

        let config = toml::from_str::<Config>(&content)?;

        Ok(config)
    }
}
