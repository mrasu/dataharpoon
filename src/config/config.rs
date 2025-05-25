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

const CONFIG_FILE_LOCAL_PATH: &str = ".data_harpoon_config.local.toml";
const CONFIG_FILE_PATH: &str = ".data_harpoon_config.toml";

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
        match fs::exists(CONFIG_FILE_LOCAL_PATH) {
            Ok(true) => return Some(CONFIG_FILE_LOCAL_PATH.to_string()),
            Err(_) => return None,
            _ => (),
        };
        match fs::exists(CONFIG_FILE_PATH) {
            Ok(true) => return Some(CONFIG_FILE_PATH.to_string()),
            Err(_) => return None,
            _ => (),
        };

        None
    }

    fn read_config_file(filepath: String) -> Result<Config, Box<dyn Error>> {
        let file_content = fs::read_to_string(filepath)?;
        let content = file_content.as_str();

        // Replace `${FOO}` characters to environment variable.
        let reg = Regex::new(r"\$\{(\w+)}").unwrap();

        for cap in reg.captures_iter(content) {
            env::var(&cap[1]).map_err(|_| {
                format!("Environment variable in config not found. ENV={}", &cap[1])
            })?;
        }

        let content = reg
            .replace_all(content, |cap: &regex::Captures| env::var(&cap[1]).unwrap())
            .to_string();

        let config = toml::from_str::<Config>(&content)?;

        Ok(config)
    }
}
