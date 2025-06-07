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

impl Default for Config {
    fn default() -> Self {
        Self {
            mcp_servers: Vec::new(),
        }
    }
}

impl Config {
    pub async fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
        if !fs::exists(path).map_err(|e| format!("failed to access {}: {:?}", path, e))? {
            return Err(format!("config file not found: {}", path).into());
        };

        let config =
            Self::read_config_file(path).map_err(|e| format!("failed to parse {}: {}", path, e))?;

        Ok(config)
    }

    fn read_config_file(filepath: &str) -> Result<Config, Box<dyn Error>> {
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
