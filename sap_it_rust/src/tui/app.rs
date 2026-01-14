//! Application state for the TUI.

use crate::config::{Config, Server, Settings};
use crate::connection::ConnectionType;
use crate::platform;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Current screen/view in the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Main server selection screen
    ServerList,
    /// Connection type selection
    ConnectionTypeSelect,
    /// Connecting status screen
    Connecting,
    /// Connected/session active screen
    Connected,
    /// Help screen
    Help,
    /// Settings screen
    Settings,
    /// Add/Edit server screen
    EditServer,
    /// Confirmation dialog
    Confirm,
}

/// Connection status during the connection process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Idle,
    ConnectingVpn,
    WaitingForVpn,
    CheckingConnectivity,
    StartingSession,
    Connected,
    Disconnecting,
    Error(String),
}

/// Confirmation dialog type.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteServer(usize),
    Disconnect,
    Quit,
}

/// Application state.
pub struct App {
    /// Current configuration.
    pub config: Config,

    /// Current screen.
    pub screen: Screen,

    /// Previous screen (for going back).
    pub prev_screen: Option<Screen>,

    /// Selected server index.
    pub selected_server: usize,

    /// Selected connection type index.
    pub selected_conn_type: usize,

    /// Current connection status.
    pub connection_status: ConnectionStatus,

    /// Status messages log.
    pub status_log: Vec<(Instant, String)>,

    /// Whether the application should quit.
    pub should_quit: bool,

    /// Shutdown flag for graceful termination.
    pub shutdown_flag: Arc<AtomicBool>,

    /// Currently connected server (if any).
    pub connected_server: Option<usize>,

    /// Connected VPN name (for disconnection).
    pub connected_vpn: Option<String>,

    /// Connection start time.
    pub connection_start: Option<Instant>,

    /// Confirmation dialog action.
    pub confirm_action: Option<ConfirmAction>,

    /// Confirm dialog selection (0 = No, 1 = Yes).
    pub confirm_selection: usize,

    /// Edit server form fields.
    pub edit_server_fields: EditServerFields,

    /// Edit mode (true = edit existing, false = add new).
    pub edit_mode: bool,

    /// Currently editing field index.
    pub edit_field_index: usize,

    /// Input buffer for text editing.
    pub input_buffer: String,

    /// Cursor position in input.
    pub cursor_position: usize,

    /// Settings scroll position.
    pub settings_scroll: usize,

    /// Help scroll position.
    pub help_scroll: usize,
}

/// Fields for editing a server.
#[derive(Debug, Clone, Default)]
pub struct EditServerFields {
    pub name: String,
    pub rdp: String,
    pub ssh: String,
    pub vpn: String,
}

impl App {
    /// Create a new application with the given configuration.
    pub fn new(config: Config) -> Self {
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        Self {
            config,
            screen: Screen::ServerList,
            prev_screen: None,
            selected_server: 0,
            selected_conn_type: 0,
            connection_status: ConnectionStatus::Idle,
            status_log: Vec::new(),
            should_quit: false,
            shutdown_flag,
            connected_server: None,
            connected_vpn: None,
            connection_start: None,
            confirm_action: None,
            confirm_selection: 0,
            edit_server_fields: EditServerFields::default(),
            edit_mode: false,
            edit_field_index: 0,
            input_buffer: String::new(),
            cursor_position: 0,
            settings_scroll: 0,
            help_scroll: 0,
        }
    }

    /// Add a status message to the log.
    pub fn log_status(&mut self, message: impl Into<String>) {
        self.status_log.push((Instant::now(), message.into()));
        // Keep only last 100 messages
        if self.status_log.len() > 100 {
            self.status_log.remove(0);
        }
    }

    /// Get the currently selected server.
    pub fn current_server(&self) -> Option<&Server> {
        self.config.servers.get(self.selected_server)
    }

    /// Get the selected connection type.
    pub fn selected_connection_type(&self) -> ConnectionType {
        match self.selected_conn_type {
            0 => ConnectionType::Rdp,
            1 => ConnectionType::Ssh,
            2 => ConnectionType::Both,
            _ => ConnectionType::Rdp,
        }
    }

    /// Get available connection types for the selected server.
    pub fn available_connection_types(&self) -> Vec<ConnectionType> {
        if let Some(server) = self.current_server() {
            if server.has_ssh() {
                vec![ConnectionType::Rdp, ConnectionType::Ssh, ConnectionType::Both]
            } else {
                vec![ConnectionType::Rdp]
            }
        } else {
            vec![ConnectionType::Rdp]
        }
    }

    /// Navigate to a screen.
    pub fn go_to_screen(&mut self, screen: Screen) {
        self.prev_screen = Some(self.screen);
        self.screen = screen;
    }

    /// Go back to previous screen.
    pub fn go_back(&mut self) {
        if let Some(prev) = self.prev_screen.take() {
            self.screen = prev;
        }
    }

    /// Move selection up in current list.
    pub fn select_previous(&mut self) {
        match self.screen {
            Screen::ServerList => {
                if self.selected_server > 0 {
                    self.selected_server -= 1;
                } else if !self.config.servers.is_empty() {
                    self.selected_server = self.config.servers.len() - 1;
                }
            }
            Screen::ConnectionTypeSelect => {
                let types = self.available_connection_types();
                if self.selected_conn_type > 0 {
                    self.selected_conn_type -= 1;
                } else {
                    self.selected_conn_type = types.len() - 1;
                }
            }
            Screen::Confirm => {
                self.confirm_selection = if self.confirm_selection == 0 { 1 } else { 0 };
            }
            Screen::EditServer => {
                if self.edit_field_index > 0 {
                    self.edit_field_index -= 1;
                    self.load_field_to_input();
                }
            }
            Screen::Help => {
                self.help_scroll = self.help_scroll.saturating_sub(1);
            }
            Screen::Settings => {
                self.settings_scroll = self.settings_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Move selection down in current list.
    pub fn select_next(&mut self) {
        match self.screen {
            Screen::ServerList => {
                if !self.config.servers.is_empty() {
                    self.selected_server = (self.selected_server + 1) % self.config.servers.len();
                }
            }
            Screen::ConnectionTypeSelect => {
                let types = self.available_connection_types();
                self.selected_conn_type = (self.selected_conn_type + 1) % types.len();
            }
            Screen::Confirm => {
                self.confirm_selection = if self.confirm_selection == 0 { 1 } else { 0 };
            }
            Screen::EditServer => {
                if self.edit_field_index < 3 {
                    self.edit_field_index += 1;
                    self.load_field_to_input();
                }
            }
            Screen::Help => {
                self.help_scroll += 1;
            }
            Screen::Settings => {
                self.settings_scroll += 1;
            }
            _ => {}
        }
    }

    /// Handle enter/confirm action.
    pub fn confirm_selection(&mut self) {
        match self.screen {
            Screen::ServerList => {
                if self.current_server().is_some() {
                    // Check if SSH is available
                    if self.current_server().map(|s| s.has_ssh()).unwrap_or(false) {
                        self.selected_conn_type = 0;
                        self.go_to_screen(Screen::ConnectionTypeSelect);
                    } else {
                        // Only RDP available, skip connection type selection
                        self.selected_conn_type = 0;
                        self.start_connection();
                    }
                }
            }
            Screen::ConnectionTypeSelect => {
                self.start_connection();
            }
            Screen::Confirm => {
                if self.confirm_selection == 1 {
                    // Yes selected
                    if let Some(action) = self.confirm_action.take() {
                        match action {
                            ConfirmAction::DeleteServer(index) => {
                                self.config.servers.remove(index);
                                if self.selected_server >= self.config.servers.len()
                                    && self.selected_server > 0
                                {
                                    self.selected_server -= 1;
                                }
                                self.log_status("Server deleted");
                            }
                            ConfirmAction::Disconnect => {
                                self.disconnect();
                            }
                            ConfirmAction::Quit => {
                                self.disconnect();
                                self.should_quit = true;
                            }
                        }
                    }
                }
                self.confirm_action = None;
                self.go_back();
            }
            Screen::EditServer => {
                self.save_current_field();
                if self.edit_field_index < 3 {
                    self.edit_field_index += 1;
                    self.load_field_to_input();
                } else {
                    // Save server
                    self.save_server();
                    self.go_to_screen(Screen::ServerList);
                }
            }
            Screen::Connected => {
                // Show disconnect confirmation
                self.confirm_action = Some(ConfirmAction::Disconnect);
                self.confirm_selection = 0;
                self.go_to_screen(Screen::Confirm);
            }
            _ => {}
        }
    }

    /// Start the connection process.
    fn start_connection(&mut self) {
        if let Some(server) = self.current_server().cloned() {
            self.connection_status = ConnectionStatus::ConnectingVpn;
            self.connection_start = Some(Instant::now());
            self.connected_vpn = Some(server.vpn.clone());
            self.connected_server = Some(self.selected_server);
            self.log_status(format!("Connecting to VPN: {}", server.vpn));
            self.go_to_screen(Screen::Connecting);

            // Start VPN connection
            if let Err(e) = platform::connect_vpn(&server.vpn) {
                self.connection_status = ConnectionStatus::Error(format!("VPN error: {}", e));
                self.log_status(format!("VPN connection failed: {}", e));
            } else {
                self.connection_status = ConnectionStatus::WaitingForVpn;
                self.log_status("Waiting for VPN to establish...");
            }
        }
    }

    /// Disconnect from current session.
    pub fn disconnect(&mut self) {
        if let Some(vpn) = self.connected_vpn.take() {
            self.connection_status = ConnectionStatus::Disconnecting;
            self.log_status(format!("Disconnecting VPN: {}", vpn));
            let _ = platform::disconnect_vpn(&vpn);
            self.log_status("Disconnected");
        }
        self.connected_server = None;
        self.connection_start = None;
        self.connection_status = ConnectionStatus::Idle;
        self.screen = Screen::ServerList;
    }

    /// Update connection status (called periodically).
    pub fn update_connection(&mut self) {
        match &self.connection_status {
            ConnectionStatus::WaitingForVpn => {
                if let Some(server) = self.current_server() {
                    // Check if VPN is connected by pinging the server
                    if platform::ping_host(&server.rdp, self.config.settings.ping_timeout_ms) {
                        self.connection_status = ConnectionStatus::StartingSession;
                        self.log_status("VPN connected, starting session...");
                    } else if let Some(start) = self.connection_start {
                        // Check for timeout
                        if start.elapsed() > Duration::from_secs(self.config.settings.vpn_timeout_secs)
                        {
                            self.connection_status = ConnectionStatus::Error(
                                "VPN connection timeout".to_string(),
                            );
                            self.log_status("VPN connection timed out");
                        }
                    }
                }
            }
            ConnectionStatus::StartingSession => {
                if let Some(server) = self.current_server().cloned() {
                    let conn_type = self.selected_connection_type();

                    match conn_type {
                        ConnectionType::Rdp | ConnectionType::Both => {
                            if let Err(e) = platform::start_rdp(&server.rdp) {
                                self.log_status(format!("RDP error: {}", e));
                            } else {
                                self.log_status(format!("RDP session started to {}", server.rdp));
                            }
                        }
                        _ => {}
                    }

                    if conn_type == ConnectionType::Ssh || conn_type == ConnectionType::Both {
                        if let Some(ssh) = server.ssh_string() {
                            self.log_status(format!("SSH: {}", ssh));
                        }
                    }

                    self.connection_status = ConnectionStatus::Connected;
                    self.screen = Screen::Connected;
                    self.log_status("Session active");
                }
            }
            _ => {}
        }
    }

    /// Request quit with confirmation.
    pub fn request_quit(&mut self) {
        if self.connected_server.is_some() {
            self.confirm_action = Some(ConfirmAction::Quit);
            self.confirm_selection = 0;
            self.go_to_screen(Screen::Confirm);
        } else {
            self.should_quit = true;
        }
    }

    /// Start adding a new server.
    pub fn add_server(&mut self) {
        self.edit_mode = false;
        self.edit_server_fields = EditServerFields::default();
        self.edit_field_index = 0;
        self.load_field_to_input();
        self.go_to_screen(Screen::EditServer);
    }

    /// Start editing selected server.
    pub fn edit_selected_server(&mut self) {
        if let Some(server) = self.current_server() {
            self.edit_mode = true;
            self.edit_server_fields = EditServerFields {
                name: server.name.clone(),
                rdp: server.rdp.clone(),
                ssh: server.ssh.clone().unwrap_or_default(),
                vpn: server.vpn.clone(),
            };
            self.edit_field_index = 0;
            self.load_field_to_input();
            self.go_to_screen(Screen::EditServer);
        }
    }

    /// Delete selected server.
    pub fn delete_selected_server(&mut self) {
        if !self.config.servers.is_empty() {
            self.confirm_action = Some(ConfirmAction::DeleteServer(self.selected_server));
            self.confirm_selection = 0;
            self.go_to_screen(Screen::Confirm);
        }
    }

    /// Load current field to input buffer.
    fn load_field_to_input(&mut self) {
        self.input_buffer = match self.edit_field_index {
            0 => self.edit_server_fields.name.clone(),
            1 => self.edit_server_fields.rdp.clone(),
            2 => self.edit_server_fields.ssh.clone(),
            3 => self.edit_server_fields.vpn.clone(),
            _ => String::new(),
        };
        self.cursor_position = self.input_buffer.len();
    }

    /// Save current input to field.
    fn save_current_field(&mut self) {
        match self.edit_field_index {
            0 => self.edit_server_fields.name = self.input_buffer.clone(),
            1 => self.edit_server_fields.rdp = self.input_buffer.clone(),
            2 => self.edit_server_fields.ssh = self.input_buffer.clone(),
            3 => self.edit_server_fields.vpn = self.input_buffer.clone(),
            _ => {}
        }
    }

    /// Save the server being edited.
    fn save_server(&mut self) {
        self.save_current_field();

        let server = Server {
            name: self.edit_server_fields.name.clone(),
            rdp: self.edit_server_fields.rdp.clone(),
            ssh: if self.edit_server_fields.ssh.is_empty() {
                None
            } else {
                Some(self.edit_server_fields.ssh.clone())
            },
            vpn: self.edit_server_fields.vpn.clone(),
        };

        if self.edit_mode {
            self.config.servers[self.selected_server] = server;
            self.log_status("Server updated");
        } else {
            self.config.servers.push(server);
            self.selected_server = self.config.servers.len() - 1;
            self.log_status("Server added");
        }
    }

    /// Handle character input.
    pub fn handle_char(&mut self, c: char) {
        if self.screen == Screen::EditServer {
            self.input_buffer.insert(self.cursor_position, c);
            self.cursor_position += 1;
        }
    }

    /// Handle backspace.
    pub fn handle_backspace(&mut self) {
        if self.screen == Screen::EditServer && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Handle delete.
    pub fn handle_delete(&mut self) {
        if self.screen == Screen::EditServer && self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Move cursor left.
    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    /// Move cursor right.
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    /// Get connection duration.
    pub fn connection_duration(&self) -> Option<Duration> {
        self.connection_start.map(|start| start.elapsed())
    }

    /// Format duration as string.
    pub fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        let secs = secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }

    /// Save configuration to file.
    pub fn save_config(&self) -> anyhow::Result<()> {
        let config_path = Config::default_path();
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Ensure VPN is disconnected when app exits
        if let Some(vpn) = self.connected_vpn.take() {
            let _ = platform::disconnect_vpn(&vpn);
        }
    }
}
