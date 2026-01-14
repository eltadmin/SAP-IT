//! Integration tests for SAP-IT.

use std::process::Command;
use tempfile::TempDir;

/// Test that the CLI displays help correctly.
#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("sap_it"));
    assert!(stdout.contains("Server connection manager"));
    assert!(stdout.contains("--config"));
    assert!(stdout.contains("--verbose"));
}

/// Test that the CLI displays version correctly.
#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--", "--version"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("2.0.0"));
}

/// Test the init subcommand creates a config file.
#[test]
fn test_init_creates_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("test_servers.toml");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "init",
            "--output",
            config_path.to_str().unwrap(),
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Init command failed");
    assert!(config_path.exists(), "Config file was not created");

    // Verify the content is valid TOML
    let content = std::fs::read_to_string(&config_path).expect("Failed to read config");
    assert!(content.contains("[[servers]]"));
    assert!(content.contains("name"));
    assert!(content.contains("vpn"));
}

/// Test the list subcommand works with a config file.
#[test]
fn test_list_with_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("servers.toml");

    // Create a test config
    let config_content = r#"
[[servers]]
name = "TestServer"
rdp = "192.168.1.1"
vpn = "TEST_VPN"
"#;

    std::fs::write(&config_path, config_content).expect("Failed to write config");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "list",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command failed");
    assert!(stdout.contains("TestServer"));
    assert!(stdout.contains("TEST_VPN"));
}

/// Test that invalid config file is handled gracefully.
#[test]
fn test_invalid_config_error() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("invalid.toml");

    // Create an invalid config
    std::fs::write(&config_path, "this is not valid toml {{{").expect("Failed to write config");

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "--config",
            config_path.to_str().unwrap(),
            "list",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "Should have failed with invalid config"
    );
}

/// Test connect with invalid server name.
#[test]
fn test_connect_invalid_server() {
    let output = Command::new("cargo")
        .args(["run", "--", "connect", "NonExistentServer"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found") || stderr.contains("NonExistentServer"));
}

/// Test connect with invalid connection type.
#[test]
fn test_connect_invalid_type() {
    let output = Command::new("cargo")
        .args(["run", "--", "connect", "1", "--connection-type", "invalid"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid connection type") || stderr.contains("invalid"));
}
