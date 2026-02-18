use serde::{Deserialize, Serialize};

/// Configuration for anodized-fmt formatting behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum line width for spec attributes
    pub max_width: usize,

    /// Number of spaces for indentation
    pub indent: usize,

    /// Trailing comma style in arrays
    pub trailing_comma: TrailingComma,

    /// Rearrange items according to their mandatory order
    pub reorder_spec_items: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TrailingComma {
    /// Always add trailing comma
    Always,
    /// Never add trailing comma
    Never,
    /// Add based on whether items span multiple lines
    Auto,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_width: 100,
            indent: 4,
            trailing_comma: TrailingComma::Always,
            sort_args: true,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;
        let config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;
        Ok(config)
    }

    /// Find and load configuration from standard locations
    pub fn load() -> Result<Self, ConfigError> {
        // Try current directory first
        for filename in &["anodized-fmt.toml", ".anodized-fmt.toml"] {
            let path = std::path::Path::new(filename);
            if path.exists() {
                return Self::from_file(path);
            }
        }

        // Try parent directories
        if let Ok(mut current_dir) = std::env::current_dir() {
            loop {
                for filename in &["anodized-fmt.toml", ".anodized-fmt.toml"] {
                    let path = current_dir.join(filename);
                    if path.exists() {
                        return Self::from_file(&path);
                    }
                }

                if !current_dir.pop() {
                    break;
                }
            }
        }

        // Try home directory
        if let Some(home_dir) = dirs::home_dir() {
            let path = home_dir.join(".config/anodized-fmt/anodized-fmt.toml");
            if path.exists() {
                return Self::from_file(&path);
            }
        }

        // Return default if no config found
        Ok(Self::default())
    }

    /// Generate a default configuration file content
    pub fn default_toml() -> String {
        toml::to_string_pretty(&Self::default()).unwrap_or_default()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
}

mod dirs {
    pub fn home_dir() -> Option<std::path::PathBuf> {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
}
