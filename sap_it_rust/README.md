# SAP-IT Server Connection Manager

A cross-platform command-line tool with an interactive TUI for managing VPN, RDP, and SSH connections to company servers.

## Features

- **Interactive TUI**: Full-featured terminal user interface with keyboard navigation
- **Server Management**: Add, edit, and delete servers directly from the UI
- **Quick Connect**: Connect to servers with a single keypress
- **VPN Management**: Automatic VPN connect/disconnect with graceful shutdown
- **Cross-Platform**: Works on Windows (rasphone/mstsc) and Linux (nmcli/xfreerdp)
- **External Configuration**: Servers defined in TOML config file
- **Graceful Shutdown**: Ctrl+C properly disconnects VPN before exit
- **Retry Logic**: Configurable ping retries with exponential backoff

## Installation

### Pre-built Binaries

Download the latest release from the [Releases](../../releases) page.

### Build from Source

```bash
cd sap_it_rust
cargo build --release
```

The binary will be at `target/release/sap_it` (or `sap_it.exe` on Windows).

## Usage

### TUI Mode (Default)

Simply run the program without arguments to launch the interactive TUI:

```bash
sap_it
```

#### TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `↑`/`k` | Move selection up |
| `↓`/`j` | Move selection down |
| `Enter` | Confirm selection / Connect |
| `ESC` | Go back / Cancel |
| `1-9` | Quick select server by number |
| `a` | Add new server |
| `e` | Edit selected server |
| `d`/`Del` | Delete selected server |
| `r` | Quick RDP connect |
| `S` | Quick SSH connect |
| `?`/`F1` | Show help |
| `s` | Settings |
| `q` | Quit |
| `Ctrl+C` | Force quit |

### Simple Text Mode

Use `--simple` flag for basic text-based interaction:

```bash
sap_it --simple
```

### Command-Line Options

```
Usage: sap_it [OPTIONS] [COMMAND]

Commands:
  init     Generate a sample configuration file
  list     List all configured servers
  connect  Connect to a server directly by name or index
  help     Print help for commands

Options:
  -c, --config <FILE>  Path to the configuration file
  -v, --verbose...     Enable verbose output (repeat for more)
      --simple         Use simple text mode instead of TUI
  -h, --help           Print help
  -V, --version        Print version
```

### Examples

```bash
# Launch interactive TUI (default)
sap_it

# Use simple text mode
sap_it --simple

# Generate a config file
sap_it init

# Generate config at specific location
sap_it init --output /path/to/servers.toml

# List all servers
sap_it list

# Connect to server by name (RDP)
sap_it connect Ilmatex

# Connect with SSH
sap_it connect Ilmatex --connection-type ssh

# Connect with both RDP and SSH
sap_it connect "Industrial Technic" -t both

# Connect by index
sap_it connect 1

# Use custom config file
sap_it --config /path/to/servers.toml list

# Verbose mode for debugging
sap_it -vv connect Ilmatex
```

## TUI Interface

### Server List View
```
╭────────────────────────────────────────╮
│   ● SAP-IT Server Connection v2.1.0   │
╰────────────────────────────────────────╯
╭─ Servers ─────────────────╮╭─ Server Details ──────────╮
│  1. Ilmatex [SSH]         ││ Name: Ilmatex             │
│▶ 2. Frodexim [RDP]        ││ VPN:  FRODEXIM            │
│  3. Industrial Tech [SSH] ││ RDP:  192.168.50.20       │
│  4. BG Nova [RDP]         ││ SSH:  Not available       │
╰───────────────────────────╯╰───────────────────────────╯
╭────────────────────────────────────────╮
│ ↑↓:Navigate | Enter:Connect | ?:Help  │
╰────────────────────────────────────────╯
```

### Connection Type Selection
```
╭─ Connection Type for Ilmatex ─────────╮
│                                       │
│  1. 🖥️  RDP - Remote Desktop Protocol │
│  2. 💻 SSH - Secure Shell             │
│  3. 🔗 Both - RDP + SSH               │
│                                       │
╰───────────────────────────────────────╯
```

### Connected Status
```
╭─ Session Active ──────────────────────╮
│                                       │
│            ✓ Connected                │
│                                       │
│  Server:   Ilmatex                    │
│  VPN:      ILMATEX                    │
│  Type:     RDP                        │
│  Duration: 05:23                      │
│                                       │
│  Press D to disconnect, ESC to return │
╰───────────────────────────────────────╯
```

### Add/Edit Server
```
╭─ Add New Server ──────────────────────╮
│                                       │
│  Name:                                │
│    My New Server                      │
│    Server display name                │
│                                       │
│  RDP Address:                         │
│    192.168.1.100│                     │
│    IP or hostname                     │
│                                       │
│  SSH (optional):                      │
│    admin@192.168.1.100                │
│    user@host format                   │
│                                       │
│  VPN Name:                            │
│    MY_VPN                             │
│    As configured in OS                │
│                                       │
│ Tab:Next | Enter:Save | ESC:Cancel    │
╰───────────────────────────────────────╯
```

## Configuration

The program looks for `servers.toml` in:
1. Path specified with `--config`
2. `~/.config/sap_it/servers.toml` (Linux) or `%APPDATA%\sap_it\servers.toml` (Windows)
3. Current directory

### Example Configuration

```toml
[settings]
vpn_timeout_secs = 30
ping_timeout_ms = 3000
ping_retries = 3

[[servers]]
name = "My Server"
ssh = "admin@192.168.1.100"  # Optional
rdp = "192.168.1.100"
vpn = "MY_VPN_NAME"

[[servers]]
name = "RDP Only Server"
rdp = "192.168.2.50"
vpn = "OTHER_VPN"
```

## Platform Requirements

### Windows
- VPN connections configured in Windows (rasphone)
- mstsc.exe for RDP
- ssh.exe for SSH (Windows 10+ or OpenSSH installed)

### Linux
- VPN connections configured in NetworkManager
- xfreerdp, xfreerdp3, or rdesktop for RDP
- OpenSSH client for SSH

## Project Structure

```
sap_it_rust/
├── Cargo.toml
├── README.md
├── servers.example.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── config.rs            # TOML configuration
│   ├── connection.rs        # Connection manager
│   ├── ui.rs                # Simple text UI helpers
│   ├── tui/
│   │   ├── mod.rs           # TUI module
│   │   ├── app.rs           # Application state
│   │   ├── event.rs         # Event handling
│   │   └── ui.rs            # TUI rendering
│   └── platform/
│       ├── mod.rs           # Platform abstraction
│       ├── windows.rs       # Windows implementation
│       └── unix.rs          # Linux implementation
└── tests/
    └── integration_tests.rs
```

## License

MIT
