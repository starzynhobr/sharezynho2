use std::net::{Ipv4Addr, UdpSocket};
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
const TAILSCALE_PATHS: &[&str] = &["C:/Program Files/Tailscale/tailscale.exe"];
#[cfg(not(target_os = "windows"))]
const TAILSCALE_PATHS: &[&str] = &[
    "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
    "/usr/local/bin/tailscale",
    "/opt/homebrew/bin/tailscale",
];

/// Detects the Tailscale IPv4 address by querying `tailscale ip -4`.
pub fn detect_tailscale_ipv4() -> Option<String> {
    // 1. Try "tailscale" from PATH
    if let Ok(ip) = query_tailscale_binary("tailscale") {
        return Some(ip);
    }

    // 2. Try standard installation paths; apps opened from Finder have a minimal PATH
    TAILSCALE_PATHS
        .iter()
        .filter(|path| Path::new(path).is_file())
        .find_map(|path| query_tailscale_binary(path).ok())
}

fn query_tailscale_binary(cmd: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut command = Command::new(cmd);
    command.args(["ip", "-4"]);
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = command.output()?;

    if !output.status.success() {
        return Err("Tailscale command failed".into());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let token = line.trim();
        if let Ok(ip) = token.parse::<Ipv4Addr>()
            && !ip.is_loopback()
        {
            return Ok(ip.to_string());
        }
    }

    Err("No valid IPv4 found in Tailscale output".into())
}

/// Detects the primary LAN IPv4 address of the local machine.
pub fn detect_lan_ipv4() -> Option<String> {
    let probe = UdpSocket::bind("0.0.0.0:0").ok()?;
    probe.connect("192.0.2.1:9").ok()?;
    if let Ok(std::net::SocketAddr::V4(v4)) = probe.local_addr() {
        let ip = *v4.ip();
        if !ip.is_loopback() {
            return Some(ip.to_string());
        }
    }
    None
}

/// Returns the preferred host address to advertise:
/// 1. Tailscale IPv4 (100.x.y.z)
/// 2. LAN IPv4 (192.168.x.x / 10.x.x.x)
/// 3. Fallback "127.0.0.1"
pub fn preferred_host() -> String {
    detect_tailscale_ipv4()
        .or_else(detect_lan_ipv4)
        .unwrap_or_else(|| "127.0.0.1".to_string())
}
