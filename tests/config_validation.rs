use ktile::config::{Config, LayoutType, MasterNewStatus, SplitMode, SplitTarget};
use std::fs;

#[test]
fn test_default_config_validation() {
    let mut config = Config::default();
    assert!(config.validate().is_ok());
    assert_eq!(config.general.layout, LayoutType::Dwindle);
    assert_eq!(config.gaps.inner, 8);
    assert_eq!(config.gaps.outer, 8);
    assert_eq!(config.dwindle.split_mode, SplitMode::LongestSide);
    assert_eq!(config.dwindle.split_ratio, 0.5);
    assert_eq!(config.dwindle.split_target, SplitTarget::Active);
    assert_eq!(config.master.mfact, 0.55);
    assert_eq!(config.master.resolved_new_status(), MasterNewStatus::Slave);
}

#[test]
fn test_invalid_split_ratio() {
    let mut config = Config::default();
    config.dwindle.split_ratio = 0.1;
    assert!(config.validate().is_err());

    config.dwindle.split_ratio = 0.9;
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_mfact() {
    let mut config = Config::default();
    config.master.mfact = 0.1;
    assert!(config.validate().is_err());

    config.master.mfact = 0.8;
    assert!(config.validate().is_err());
}

#[test]
fn test_new_is_master_alias_and_conflict() {
    let mut config = Config::default();
    config.behavior.new_is_master = Some(true);
    assert!(config.validate().is_ok());
    assert_eq!(config.master.resolved_new_status(), MasterNewStatus::Master);

    let mut config_conflict = Config::default();
    config_conflict.master.new_status = Some(MasterNewStatus::Slave);
    config_conflict.behavior.new_is_master = Some(true);
    assert!(config_conflict.validate().is_err());
}

#[test]
fn test_toml_parsing() {
    let toml_content = r#"
[general]
layout = "master-stack"

[gaps]
inner = 10
outer = 12

[dwindle]
split_mode = "alternating"
split_ratio = 0.6
split_target = "mru"

[master]
mfact = 0.5
new_status = "master"

[exclude]
classes = ["kupfer.py"]
instances = ["conky"]
title_regex = ["^np\\.vc\\.virtualbox$"]
"#;

    let file_path = std::env::temp_dir().join("test_ktile_valid.toml");
    fs::write(&file_path, toml_content).unwrap();

    let loaded = Config::load_from_file(&file_path);
    assert!(loaded.is_ok());

    let cfg = loaded.unwrap();
    assert_eq!(cfg.general.layout, LayoutType::MasterStack);
    assert_eq!(cfg.gaps.inner, 10);
    assert_eq!(cfg.gaps.outer, 12);
    assert_eq!(cfg.dwindle.split_mode, SplitMode::Alternating);
    assert_eq!(cfg.dwindle.split_ratio, 0.6);
    assert_eq!(cfg.dwindle.split_target, SplitTarget::Mru);
    assert_eq!(cfg.master.mfact, 0.5);
    assert_eq!(cfg.master.resolved_new_status(), MasterNewStatus::Master);
    assert_eq!(cfg.exclude.classes, vec!["kupfer.py"]);

    let _ = fs::remove_file(file_path);
}
