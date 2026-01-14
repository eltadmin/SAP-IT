use colored::*;
use std::io::{self, Write};
use std::process::{Command, Stdio};

#[derive(Clone)]
struct Server {
    name: &'static str,
    ssh: &'static str,
    rdp: &'static str,
    vpn: &'static str,
}

#[derive(Clone, Copy, PartialEq)]
enum ConnectionType {
    Rdp,
    Ssh,
    Both,
}

impl ConnectionType {
    fn name(&self) -> &'static str {
        match self {
            ConnectionType::Rdp => "RDP",
            ConnectionType::Ssh => "SSH",
            ConnectionType::Both => "Both",
        }
    }
}

const SERVERS: &[Server] = &[
    Server {
        name: "Ilmatex",
        ssh: "root@192.168.0.98",
        rdp: "192.168.0.99",
        vpn: "ILMATEX",
    },
    Server {
        name: "Frodexim",
        ssh: "",
        rdp: "192.168.50.20",
        vpn: "FRODEXIM",
    },
    Server {
        name: "Industrial Technic",
        ssh: "root@192.168.100.10",
        rdp: "192.168.100.20",
        vpn: "Industrial Technik",
    },
    Server {
        name: "BG Nova",
        ssh: "",
        rdp: "192.168.100.20",
        vpn: "Industrial Technik",
    },
];

const CONNECTION_TYPES: &[ConnectionType] = &[
    ConnectionType::Rdp,
    ConnectionType::Ssh,
    ConnectionType::Both,
];

fn main() {
    // Select server
    println!();
    println!("{}", "Select a server:".cyan());
    println!("------------------");

    for (i, server) in SERVERS.iter().enumerate() {
        println!("{}) {}", i + 1, server.name);
    }

    println!();
    let server_choice = read_input("Enter number");

    let server_index = match server_choice.trim().parse::<usize>() {
        Ok(n) if n >= 1 && n <= SERVERS.len() => n - 1,
        _ => {
            eprintln!("{}", "Invalid selection.".red());
            std::process::exit(1);
        }
    };

    let server = &SERVERS[server_index];

    // Select connection type (only if SSH is available)
    let connection_type = if !server.ssh.is_empty() {
        println!();
        println!("{}", "Select connection type:".cyan());
        println!("------------------");

        for (i, conn_type) in CONNECTION_TYPES.iter().enumerate() {
            println!("{}) {}", i + 1, conn_type.name());
        }

        let conn_choice = read_input("Enter Number");

        match conn_choice.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= CONNECTION_TYPES.len() => CONNECTION_TYPES[n - 1],
            _ => {
                eprintln!("{}", "Invalid selection.".red());
                std::process::exit(1);
            }
        }
    } else {
        ConnectionType::Rdp
    };

    println!("{}", connection_type.name());
    println!("{}", server.ssh);

    // Connection logic
    match connection_type {
        ConnectionType::Rdp => {
            connect_vpn(server.vpn);
            if ping_host(server.rdp) {
                let rdp_process = start_rdp(server.rdp);
                if let Some(mut child) = rdp_process {
                    let _ = child.wait();
                }
            }
        }
        ConnectionType::Ssh => {
            connect_vpn(server.vpn);
            if let Some(ip) = extract_ip(server.ssh) {
                if ping_host(&ip) {
                    start_ssh(server.ssh);
                }
            }
        }
        ConnectionType::Both => {
            connect_vpn(server.vpn);
            std::thread::sleep(std::time::Duration::from_secs(15));

            let rdp_child = if ping_host(server.rdp) {
                start_rdp(server.rdp)
            } else {
                None
            };

            if let Some(ip) = extract_ip(server.ssh) {
                if ping_host(&ip) {
                    start_ssh(server.ssh);
                }
            }

            if let Some(mut child) = rdp_child {
                let _ = child.wait();
            }
        }
    }

    // Disconnect VPN
    disconnect_vpn(server.vpn);
}

fn read_input(prompt: &str) -> String {
    print!("{}: ", prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

fn connect_vpn(vpn_name: &str) {
    println!("Connecting to VPN: {}", vpn_name);
    let _ = Command::new("rasphone")
        .args(["-d", vpn_name])
        .spawn();
}

fn disconnect_vpn(vpn_name: &str) {
    println!("Disconnecting VPN: {}", vpn_name);
    let _ = Command::new("rasphone")
        .args(["-h", vpn_name])
        .spawn();
}

fn ping_host(host: &str) -> bool {
    println!("Checking connectivity to {}...", host);
    let output = Command::new("ping")
        .args(["-n", "1", "-w", "3000", host])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match output {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn start_rdp(rdp_address: &str) -> Option<std::process::Child> {
    println!("Starting RDP session to {}...", rdp_address);
    Command::new("mstsc.exe")
        .arg(format!("/v:{}", rdp_address))
        .spawn()
        .ok()
}

fn start_ssh(ssh_target: &str) {
    println!("Starting SSH session to {}...", ssh_target);
    let _ = Command::new("ssh")
        .arg(ssh_target)
        .status();
}

fn extract_ip(ssh_string: &str) -> Option<String> {
    // Extract IP address from SSH string (e.g., "root@192.168.0.98" -> "192.168.0.98")
    let parts: Vec<&str> = ssh_string.split('@').collect();
    if parts.len() == 2 {
        Some(parts[1].to_string())
    } else {
        None
    }
}
