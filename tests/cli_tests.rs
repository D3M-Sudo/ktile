use std::process::Command;

#[test]
fn test_cli_version_flag() {
    let output = Command::new(env!("CARGO_BIN_EXE_ktile"))
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_cli_check_config_valid() {
    let output = Command::new(env!("CARGO_BIN_EXE_ktile"))
        .arg("--check-config")
        .output()
        .expect("Failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configurazione valida:"));
}

#[test]
fn test_cli_check_config_invalid_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_ktile"))
        .arg("--config")
        .arg("/nonexistent/file/ktile.toml")
        .arg("--check-config")
        .output()
        .expect("Failed to execute binary");

    assert_eq!(output.status.code(), Some(3));
}
