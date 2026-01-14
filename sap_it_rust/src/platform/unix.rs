//! Unix/Linux-specific implementations.

use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use tracing::{debug, warn};

/// Connect to a VPN using nmcli (NetworkManager) or openconnect.
pub fn connect_vpn(vpn_name: &str) -> Result<()> {
    // Try NetworkManager first
    debug!("Attempting VPN connection via nmcli: {}", vpn_name);

    let result = Command::new("nmcli")
        .args(["connection", "up", vpn_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => {
            debug!("VPN connected via nmcli");
            return Ok(());
        }
        Ok(_) => {
            warn!("nmcli connection failed, VPN '{}' may not exist", vpn_name);
        }
        Err(e) => {
            debug!("nmcli not available: {}", e);
        }
    }

    // Fallback: try to use vpnc or other tools
    warn!(
        "NetworkManager VPN connection failed. Please ensure VPN '{}' is configured in NetworkManager.",
        vpn_name
    );

    Ok(())
}

/// Disconnect from a VPN using nmcli.
pub fn disconnect_vpn(vpn_name: &str) -> Result<()> {
    debug!("Disconnecting VPN via nmcli: {}", vpn_name);

    let result = Command::new("nmcli")
        .args(["connection", "down", vpn_name])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) if status.success() => {
            debug!("VPN disconnected via nmcli");
        }
        Ok(_) => {
            warn!("nmcli disconnection returned non-zero status");
        }
        Err(e) => {
            debug!("nmcli not available: {}", e);
        }
    }

    Ok(())
}

/// Ping a host using the ping command (Linux syntax).
pub fn ping_host(host: &str, timeout_ms: u32) -> bool {
    // Linux ping uses -W for timeout in seconds (with decimal support on some systems)
    // and -c for count
    let timeout_secs = (timeout_ms as f64 / 1000.0).max(1.0);

    debug!("Executing: ping -c 1 -W {} {}", timeout_secs as u32, host);

    let result = Command::new("ping")
        .args(["-c", "1", "-W", &(timeout_secs as u32).to_string(), host])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match result {
        Ok(status) => status.success(),
        Err(e) => {
            debug!("Ping failed: {}", e);
            false
        }
    }
}

/// Start an RDP session using xfreerdp or rdesktop.
pub fn start_rdp(address: &str) -> Result<Child> {
    // Try xfreerdp first (more modern, better protocol support)
    debug!("Attempting RDP via xfreerdp: {}", address);

    let xfreerdp_result = Command::new("xfreerdp")
        .args([
            &format!("/v:{}", address),
            "/cert:ignore",
            "/dynamic-resolution",
        ])
        .spawn();

    if let Ok(child) = xfreerdp_result {
        return Ok(child);
    }

    // Fallback to xfreerdp3 (newer version with different binary name)
    debug!("xfreerdp not found, trying xfreerdp3...");

    let xfreerdp3_result = Command::new("xfreerdp3")
        .args([
            &format!("/v:{}", address),
            "/cert:ignore",
            "/dynamic-resolution",
        ])
        .spawn();

    if let Ok(child) = xfreerdp3_result {
        return Ok(child);
    }

    // Fallback to rdesktop
    debug!("xfreerdp3 not found, trying rdesktop...");

    Command::new("rdesktop")
        .arg(address)
        .spawn()
        .context("Failed to start RDP client. Please install xfreerdp or rdesktop.")
}

/// Start an SSH session using the ssh command.
pub fn start_ssh(target: &str) -> Result<()> {
    debug!("Executing: ssh {}", target);

    Command::new("ssh")
        .arg(target)
        .status()
        .context("Failed to execute ssh")?;

    Ok(())
}

/// Clear the terminal screen.
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    let _ = std::io::Write::flush(&mut std::io::stdout());
}
