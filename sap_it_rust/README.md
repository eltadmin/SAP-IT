# SAP-IT Server Connection Manager

A cross-platform command-line tool for managing VPN, RDP, and SSH connections to company servers.

## Features

- **Interactive Mode**: Menu-driven server selection
- **Direct Connect**: Connect by server name or index from command line
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

### Interactive Mode

Simply run the program without arguments:

```bash
sap_it
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
  -h, --help           Print help
  -V, --version        Print version
```

### Examples

```bash
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

## License

MIT
