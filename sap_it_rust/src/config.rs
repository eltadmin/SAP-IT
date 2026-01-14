//! Configuration module for loading server definitions from TOML files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

/// Application configuration containing server definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// List of servers available for connection.
    #[serde(default)]
    pub servers: Vec<Server>,

    /// Global settings.
    #[serde(default)]
    pub settings: Settings,
}

/// Global application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Timeout in seconds for VPN connection attempts.
    #[serde(default = "default_vpn_timeout")]
    pub vpn_timeout_secs: u64,

    /// Timeout in milliseconds for ping checks.
    #[serde(default = "default_ping_timeout")]
    pub ping_timeout_ms: u32,

    /// Number of ping retries before giving up.
    #[serde(default = "default_ping_retries")]
    pub ping_retries: u32,
}

fn default_vpn_timeout() -> u64 {
    30
}

fn default_ping_timeout() -> u32 {
    3000
}

fn default_ping_retries() -> u32 {
    3
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            vpn_timeout_secs: default_vpn_timeout(),
            ping_timeout_ms: default_ping_timeout(),
            ping_retries: default_ping_retries(),
        }
    }
}

/// Server definition with connection details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    /// Display name of the server.
    pub name: String,

    /// SSH connection string (e.g., "root@192.168.0.98").
    /// Empty or None if SSH is not available.
    #[serde(default)]
    pub ssh: Option<String>,

    /// RDP address (IP or hostname).
    pub rdp: String,

    /// VPN connection name as configured in the system.
    pub vpn: String,
}

impl Server {
    /// Check if SSH is available for this server.
    pub fn has_ssh(&self) -> bool {
        self.ssh.as_ref().map(|s| !s.is_empty()).unwrap_or(false)
    }

    /// Get the SSH connection string if available.
    pub fn ssh_string(&self) -> Option<&str> {
        self.ssh.as_ref().filter(|s| !s.is_empty()).map(|s| s.as_str())
    }

    /// Extract the IP address from the SSH connection string.
    pub fn ssh_ip(&self) -> Option<String> {
        self.ssh_string().and_then(|ssh| {
            ssh.split('@').nth(1).map(|s| s.to_string())
        })
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: &PathBuf) -> Result<Self> {
        info!("Loading configuration from: {}", path.display());

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        debug!("Loaded {} servers from config", config.servers.len());

        if config.servers.is_empty() {
            anyhow::bail!("No servers defined in configuration file");
        }

        Ok(config)
    }

    /// Get the default configuration file path.
    pub fn default_path() -> PathBuf {
        // Try user config directory first, then current directory
        if let Some(config_dir) = dirs::config_dir() {
            let app_config = config_dir.join("sap_it").join("servers.toml");
            if app_config.exists() {
                return app_config;
            }
        }

        // Fall back to current directory
        PathBuf::from("servers.toml")
    }

    /// Create a default configuration with example servers.
    pub fn default_config() -> Self {
        Config {
            servers: vec![
                Server {
                    name: "Ilmatex".to_string(),
                    ssh: Some("root@192.168.0.98".to_string()),
                    rdp: "192.168.0.99".to_string(),
                    vpn: "ILMATEX".to_string(),
                },
                Server {
                    name: "Frodexim".to_string(),
                    ssh: None,
                    rdp: "192.168.50.20".to_string(),
                    vpn: "FRODEXIM".to_string(),
                },
                Server {
                    name: "Industrial Technic".to_string(),
                    ssh: Some("root@192.168.100.10".to_string()),
                    rdp: "192.168.100.20".to_string(),
                    vpn: "Industrial Technik".to_string(),
                },
                Server {
                    name: "BG Nova".to_string(),
                    ssh: None,
                    rdp: "192.168.100.20".to_string(),
                    vpn: "Industrial Technik".to_string(),
                },
            ],
            settings: Settings::default(),
        }
    }

    /// Generate a sample configuration file content.
    pub fn sample_toml() -> String {
        let config = Self::default_config();
        toml::to_string_pretty(&config).unwrap_or_else(|_| String::from("# Failed to generate sample"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_has_ssh() {
        let server_with_ssh = Server {
            name: "Test".to_string(),
            ssh: Some("root@192.168.1.1".to_string()),
            rdp: "192.168.1.2".to_string(),
            vpn: "TEST_VPN".to_string(),
        };
        assert!(server_with_ssh.has_ssh());

        let server_without_ssh = Server {
            name: "Test".to_string(),
            ssh: None,
            rdp: "192.168.1.2".to_string(),
            vpn: "TEST_VPN".to_string(),
        };
        assert!(!server_without_ssh.has_ssh());

        let server_empty_ssh = Server {
            name: "Test".to_string(),
            ssh: Some("".to_string()),
            rdp: "192.168.1.2".to_string(),
            vpn: "TEST_VPN".to_string(),
        };
        assert!(!server_empty_ssh.has_ssh());
    }

    #[test]
    fn test_ssh_ip_extraction() {
        let server = Server {
            name: "Test".to_string(),
            ssh: Some("root@192.168.1.100".to_string()),
            rdp: "192.168.1.2".to_string(),
            vpn: "TEST_VPN".to_string(),
        };
        assert_eq!(server.ssh_ip(), Some("192.168.1.100".to_string()));

        let server_no_ssh = Server {
            name: "Test".to_string(),
            ssh: None,
            rdp: "192.168.1.2".to_string(),
            vpn: "TEST_VPN".to_string(),
        };
        assert_eq!(server_no_ssh.ssh_ip(), None);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default_config();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.servers.len(), config.servers.len());
    }

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.vpn_timeout_secs, 30);
        assert_eq!(settings.ping_timeout_ms, 3000);
        assert_eq!(settings.ping_retries, 3);
    }
}
