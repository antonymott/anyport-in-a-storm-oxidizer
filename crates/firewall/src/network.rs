//! Interface + host auto-detection.

use anyhow::{Context, Result, bail};
use std::process::Command;

use crate::logging;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    Oxidizer,
    Radon,
    Tungsten,
    Cobalt,
    Cryptlok,
    Unknown,
}

impl Host {
    pub fn as_str(self) -> &'static str {
        match self {
            Host::Oxidizer => "oxidizer",
            Host::Radon => "radon",
            Host::Tungsten => "tungsten",
            Host::Cobalt => "cobalt",
            Host::Cryptlok => "cryptlok",
            Host::Unknown => "unknown",
        }
    }

    pub fn is_radon(self) -> bool {
        matches!(self, Host::Radon)
    }
}

pub fn detect_host() -> Result<Host> {
    let output = Command::new("hostname")
        .output()
        .context("failed to execute `hostname`")?;
    let hostname = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_lowercase();

    Ok(if hostname.contains("oxidizer") {
        Host::Oxidizer
    } else if hostname.contains("radon") {
        Host::Radon
    } else if hostname.contains("tungsten") {
        Host::Tungsten
    } else if hostname.contains("cobalt") {
        Host::Cobalt
    } else if hostname.contains("cryptlok") {
        Host::Cryptlok
    } else {
        Host::Unknown
    })
}

pub fn detect_interface() -> Result<String> {
    let output = Command::new("ip")
        .args(["route", "show", "default"])
        .output()
        .context("failed to execute `ip route show default`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Some(iface) = parse_default_route_interface(&stdout) {
        return Ok(iface);
    }

    logging::warning("Could not auto-detect interface, trying common names...");

    for candidate in ["eth0", "ens3", "ens18", "enp0s3", "enp1s0"] {
        let ok = Command::new("ip")
            .args(["link", "show", candidate])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ok {
            return Ok(candidate.to_string());
        }
    }

    logging::error("Failed to detect network interface!");
    logging::honey("Available interfaces:");
    if let Ok(out) = Command::new("ip").args(["link", "show"]).output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let trimmed = line.trim_start();
            let starts_numeric = trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false);
            if starts_numeric && let Some(name) = trimmed.split_whitespace().nth(1) {
                println!("{}", name.trim_end_matches(':'));
            }
        }
    }

    bail!("network interface detection failed");
}

fn parse_default_route_interface(route_output: &str) -> Option<String> {
    for line in route_output.lines() {
        if !line.starts_with("default") {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let pos = tokens.iter().position(|&t| t == "dev")?;
        return tokens.get(pos + 1).map(|s| s.to_string());
    }
    None
}

// bottom of network.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dev_interface_from_default_route() {
        let route = "default via 10.0.0.1 dev eth0 proto dhcp metric 100";
        assert_eq!(
            parse_default_route_interface(route),
            Some("eth0".to_string())
        );
    }

    #[test]
    fn returns_none_without_a_default_route() {
        assert_eq!(
            parse_default_route_interface("10.0.0.0/24 dev eth0 scope link"),
            None
        );
    }
}
