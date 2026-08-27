//! Live-session sanity check (test mode only): confirms the operator's own
//! SSH connection survived the reload. Cannot verify brand-new inbound
//! connections (needs an external vantage point) - fast-fail check only.

use anyhow::{Context, Result};
use std::env;
use std::process::Command;

use crate::logging;

pub fn verify_ssh_session_alive() -> Result<bool> {
    let ssh_connection = match env::var("SSH_CONNECTION") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => {
            logging::warning(
                "No SSH_CONNECTION detected (console/tty session) - skipping live-session check.",
            );
            return Ok(true);
        }
    };

    let mut fields = ssh_connection.split_whitespace();
    let client_ip = fields.next().unwrap_or_default();
    let client_port = fields.next().unwrap_or_default();

    logging::honey(&format!(
        "Verifying current SSH session ({client_ip}:{client_port}) survived the reload..."
    ));

    let filter = format!("( dst {client_ip} and dport = {client_port} )");
    let output = Command::new("ss")
        .args(["-tn", "state", "established", &filter])
        .output()
        .context("failed to execute `ss`")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains(client_ip) {
        logging::success("Live SSH session confirmed intact.");
        return Ok(true);
    }

    logging::error("Current SSH session missing from conntrack after reload - rules likely broke it!");
    Ok(false)
}