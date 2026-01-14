//! Windows-specific implementations.

use anyhow::{Context, Result};
use std::process::{Child, Command, Stdio};
use tracing::debug;

/// Connect to a VPN using Windows rasphone.
pub fn connect_vpn(vpn_name: &str) -> Result<()> {
    debug!("Executing: rasphone -d {}", vpn_name);

    Command::new("rasphone")
        .args(["-d", vpn_name])
        .spawn()
        .context("Failed to execute rasphone for VPN connection")?;

    Ok(())
}

/// Disconnect from a VPN using Windows rasphone.
pub fn disconnect_vpn(vpn_name: &str) -> Result<()> {
    debug!("Executing: rasphone -h {}", vpn_name);

    Command::new("rasphone")
        .args(["-h", vpn_name])
        .spawn()
        .context("Failed to execute rasphone for VPN disconnection")?;

    Ok(())
}

/// Ping a host using Windows ping command.
pub fn ping_host(host: &str, timeout_ms: u32) -> bool {
    debug!("Executing: ping -n 1 -w {} {}", timeout_ms, host);

    let result = Command::new("ping")
        .args(["-n", "1", "-w", &timeout_ms.to_string(), host])
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

/// Start an RDP session using mstsc.exe.
pub fn start_rdp(address: &str) -> Result<Child> {
    debug!("Executing: mstsc.exe /v:{}", address);

    Command::new("mstsc.exe")
        .arg(format!("/v:{}", address))
        .spawn()
        .context("Failed to start mstsc.exe")
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
    let _ = Command::new("cmd").args(["/c", "cls"]).status();
}
