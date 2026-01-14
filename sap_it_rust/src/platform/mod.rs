//! Platform-specific implementations for VPN, RDP, and SSH operations.

#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unix;

use anyhow::Result;
use std::process::Child;

/// Connect to a VPN by name.
#[cfg(windows)]
pub fn connect_vpn(vpn_name: &str) -> Result<()> {
    windows::connect_vpn(vpn_name)
}

#[cfg(not(windows))]
pub fn connect_vpn(vpn_name: &str) -> Result<()> {
    unix::connect_vpn(vpn_name)
}

/// Disconnect from a VPN by name.
#[cfg(windows)]
pub fn disconnect_vpn(vpn_name: &str) -> Result<()> {
    windows::disconnect_vpn(vpn_name)
}

#[cfg(not(windows))]
pub fn disconnect_vpn(vpn_name: &str) -> Result<()> {
    unix::disconnect_vpn(vpn_name)
}

/// Ping a host to check connectivity.
#[cfg(windows)]
pub fn ping_host(host: &str, timeout_ms: u32) -> bool {
    windows::ping_host(host, timeout_ms)
}

#[cfg(not(windows))]
pub fn ping_host(host: &str, timeout_ms: u32) -> bool {
    unix::ping_host(host, timeout_ms)
}

/// Start an RDP session to the specified address.
#[cfg(windows)]
pub fn start_rdp(address: &str) -> Result<Child> {
    windows::start_rdp(address)
}

#[cfg(not(windows))]
pub fn start_rdp(address: &str) -> Result<Child> {
    unix::start_rdp(address)
}

/// Start an SSH session to the specified target.
#[cfg(windows)]
pub fn start_ssh(target: &str) -> Result<()> {
    windows::start_ssh(target)
}

#[cfg(not(windows))]
pub fn start_ssh(target: &str) -> Result<()> {
    unix::start_ssh(target)
}

/// Clear the terminal screen.
#[cfg(windows)]
pub fn clear_screen() {
    windows::clear_screen()
}

#[cfg(not(windows))]
pub fn clear_screen() {
    unix::clear_screen()
}
