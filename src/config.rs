use regex::Regex;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Impossibile leggere il file di configurazione '{path}': {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Errore di sintassi TOML in '{path}': {source}")]
    ParseError {
        path: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("Errore di validazione configurazione: {0}")]
    ValidationError(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutType {
    #[default]
    Dwindle,
    MasterStack,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SplitMode {
    #[default]
    LongestSide,
    Alternating,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum SplitTarget {
    #[default]
    Active,
    Mru,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum MasterNewStatus {
    #[default]
    Slave,
    Master,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct GeneralConfig {
    #[serde(default)]
    pub layout: LayoutType,
    #[serde(default)]
    pub follow_monitor_focus: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GapsConfig {
    #[serde(default = "default_gap")]
    pub inner: u32,
    #[serde(default = "default_gap")]
    pub outer: u32,
}

fn default_gap() -> u32 {
    8
}

impl Default for GapsConfig {
    fn default() -> Self {
        Self { inner: 8, outer: 8 }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DwindleConfig {
    #[serde(default)]
    pub split_mode: SplitMode,
    #[serde(default = "default_ratio")]
    pub split_ratio: f32,
    #[serde(default)]
    pub split_target: SplitTarget,
}

fn default_ratio() -> f32 {
    0.5
}

impl Default for DwindleConfig {
    fn default() -> Self {
        Self {
            split_mode: SplitMode::LongestSide,
            split_ratio: 0.5,
            split_target: SplitTarget::Active,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MasterConfig {
    #[serde(default = "default_mfact")]
    pub mfact: f32,
    pub new_status: Option<MasterNewStatus>,
}

fn default_mfact() -> f32 {
    0.55
}

impl Default for MasterConfig {
    fn default() -> Self {
        Self {
            mfact: 0.55,
            new_status: None,
        }
    }
}

impl MasterConfig {
    pub fn resolved_new_status(&self) -> MasterNewStatus {
        self.new_status.unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct BehaviorConfig {
    pub new_is_master: Option<bool>,
    #[serde(default = "default_true")]
    pub single_monitor_only: bool,
    #[serde(default)]
    pub smart_gaps: bool,
    #[serde(default = "default_true")]
    pub respect_monitor_reserved: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ExcludeConfig {
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(default)]
    pub instances: Vec<String>,
    #[serde(default)]
    pub title_regex: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub gaps: GapsConfig,
    #[serde(default)]
    pub dwindle: DwindleConfig,
    #[serde(default)]
    pub master: MasterConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub exclude: ExcludeConfig,
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();
        let content = fs::read_to_string(path_ref).map_err(|e| ConfigError::IoError {
            path: path_str.clone(),
            source: e,
        })?;
        let mut config: Config =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError {
                path: path_str,
                source: e,
            })?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&mut self) -> Result<(), ConfigError> {
        // Validation for split_ratio
        if self.dwindle.split_ratio < 0.25 || self.dwindle.split_ratio > 0.75 {
            return Err(ConfigError::ValidationError(format!(
                "dwindle.split_ratio ({}) fuori dai limiti consentiti [0.25, 0.75]",
                self.dwindle.split_ratio
            )));
        }

        // Validation for mfact
        if self.master.mfact < 0.25 || self.master.mfact > 0.75 {
            return Err(ConfigError::ValidationError(format!(
                "master.mfact ({}) fuori dai limiti consentiti [0.25, 0.75]",
                self.master.mfact
            )));
        }

        // Legacy alias new_is_master conflict check
        match (self.behavior.new_is_master, self.master.new_status) {
            (Some(new_is_master), Some(status)) => {
                let equivalent_status = if new_is_master {
                    MasterNewStatus::Master
                } else {
                    MasterNewStatus::Slave
                };
                if status != equivalent_status {
                    return Err(ConfigError::ValidationError(
                        "Conflitto tra behavior.new_is_master e master.new_status".to_string(),
                    ));
                }
            }
            (Some(new_is_master), None) => {
                let equivalent_status = if new_is_master {
                    MasterNewStatus::Master
                } else {
                    MasterNewStatus::Slave
                };
                self.master.new_status = Some(equivalent_status);
            }
            _ => {}
        }

        // Validate title_regex patterns
        for pattern in &self.exclude.title_regex {
            Regex::new(pattern).map_err(|e| {
                ConfigError::ValidationError(format!(
                    "Espressione regolare non valida in exclude.title_regex ('{}'): {}",
                    pattern, e
                ))
            })?;
        }

        Ok(())
    }
}
