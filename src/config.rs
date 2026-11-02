use crate::layout::{LayoutEngine, LayoutKind};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;

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
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid TOML: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error("cannot write config {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
}

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(0.1..=0.9).contains(&self.master_ratio) {
            return Err(ConfigError::Invalid(
                "master_ratio must be between 0.1 and 0.9".into(),
            ));
        }
        if self.master_count == 0 || self.master_count > 16 {
            return Err(ConfigError::Invalid(
                "master_count must be between 1 and 16".into(),
            ));
        }
        if self.gap > 128 {
            return Err(ConfigError::Invalid(
                "gap must not exceed 128 pixels".into(),
            ));
        }
        if !["trace", "debug", "info", "warn", "error"].contains(&self.log_level.as_str()) {
            return Err(ConfigError::Invalid(
                "log_level must be trace, debug, info, warn, or error".into(),
            ));
        }
        Ok(())
    }

    pub fn engine(&self) -> LayoutEngine {
        LayoutEngine {
            master_ratio: self.master_ratio,
            master_count: self.master_count,
            gap: self.gap,
        }
    }

    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn write_default(path: &Path) -> Result<(), ConfigError> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let body = toml::to_string_pretty(&Self::default())
            .expect("serializing default config cannot fail");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|source| ConfigError::Write {
                path: path.to_path_buf(),
                source,
            })?;
        if let Err(source) = file
            .write_all(body.as_bytes())
            .and_then(|_| file.sync_all())
        {
            drop(file);
            let _ = fs::remove_file(path);
            return Err(ConfigError::Write {
                path: path.to_path_buf(),
                source,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid() {
        assert!(Config::default().validate().is_ok());
    }

    #[test]
    fn rejects_out_of_range_values() {
        let config = Config {
            master_ratio: 0.99,
            ..Default::default()
        };
        assert!(config.validate().is_err());
        let config = Config {
            gap: 129,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn rejects_invalid_master_count_and_log_level() {
        let config = Config {
            master_count: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            master_count: 17,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = Config {
            log_level: "verbose".into(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn accepts_documented_numeric_boundaries() {
        for ratio in [0.1, 0.9] {
            let config = Config {
                master_ratio: ratio,
                master_count: 16,
                gap: 128,
                ..Default::default()
            };
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn write_default_creates_config_and_refuses_overwrite() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nested/config.toml");

        Config::write_default(&path).unwrap();
        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.master_ratio, Config::default().master_ratio);
        assert!(Config::write_default(&path).is_err());
    }

    #[test]
    fn engine_uses_validated_layout_settings() {
        let config = Config {
            master_ratio: 0.7,
            master_count: 2,
            gap: 3,
            ..Default::default()
        };
        let engine = config.engine();
        assert_eq!(engine.master_ratio, 0.7);
        assert_eq!(engine.master_count, 2);
        assert_eq!(engine.gap, 3);
    }

    #[test]
    fn toml_roundtrip() {
        let serialized = toml::to_string(&Config::default()).unwrap();
        let parsed: Config = toml::from_str(&serialized).unwrap();
        assert!(parsed.validate().is_ok());
        assert_eq!(parsed.master_ratio, Config::default().master_ratio);
    }
}
