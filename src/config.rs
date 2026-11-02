use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use thiserror::Error;
use crate::layout::{LayoutEngine, LayoutKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub layout: LayoutKind,
    pub master_ratio: f64,
    pub master_count: usize,
    pub gap: u32,
    pub respect_workarea: bool,
    pub restore_on_exit: bool,
    pub floating_classes: Vec<String>,
    pub excluded_classes: Vec<String>,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: LayoutKind::VerticalLeft,
            master_ratio: 0.58,
            master_count: 1,
            gap: 8,
            respect_workarea: true,
            restore_on_exit: true,
            floating_classes: vec!["Pavucontrol".into(), "Nm-connection-editor".into()],
            excluded_classes: Vec::new(),
            log_level: "info".into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read config {path}: {source}")]
    Read { path: PathBuf, source: std::io::Error },
    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("cannot write config {path}: {source}")]
    Write { path: PathBuf, source: std::io::Error },
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(0.1..=0.9).contains(&self.master_ratio) {
            return Err(ConfigError::Invalid("master_ratio must be between 0.1 and 0.9".into()));
        }
        if self.master_count == 0 || self.master_count > 16 {
            return Err(ConfigError::Invalid("master_count must be between 1 and 16".into()));
        }
        if self.gap > 128 {
            return Err(ConfigError::Invalid("gap must not exceed 128 pixels".into()));
        }
        if !["trace", "debug", "info", "warn", "error"].contains(&self.log_level.as_str()) {
            return Err(ConfigError::Invalid("log_level must be trace, debug, info, warn, or error".into()));
        }
        Ok(())
    }

    pub fn engine(&self) -> LayoutEngine {
        LayoutEngine { master_ratio: self.master_ratio, master_count: self.master_count, gap: self.gap }
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|source| ConfigError::Read { path: path.to_path_buf(), source })?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn write_default(path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write { path: parent.to_path_buf(), source })?;
        }
        let body = toml::to_string_pretty(&Self::default()).expect("serializing default config cannot fail");
        fs::write(path, body).map_err(|source| ConfigError::Write { path: path.to_path_buf(), source })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() { assert!(Config::default().validate().is_ok()); }

    #[test]
    fn rejects_out_of_range_values() {
        let config = Config { master_ratio: 0.99, ..Default::default() };
        assert!(config.validate().is_err());
        let config = Config { gap: 129, ..Default::default() };
        assert!(config.validate().is_err());
    }

    #[test]
    fn toml_roundtrip() {
        let serialized = toml::to_string(&Config::default()).unwrap();
        let parsed: Config = toml::from_str(&serialized).unwrap();
        assert!(parsed.validate().is_ok());
        assert_eq!(parsed.master_ratio, Config::default().master_ratio);
    }
}
