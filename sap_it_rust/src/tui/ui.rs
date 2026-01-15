//! UI rendering for the TUI.

use super::app::{App, ConfirmAction, ConnectionStatus, Screen};
use crate::connection::ConnectionType;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, List, ListItem, Padding, Paragraph, Row, Table, Wrap,
    },
    Frame,
};

/// Render the application UI.
pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Main layout: header, content, footer
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Footer/status
        ])
        .split(area);

    render_header(app, frame, main_layout[0]);
    render_content(app, frame, main_layout[1]);
    render_footer(app, frame, main_layout[2]);

    // Render popup dialogs on top
    match app.screen {
        Screen::Confirm => render_confirm_dialog(app, frame, area),
        Screen::Help => render_help_popup(app, frame, area),
        _ => {}
    }
}

fn render_header(app: &App, frame: &mut Frame, area: Rect) {
    let title = match app.screen {
        Screen::ServerList => " SAP-IT Server Manager ",
        Screen::ConnectionTypeSelect => " Select Connection Type ",
        Screen::Connecting => " Connecting... ",
        Screen::Connected => " Connected ",
        Screen::Help => " Help ",
        Screen::Settings => " Settings ",
        Screen::EditServer => {
            if app.edit_mode {
                " Edit Server "
            } else {
                " Add Server "
            }
        }
        Screen::Confirm => " Confirm ",
    };

    let status_indicator = match &app.connection_status {
        ConnectionStatus::Idle => Span::styled("●", Style::default().fg(Color::Gray)),
        ConnectionStatus::ConnectingVpn | ConnectionStatus::WaitingForVpn => {
            Span::styled("●", Style::default().fg(Color::Yellow))
        }
        ConnectionStatus::CheckingConnectivity | ConnectionStatus::StartingSession => {
            Span::styled("●", Style::default().fg(Color::Cyan))
        }
        ConnectionStatus::Connected => Span::styled("●", Style::default().fg(Color::Green)),
        ConnectionStatus::Disconnecting => Span::styled("●", Style::default().fg(Color::Yellow)),
        ConnectionStatus::Error(_) => Span::styled("●", Style::default().fg(Color::Red)),
    };

    let header_text = Line::from(vec![
        Span::raw(" "),
        status_indicator,
        Span::raw(" "),
        Span::styled(title, Style::default().fg(Color::Cyan).bold()),
        Span::raw(" v2.1.0"),
    ]);

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    frame.render_widget(header, area);
}

fn render_content(app: &App, frame: &mut Frame, area: Rect) {
    match app.screen {
        Screen::ServerList => render_server_list(app, frame, area),
        Screen::ConnectionTypeSelect => render_connection_type(app, frame, area),
        Screen::Connecting => render_connecting(app, frame, area),
        Screen::Connected => render_connected(app, frame, area),
        Screen::Settings => render_settings(app, frame, area),
        Screen::EditServer => render_edit_server(app, frame, area),
        Screen::Help | Screen::Confirm => {
            // These are rendered as popups, show server list behind
            render_server_list(app, frame, area);
        }
    }
}

fn render_server_list(app: &App, frame: &mut Frame, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Server list
    let items: Vec<ListItem> = app
        .config
        .servers
        .iter()
        .enumerate()
        .map(|(i, server)| {
            let ssh_indicator = if server.has_ssh() {
                Span::styled(" [SSH]", Style::default().fg(Color::Green))
            } else {
                Span::styled(" [RDP]", Style::default().fg(Color::Yellow))
            };

            let prefix = format!(" {}. ", i + 1);
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                Span::raw(&server.name),
                ssh_indicator,
            ]);

            if i == app.selected_server {
                ListItem::new(line).style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Servers ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .padding(Padding::horizontal(1)),
        )
        .highlight_style(Style::default().bg(Color::Blue));

    frame.render_widget(list, layout[0]);

    // Server details panel
    render_server_details(app, frame, layout[1]);
}

fn render_server_details(app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Server Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .padding(Padding::uniform(1));

    if let Some(server) = app.current_server() {
        let rows = vec![
            Row::new(vec!["Name:", &server.name]),
            Row::new(vec!["VPN:", &server.vpn]),
            Row::new(vec!["RDP:", &server.rdp]),
            Row::new(vec!["SSH:", server.ssh_string().unwrap_or("Not available")]),
        ];

        let widths = [Constraint::Length(6), Constraint::Min(10)];

        let table = Table::new(rows, widths)
            .block(block)
            .style(Style::default())
            .column_spacing(1);

        frame.render_widget(table, area);
    } else {
        let text = Paragraph::new("No server selected")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(text, area);
    }
}

fn render_connection_type(app: &App, frame: &mut Frame, area: Rect) {
    let types = app.available_connection_types();

    let items: Vec<ListItem> = types
        .iter()
        .enumerate()
        .map(|(i, conn_type)| {
            let (icon, desc) = match conn_type {
                ConnectionType::Rdp => ("🖥️ ", "Remote Desktop Protocol"),
                ConnectionType::Ssh => ("💻", "Secure Shell"),
                ConnectionType::Both => ("🔗", "RDP + SSH"),
            };

            let line = Line::from(vec![
                Span::raw(format!(" {}. ", i + 1)),
                Span::raw(icon),
                Span::raw(" "),
                Span::styled(conn_type.name(), Style::default().bold()),
                Span::styled(format!(" - {}", desc), Style::default().fg(Color::DarkGray)),
            ]);

            if i == app.selected_conn_type {
                ListItem::new(line).style(
                    Style::default()
                        .bg(Color::Blue)
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ListItem::new(line)
            }
        })
        .collect();

    let server_name = app
        .current_server()
        .map(|s| s.name.as_str())
        .unwrap_or("Unknown");

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Connection Type for {} ", server_name))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .padding(Padding::uniform(1)),
    );

    // Center the list
    let centered = centered_rect(50, 50, area);
    frame.render_widget(Clear, centered);
    frame.render_widget(list, centered);
}

fn render_connecting(app: &App, frame: &mut Frame, area: Rect) {
    let centered = centered_rect(60, 50, area);

    let status_text = match &app.connection_status {
        ConnectionStatus::ConnectingVpn => "Initiating VPN connection...",
        ConnectionStatus::WaitingForVpn => "Waiting for VPN to establish...",
        ConnectionStatus::CheckingConnectivity => "Checking connectivity...",
        ConnectionStatus::StartingSession => "Starting session...",
        ConnectionStatus::Error(msg) => msg.as_str(),
        _ => "Connecting...",
    };

    let spinner = get_spinner_frame();

    let elapsed = app
        .connection_duration()
        .map(App::format_duration)
        .unwrap_or_else(|| "00:00".to_string());

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(" {} ", spinner),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(Span::styled(
            status_text,
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Elapsed: {}", elapsed),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press ESC to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let server_name = app
        .current_server()
        .map(|s| s.name.as_str())
        .unwrap_or("Unknown");

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(format!(" Connecting to {} ", server_name))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(Clear, centered);
    frame.render_widget(paragraph, centered);
}

fn render_connected(app: &App, frame: &mut Frame, area: Rect) {
    let centered = centered_rect(60, 60, area);

    let elapsed = app
        .connection_duration()
        .map(App::format_duration)
        .unwrap_or_else(|| "00:00".to_string());

    let server = app.current_server();
    let server_name = server.map(|s| s.name.as_str()).unwrap_or("Unknown");
    let vpn_name = server.map(|s| s.vpn.as_str()).unwrap_or("Unknown");

    let conn_type = app.selected_connection_type();

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "✓ Connected",
            Style::default().fg(Color::Green).bold(),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Server: ", Style::default().fg(Color::DarkGray)),
            Span::styled(server_name, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("VPN: ", Style::default().fg(Color::DarkGray)),
            Span::styled(vpn_name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::DarkGray)),
            Span::styled(conn_type.name(), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Duration: ", Style::default().fg(Color::DarkGray)),
            Span::styled(elapsed, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press D to disconnect, ESC to return",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Session Active ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(Clear, centered);
    frame.render_widget(paragraph, centered);
}

fn render_settings(app: &App, frame: &mut Frame, area: Rect) {
    let settings = &app.config.settings;

    let vpn_timeout_str = format!("{} seconds", settings.vpn_timeout_secs);
    let ping_timeout_str = format!("{} ms", settings.ping_timeout_ms);
    let ping_retries_str = settings.ping_retries.to_string();

    let rows = vec![
        Row::new(vec!["VPN Timeout", vpn_timeout_str.as_str()]),
        Row::new(vec!["Ping Timeout", ping_timeout_str.as_str()]),
        Row::new(vec!["Ping Retries", ping_retries_str.as_str()]),
    ];

    let widths = [Constraint::Length(20), Constraint::Min(10)];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title(" Settings ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .padding(Padding::uniform(1)),
        )
        .column_spacing(2);

    let centered = centered_rect(60, 50, area);
    frame.render_widget(Clear, centered);
    frame.render_widget(table, centered);
}

fn render_edit_server(app: &App, frame: &mut Frame, area: Rect) {
    let centered = centered_rect(70, 60, area);

    let fields = [
        ("Name", &app.edit_server_fields.name, "Server display name"),
        ("RDP Address", &app.edit_server_fields.rdp, "IP or hostname"),
        (
            "SSH (optional)",
            &app.edit_server_fields.ssh,
            "user@host format",
        ),
        (
            "VPN Name",
            &app.edit_server_fields.vpn,
            "As configured in OS",
        ),
    ];

    let mut lines = vec![Line::from("")];

    for (i, (label, value, hint)) in fields.iter().enumerate() {
        let is_selected = i == app.edit_field_index;

        let display_value = if is_selected {
            &app.input_buffer
        } else {
            value
        };

        let label_style = if is_selected {
            Style::default().fg(Color::Cyan).bold()
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let value_style = if is_selected {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![Span::styled(
            format!(" {}: ", label),
            label_style,
        )]));

        let cursor_indicator = if is_selected { "│" } else { "" };
        lines.push(Line::from(vec![
            Span::raw("   "),
            Span::styled(display_value, value_style),
            Span::styled(cursor_indicator, Style::default().fg(Color::Cyan)),
        ]));

        lines.push(Line::from(vec![Span::styled(
            format!("   {}", hint),
            Style::default().fg(Color::DarkGray).italic(),
        )]));

        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        " Tab/↓: Next field | Shift+Tab/↑: Previous | Enter: Save | ESC: Cancel ",
        Style::default().fg(Color::DarkGray),
    )));

    let title = if app.edit_mode {
        " Edit Server "
    } else {
        " Add New Server "
    };

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(Clear, centered);
    frame.render_widget(paragraph, centered);
}

fn render_confirm_dialog(app: &App, frame: &mut Frame, area: Rect) {
    let dialog = centered_rect(50, 30, area);

    let (title, message) = match &app.confirm_action {
        Some(ConfirmAction::DeleteServer(i)) => {
            let name = app
                .config
                .servers
                .get(*i)
                .map(|s| s.name.as_str())
                .unwrap_or("this server");
            (
                " Delete Server ",
                format!("Are you sure you want to delete '{}'?", name),
            )
        }
        Some(ConfirmAction::Disconnect) => (
            " Disconnect ",
            "Are you sure you want to disconnect?".to_string(),
        ),
        Some(ConfirmAction::Quit) => (
            " Quit ",
            "You are connected. Quit and disconnect?".to_string(),
        ),
        None => (" Confirm ", "Confirm action?".to_string()),
    };

    let no_style = if app.confirm_selection == 0 {
        Style::default().bg(Color::Blue).fg(Color::White).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let yes_style = if app.confirm_selection == 1 {
        Style::default().bg(Color::Red).fg(Color::White).bold()
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(&message, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::raw("        "),
            Span::styled(" No ", no_style),
            Span::raw("     "),
            Span::styled(" Yes ", yes_style),
            Span::raw("        "),
        ]),
        Line::from(""),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center);

    frame.render_widget(Clear, dialog);
    frame.render_widget(paragraph, dialog);
}

fn render_help_popup(app: &App, frame: &mut Frame, area: Rect) {
    let help_area = centered_rect(80, 80, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts",
            Style::default().bold().underlined(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from("  ↑/k      Move selection up"),
        Line::from("  ↓/j      Move selection down"),
        Line::from("  Enter    Confirm selection"),
        Line::from("  ESC      Go back / Cancel"),
        Line::from("  1-9      Quick select server by number"),
        Line::from(""),
        Line::from(Span::styled(
            "Server Management",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from("  a        Add new server"),
        Line::from("  e        Edit selected server"),
        Line::from("  d/Del    Delete selected server"),
        Line::from(""),
        Line::from(Span::styled(
            "Quick Connect",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from("  r        Quick RDP connect"),
        Line::from("  S        Quick SSH connect (if available)"),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from("  ?/F1     Show this help"),
        Line::from("  s        Settings"),
        Line::from("  q        Quit"),
        Line::from("  Ctrl+C   Force quit"),
        Line::from(""),
        Line::from(Span::styled(
            "While Connected",
            Style::default().fg(Color::Cyan).bold(),
        )),
        Line::from("  d        Disconnect"),
        Line::from("  ESC      Return to menu"),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to close",
            Style::default().fg(Color::DarkGray).italic(),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" Help ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .padding(Padding::uniform(1)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll as u16, 0));

    frame.render_widget(Clear, help_area);
    frame.render_widget(paragraph, help_area);
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let shortcuts = match app.screen {
        Screen::ServerList => {
            "↑↓:Navigate | Enter:Connect | a:Add | e:Edit | d:Delete | ?:Help | q:Quit"
        }
        Screen::ConnectionTypeSelect => "↑↓:Navigate | Enter:Select | ESC:Back",
        Screen::Connecting => "ESC:Cancel",
        Screen::Connected => "d:Disconnect | ESC:Menu",
        Screen::EditServer => "Tab:Next | Enter:Save | ESC:Cancel",
        Screen::Settings => "S:Save | ESC:Back",
        Screen::Help => "ESC:Close",
        Screen::Confirm => "←→:Select | Enter:Confirm | ESC:Cancel",
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" ", Style::default()),
        Span::styled(shortcuts, Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(footer, area);
}

/// Helper function to create a centered rect.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Get current spinner frame character.
fn get_spinner_frame() -> &'static str {
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        / 100) as usize
        % frames.len();
    frames[idx]
}
