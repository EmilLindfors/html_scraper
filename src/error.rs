use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[cfg(feature = "toml_config")]
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    #[error("Unsupported config file format. Use .json or .toml")]
    UnsupportedFormat,
    #[error("TOML support is not enabled. Enable the 'toml_config' feature to use TOML configs.")]
    TomlNotEnabled,
}