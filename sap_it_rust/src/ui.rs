//! User interface helpers for terminal interaction.

use crate::config::Server;
use crate::connection::ConnectionType;
use anyhow::{Context, Result};
use colored::*;
use std::io::{self, Write};

/// Display the application header.
pub fn display_header() {
    println!();
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║       SAP-IT Server Connection         ║".cyan());
    println!("{}", "║           Manager v2.0.0               ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();
}

/// Display a menu and get user selection.
pub fn select_from_menu<T, F>(
    title: &str,
    items: &[T],
    display_fn: F,
    max_retries: u32,
) -> Result<usize>
where
    F: Fn(&T) -> String,
{
    for attempt in 1..=max_retries {
        println!("{}", title.cyan());
        println!("{}", "─".repeat(title.len()).cyan());

        for (i, item) in items.iter().enumerate() {
            println!("  {}) {}", i + 1, display_fn(item));
        }

        println!();
        let input = read_input("Enter number")?;

        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= items.len() => {
                return Ok(n - 1);
            }
            _ => {
                if attempt < max_retries {
                    println!(
                        "{}",
                        format!(
                            "Invalid selection. Please enter a number between 1 and {}. ({} attempts remaining)",
                            items.len(),
                            max_retries - attempt
                        ).yellow()
                    );
                    println!();
                } else {
                    anyhow::bail!("Invalid selection after {} attempts", max_retries);
                }
            }
        }
    }

    unreachable!()
}

/// Display server selection menu and return the selected index.
pub fn select_server(servers: &[Server], max_retries: u32) -> Result<usize> {
    select_from_menu(
        "Select a server:",
        servers,
        |server| {
            let ssh_indicator = if server.has_ssh() { " [SSH]" } else { "" };
            format!("{}{}", server.name, ssh_indicator.dimmed())
        },
        max_retries,
    )
}

/// Display connection type selection menu and return the selected type.
pub fn select_connection_type(max_retries: u32) -> Result<ConnectionType> {
    let types = ConnectionType::all();

    let index = select_from_menu(
        "Select connection type:",
        types,
        |conn_type| conn_type.name().to_string(),
        max_retries,
    )?;

    Ok(types[index])
}

/// Read a line of input from the user.
pub fn read_input(prompt: &str) -> Result<String> {
    print!("{}: ", prompt.white().bold());
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    Ok(input)
}

/// Display a status message.
pub fn status(message: &str) {
    println!("{} {}", "→".blue(), message);
}

/// Display a success message.
pub fn success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Display a warning message.
pub fn warning(message: &str) {
    println!("{} {}", "⚠".yellow(), message);
}

/// Display an error message.
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

/// Display connection info before connecting.
pub fn display_connection_info(server: &Server, conn_type: ConnectionType) {
    println!();
    println!("{}", "Connection Details:".cyan().bold());
    println!("  Server: {}", server.name.white().bold());
    println!("  VPN:    {}", server.vpn);
    println!("  Type:   {}", conn_type.name());

    match conn_type {
        ConnectionType::Rdp => {
            println!("  RDP:    {}", server.rdp);
        }
        ConnectionType::Ssh => {
            if let Some(ssh) = server.ssh_string() {
                println!("  SSH:    {}", ssh);
            }
        }
        ConnectionType::Both => {
            println!("  RDP:    {}", server.rdp);
            if let Some(ssh) = server.ssh_string() {
                println!("  SSH:    {}", ssh);
            }
        }
    }

    println!();
}

/// Display a spinner while waiting (simple text-based).
pub fn display_waiting(message: &str) {
    println!("{} {}...", "⏳".yellow(), message);
}

/// Ask for confirmation.
pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N]: ", prompt);
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}
