//! Connection management module with graceful shutdown support.

use crate::config::{Server, Settings};
use crate::platform;
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Connection type options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionType {
    Rdp,
    Ssh,
    Both,
}

impl ConnectionType {
    /// Get the display name of the connection type.
    pub fn name(&self) -> &'static str {
        match self {
            ConnectionType::Rdp => "RDP",
            ConnectionType::Ssh => "SSH",
            ConnectionType::Both => "Both",
        }
    }

    /// Get all available connection types.
    pub fn all() -> &'static [ConnectionType] {
        &[ConnectionType::Rdp, ConnectionType::Ssh, ConnectionType::Both]
    }
}

/// Manages server connections with automatic cleanup.
pub struct ConnectionManager {
    server: Server,
    settings: Settings,
    vpn_connected: AtomicBool,
    shutdown_flag: Arc<AtomicBool>,
}

impl ConnectionManager {
    /// Create a new connection manager for the given server.
    pub fn new(server: Server, settings: Settings, shutdown_flag: Arc<AtomicBool>) -> Self {
        Self {
            server,
            settings,
            vpn_connected: AtomicBool::new(false),
            shutdown_flag,
        }
    }

    /// Connect to VPN and wait for it to establish.
    pub fn connect_vpn(&self) -> Result<()> {
        if self.shutdown_flag.load(Ordering::SeqCst) {
            anyhow::bail!("Shutdown requested");
        }

        info!("Connecting to VPN: {}", self.server.vpn);
        platform::connect_vpn(&self.server.vpn)?;
        self.vpn_connected.store(true, Ordering::SeqCst);

        // Wait for VPN to establish with polling
        self.wait_for_vpn_connection()?;

        Ok(())
    }

    /// Wait for VPN connection to establish by polling connectivity.
    fn wait_for_vpn_connection(&self) -> Result<()> {
        let timeout = Duration::from_secs(self.settings.vpn_timeout_secs);
        let start = Instant::now();
        let poll_interval = Duration::from_secs(2);

        info!(
            "Waiting for VPN connection (timeout: {}s)...",
            self.settings.vpn_timeout_secs
        );

        while start.elapsed() < timeout {
            if self.shutdown_flag.load(Ordering::SeqCst) {
                anyhow::bail!("Shutdown requested during VPN connection");
            }

            // Try to ping the RDP host to verify connectivity
            debug!("Checking connectivity to {}...", self.server.rdp);
            if platform::ping_host(&self.server.rdp, self.settings.ping_timeout_ms) {
                info!("VPN connection established successfully");
                return Ok(());
            }

            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining > poll_interval {
                std::thread::sleep(poll_interval);
            } else if remaining > Duration::ZERO {
                std::thread::sleep(remaining);
            }
        }

        warn!("VPN connection timeout - proceeding anyway");
        Ok(())
    }

    /// Disconnect from VPN.
    pub fn disconnect_vpn(&self) {
        if self.vpn_connected.load(Ordering::SeqCst) {
            info!("Disconnecting VPN: {}", self.server.vpn);
            if let Err(e) = platform::disconnect_vpn(&self.server.vpn) {
                error!("Failed to disconnect VPN: {}", e);
            }
            self.vpn_connected.store(false, Ordering::SeqCst);
        }
    }

    /// Check if a host is reachable with retries.
    pub fn check_host_reachable(&self, host: &str) -> bool {
        for attempt in 1..=self.settings.ping_retries {
            if self.shutdown_flag.load(Ordering::SeqCst) {
                return false;
            }

            debug!("Ping attempt {} of {} for {}", attempt, self.settings.ping_retries, host);

            if platform::ping_host(host, self.settings.ping_timeout_ms) {
                info!("Host {} is reachable", host);
                return true;
            }

            if attempt < self.settings.ping_retries {
                // Exponential backoff: 1s, 2s, 4s...
                let backoff = Duration::from_secs(1 << (attempt - 1));
                debug!("Waiting {:?} before retry...", backoff);
                std::thread::sleep(backoff);
            }
        }

        warn!("Host {} is not reachable after {} attempts", host, self.settings.ping_retries);
        false
    }

    /// Start an RDP session and return the process handle.
    pub fn start_rdp(&self) -> Result<Option<std::process::Child>> {
        if self.shutdown_flag.load(Ordering::SeqCst) {
            return Ok(None);
        }

        if !self.check_host_reachable(&self.server.rdp) {
            warn!("RDP host {} not reachable, skipping RDP session", self.server.rdp);
            return Ok(None);
        }

        info!("Starting RDP session to {}...", self.server.rdp);
        let child = platform::start_rdp(&self.server.rdp)
            .context("Failed to start RDP session")?;

        Ok(Some(child))
    }

    /// Start an SSH session (blocks until session ends).
    pub fn start_ssh(&self) -> Result<()> {
        if self.shutdown_flag.load(Ordering::SeqCst) {
            return Ok(());
        }

        let ssh_string = self.server.ssh_string()
            .context("SSH not available for this server")?;

        let ssh_ip = self.server.ssh_ip()
            .context("Could not extract IP from SSH string")?;

        if !self.check_host_reachable(&ssh_ip) {
            warn!("SSH host {} not reachable, skipping SSH session", ssh_ip);
            return Ok(());
        }

        info!("Starting SSH session to {}...", ssh_string);
        platform::start_ssh(ssh_string)
            .context("Failed to start SSH session")?;

        Ok(())
    }

    /// Execute the connection based on the selected type.
    pub fn connect(&self, conn_type: ConnectionType) -> Result<()> {
        // Connect to VPN first
        self.connect_vpn()?;

        match conn_type {
            ConnectionType::Rdp => {
                if let Some(mut child) = self.start_rdp()? {
                    info!("Waiting for RDP session to end...");
                    let _ = child.wait();
                }
            }
            ConnectionType::Ssh => {
                self.start_ssh()?;
            }
            ConnectionType::Both => {
                // Start RDP first (non-blocking)
                let rdp_child = self.start_rdp()?;

                // Then start SSH (blocking)
                self.start_ssh()?;

                // Wait for RDP to finish if it was started
                if let Some(mut child) = rdp_child {
                    info!("Waiting for RDP session to end...");
                    let _ = child.wait();
                }
            }
        }

        Ok(())
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        // Ensure VPN is disconnected when the manager is dropped
        self.disconnect_vpn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_type_names() {
        assert_eq!(ConnectionType::Rdp.name(), "RDP");
        assert_eq!(ConnectionType::Ssh.name(), "SSH");
        assert_eq!(ConnectionType::Both.name(), "Both");
    }

    #[test]
    fn test_connection_type_all() {
        let all = ConnectionType::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ConnectionType::Rdp));
        assert!(all.contains(&ConnectionType::Ssh));
        assert!(all.contains(&ConnectionType::Both));
    }
}
