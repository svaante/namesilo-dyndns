use anyhow::{Context, Result};
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILENAME: &str = ".namesilo-dyndns.toml";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub namesilo_api_key: String,
    pub domain: String,
    pub poll_duration_s: u64,
    pub ip_fetchers: Vec<String>
}

impl Config {
    pub fn read() -> Result<Self> {
        let config_path = Config::config_path()?;
        let config_string = fs::read_to_string(config_path)?;

        Ok(Config::from_str(&config_string)?)
    }

    pub fn example_config() -> Self {
        Config {
            namesilo_api_key: "12345".to_string(),
            domain: "example.com".to_string(),
            poll_duration_s: 900,
            ip_fetchers: vec!["example.com".to_string()],
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(env::home_dir()
            .context("$HOME env not set")?
            .join(CONFIG_FILENAME))
    }

    fn from_str(config_string: &str) -> Result<Config> {
        Ok(toml::from_str(config_string)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let config_string = r#"
namesilo_api_key = "12345"
domain = "example.com"
poll_duration = 900
ip_fetchers = ["example.com"]
"#;
        assert_eq!(Config::from_str(&config_string), Config::example_config());
    }
}
