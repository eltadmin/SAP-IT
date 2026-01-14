//! SAP-IT Server Connection Manager
//!
//! A command-line tool for managing connections to company servers
//! via VPN, RDP, and SSH.

mod config;
mod connection;
mod platform;
mod tui;
mod ui;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use connection::{ConnectionManager, ConnectionType};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::stdout;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info, Level};
use tracing_subscriber::EnvFilter;

/// SAP-IT Server Connection Manager
#[derive(Parser, Debug)]
#[command(name = "sap_it")]
#[command(author = "SAP-IT Team")]
#[command(version = "2.1.0")]
#[command(about = "Server connection manager for IT infrastructure", long_about = None)]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Enable verbose output (can be repeated for more verbosity)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Use simple text mode instead of TUI
    #[arg(long)]
    simple: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate a sample configuration file
    Init {
        /// Output path for the configuration file
        #[arg(short, long, default_value = "servers.toml")]
        output: PathBuf,
    },

    /// List all configured servers
    List,

    /// Connect to a server directly by name or index
    Connect {
        /// Server name or index (1-based)
        server: String,

        /// Connection type: rdp, ssh, or both
        #[arg(short = 't', long, default_value = "rdp")]
        connection_type: String,
    },
}

fn main() {
    if let Err(e) = run() {
        ui::error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on verbosity (only for non-TUI modes)
    if cli.simple || cli.command.is_some() {
        init_logging(cli.verbose);
    }

    debug!("CLI arguments: {:?}", cli);

    // Handle subcommands
    match cli.command {
        Some(Commands::Init { output }) => {
            return init_config(&output);
        }
        Some(Commands::List) => {
            let config = load_config(cli.config.as_ref(), true)?;
            return list_servers(&config);
        }
        Some(Commands::Connect {
            server,
            connection_type,
        }) => {
            let config = load_config(cli.config.as_ref(), true)?;
            return direct_connect(&config, &server, &connection_type);
        }
        None => {
            // Interactive mode
            if cli.simple {
                return simple_interactive_mode(cli.config.as_ref());
            } else {
                return tui_mode(cli.config.as_ref());
            }
        }
    }
}

/// Initialize logging with the specified verbosity level.
fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    let filter = EnvFilter::from_default_env().add_directive(level.into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

/// Load configuration from file or use defaults.
fn load_config(path: Option<&PathBuf>, show_warning: bool) -> Result<Config> {
    let config_path = path.cloned().unwrap_or_else(Config::default_path);

    if config_path.exists() {
        Config::load(&config_path)
    } else {
        if show_warning {
            ui::warning(&format!(
                "Config file not found at '{}', using built-in defaults",
                config_path.display()
            ));
            ui::status("Run 'sap_it init' to create a configuration file");
            println!();
        }
        Ok(Config::default_config())
    }
}

/// Generate a sample configuration file.
fn init_config(output: &PathBuf) -> Result<()> {
    if output.exists() {
        ui::warning(&format!("File '{}' already exists", output.display()));
        if !ui::confirm("Overwrite?")? {
            ui::status("Aborted");
            return Ok(());
        }
    }

    let sample = Config::sample_toml();
    std::fs::write(output, &sample)
        .with_context(|| format!("Failed to write config file: {}", output.display()))?;

    ui::success(&format!("Configuration file created: {}", output.display()));
    println!();
    println!("Edit this file to configure your servers, then run 'sap_it' to connect.");

    Ok(())
}

/// List all configured servers.
fn list_servers(config: &Config) -> Result<()> {
    ui::display_header();
    println!("{}", "Configured Servers:".cyan());
    println!("{}", "─".repeat(40));

    for (i, server) in config.servers.iter().enumerate() {
        let ssh_status = if server.has_ssh() {
            "SSH available".green()
        } else {
            "RDP only".yellow()
        };

        println!();
        println!(
            "  {}. {} ({})",
            i + 1,
            server.name.white().bold(),
            ssh_status
        );
        println!("     VPN: {}", server.vpn);
        println!("     RDP: {}", server.rdp);
        if let Some(ssh) = server.ssh_string() {
            println!("     SSH: {}", ssh);
        }
    }

    println!();
    Ok(())
}

/// Connect directly to a server by name or index.
fn direct_connect(config: &Config, server_ref: &str, conn_type_str: &str) -> Result<()> {
    // Find server by name or index
    let server_index = if let Ok(index) = server_ref.parse::<usize>() {
        if index < 1 || index > config.servers.len() {
            anyhow::bail!(
                "Server index {} out of range (1-{})",
                index,
                config.servers.len()
            );
        }
        index - 1
    } else {
        config
            .servers
            .iter()
            .position(|s| s.name.to_lowercase() == server_ref.to_lowercase())
            .with_context(|| format!("Server '{}' not found", server_ref))?
    };

    // Parse connection type
    let conn_type = match conn_type_str.to_lowercase().as_str() {
        "rdp" => ConnectionType::Rdp,
        "ssh" => ConnectionType::Ssh,
        "both" => ConnectionType::Both,
        _ => anyhow::bail!(
            "Invalid connection type: {}. Use 'rdp', 'ssh', or 'both'",
            conn_type_str
        ),
    };

    let server = &config.servers[server_index];

    // Check if SSH is requested but not available
    if (conn_type == ConnectionType::Ssh || conn_type == ConnectionType::Both) && !server.has_ssh()
    {
        anyhow::bail!("SSH not available for server '{}'", server.name);
    }

    // Set up graceful shutdown
    let shutdown_flag = setup_shutdown_handler();

    ui::display_header();
    ui::display_connection_info(server, conn_type);

    // Create connection manager and connect
    let manager = ConnectionManager::new(server.clone(), config.settings.clone(), shutdown_flag);

    manager.connect(conn_type)?;

    ui::success("Session ended");
    Ok(())
}

/// Run the TUI mode.
fn tui_mode(config_path: Option<&PathBuf>) -> Result<()> {
    let config = load_config(config_path, false)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = tui::App::new(config);

    // Event handler
    let event_handler = tui::EventHandler::new(250); // 250ms tick rate

    // Main loop
    let result = run_tui_loop(&mut terminal, &mut app, &event_handler);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// Run the TUI event loop.
fn run_tui_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut tui::App,
    event_handler: &tui::EventHandler,
) -> Result<()> {
    while !app.should_quit {
        // Render UI
        terminal.draw(|frame| {
            tui::ui::render(app, frame);
        })?;

        // Handle events
        match event_handler.next()? {
            tui::Event::Tick => {
                // Update connection status on tick
                app.update_connection();
            }
            tui::Event::Key(key) => {
                tui::event::handle_key_event(app, key);
            }
            tui::Event::Resize(_, _) => {
                // Terminal resize is handled automatically by ratatui
            }
            tui::Event::Mouse(_) => {
                // Mouse events not used currently
            }
        }
    }

    Ok(())
}

/// Run in simple text interactive mode.
fn simple_interactive_mode(config_path: Option<&PathBuf>) -> Result<()> {
    let config = load_config(config_path, true)?;

    // Set up graceful shutdown
    let shutdown_flag = setup_shutdown_handler();

    // Display header
    platform::clear_screen();
    ui::display_header();

    // Select server
    let server_index = ui::select_server(&config.servers, 3)?;
    let server = &config.servers[server_index];

    // Select connection type
    let conn_type = if server.has_ssh() {
        println!();
        ui::select_connection_type(3)?
    } else {
        ui::status("SSH not available, using RDP");
        ConnectionType::Rdp
    };

    // Display connection info
    ui::display_connection_info(server, conn_type);

    // Create connection manager
    let manager = ConnectionManager::new(server.clone(), config.settings.clone(), shutdown_flag);

    // Connect
    ui::display_waiting("Establishing connection");
    manager.connect(conn_type)?;

    ui::success("Session ended");
    Ok(())
}

/// Set up the Ctrl+C handler for graceful shutdown.
fn setup_shutdown_handler() -> Arc<AtomicBool> {
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let flag_clone = shutdown_flag.clone();

    ctrlc::set_handler(move || {
        info!("Shutdown signal received");
        flag_clone.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    shutdown_flag
}

// Import colored for the list_servers function
use colored::Colorize;
