use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

/// Application configuration
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_defaults")]
    pub defaults: DefaultsConfig,
    #[serde(default = "Config::default_storage")]
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct DefaultsConfig {
    /// Default number of thaw days when freezing
    #[serde(default = "default_thaw_days")]
    pub thaw_days: u32,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    /// Path to tasks.json (defaults to ~/.config/kelvin/tasks.json if not specified)
    #[serde(default)]
    pub data_file: Option<String>,
}

fn default_thaw_days() -> u32 {
    7
}

impl Config {
    fn default_defaults() -> DefaultsConfig {
        DefaultsConfig { thaw_days: 7 }
    }

    fn default_storage() -> StorageConfig {
        StorageConfig { data_file: None }
    }

    /// Loads the configuration file. Returns default values if the file does not exist.
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Kelvin's configuration directory (~/.config/kelvin/)
    pub fn kelvin_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
        Ok(home.join(".config").join("kelvin"))
    }

    fn config_path() -> Result<PathBuf> {
        Ok(Self::kelvin_dir()?.join("config.toml"))
    }

    /// Gets the path to tasks.json (can be overridden in the configuration)
    pub fn data_file_path(&self) -> Result<PathBuf> {
        match &self.storage.data_file {
            Some(custom_path) => {
                let path = PathBuf::from(shellexpand::tilde(custom_path).as_ref());
                Ok(path)
            }
            None => Ok(Self::kelvin_dir()?.join("tasks.json")),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: DefaultsConfig { thaw_days: 7 },
            storage: StorageConfig { data_file: None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.defaults.thaw_days, 7);
        assert!(config.storage.data_file.is_none());
    }

    #[test]
    fn parse_toml_config() {
        let toml_str = r#"
[defaults]
thaw_days = 14

[storage]
data_file = "/tmp/my_tasks.json"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.thaw_days, 14);
        assert_eq!(
            config.storage.data_file.as_deref(),
            Some("/tmp/my_tasks.json")
        );
    }

    #[test]
    fn parse_empty_toml_uses_defaults() {
        let toml_str = "";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.thaw_days, 7);
        assert!(config.storage.data_file.is_none());
    }

    #[test]
    fn custom_data_file_path() {
        let config = Config {
            defaults: DefaultsConfig { thaw_days: 7 },
            storage: StorageConfig {
                data_file: Some("/tmp/custom.json".to_string()),
            },
        };
        let path = config.data_file_path().unwrap();
        assert_eq!(path, PathBuf::from("/tmp/custom.json"));
    }
}
