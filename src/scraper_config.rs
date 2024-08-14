
use serde::{Deserialize, Serialize};
use std::{fmt::Display, fs, path::Path,};

use crate::ConfigError;

pub trait ScrapeConfig: for<'de> Deserialize<'de> + Sized {
    fn get_config() -> ScraperConfig;

    fn from_config(config: &str) -> Result<ScraperConfig, ConfigError> {
        if Path::new(config).exists() {
            let config_content = fs::read_to_string(config)?;
            if config.ends_with(".json") {
                Ok(serde_json::from_str(&config_content)?)
            } else if config.ends_with(".toml") {
                #[cfg(feature = "toml_config")]
                {
                    Ok(toml::from_str(&config_content)?)
                }
                #[cfg(not(feature = "toml_config"))]
                {
                    Err(ConfigError::TomlNotEnabled)
                }
            } else {
                Err(ConfigError::UnsupportedFormat)
            }
        } else {
            // Try parsing as JSON first, then TOML if that fails and the feature is enabled
            serde_json::from_str(config).or_else(|_| {
                #[cfg(feature = "toml_config")]
                {
                    toml::from_str(config).map_err(|e| e.into())
                }
                #[cfg(not(feature = "toml_config"))]
                {
                    Err(ConfigError::UnsupportedFormat)
                }
            })
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScrapeRule {
    One {
        selector: String,
        name: String,
        #[serde(default)]
        sub_rules: Option<Vec<ScrapeRule>>,
        #[serde(default)]
        attribute: Option<String>,
    },
    All {
        selector: String,
        name: String,
        #[serde(default)]
        sub_rules: Option<Vec<ScrapeRule>>,
        #[serde(default)]
        attribute: Option<String>,
    },
    Text {
        selector: String,
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScraperConfig {
    pub(crate) rules: Vec<ScrapeRule>,
}

impl ScraperConfig {
    pub fn new(rules: Vec<ScrapeRule>) -> Self {
        ScraperConfig { rules }
    }
}

impl Display for ScraperConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OutputConfig {
    #[serde(rename = "type")]
    output_type: String,
}