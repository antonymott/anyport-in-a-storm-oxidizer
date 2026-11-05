//! Layer-3 `inet filter` table: INPUT / FORWARD / OUTPUT rules.

use anyhow::{bail, Context, Result};
use std::process::Command;

use crate::config::CUSTOM_SSH_PORT;
use crate::logging;
use crate::network::Host;

/// Runs `nft <rule>`, passing the whole rule as a single argument.
/// nft joins argv with spaces internally before parsing, so this is
/// equivalent to how the original bash script invoked it.
fn nft(rule: &str) -> Result<()> {
    let status = Command::new("nft")
        .arg(rule)
        .status()
        .with_context(|| format!("failed to execute: nft {rule}"))?;
    if !status.success() {
        bail!("nft command failed: nft {rule}");
    }
    Ok(())
}

/// Removes every table, chain and rule from the host, leaving no firewall.
///
/// # Errors
///
/// Returns an error if `nft` cannot be run or rejects the command.
pub fn flush_ruleset() -> Result<()> {
    nft("flush ruleset")
}

/// Builds the `inet filter` table for `host`.
///
/// Creates `input` and `forward` chains with a drop policy and an `output`
/// chain with an accept policy. Input allows loopback, established traffic,
/// SSH on [`CUSTOM_SSH_PORT`] at 10 new connections per minute, HTTP, HTTPS,
/// rate-limited ICMP echo and ICMPv6 discovery, plus TCP 9010 on radon. It drops
/// invalid packets, failed reverse-path checks and illegal TCP flag
/// combinations, and rejects TCP and UDP 3000 and 3001 from anything but
/// loopback. Output lets `app-runner` reach only Redis on loopback, blocks
/// every other user from Redis on loopback and blocks all new connections
/// from `git-deploy`.
///
/// Expects the ruleset to have been flushed first.
///
/// # Errors
///
/// Returns an error on the first `nft` command that fails. Rules added before
/// it stay in place.
pub fn create_main_firewall(host: Host) -> Result<()> {
    logging::honey("Creating inet filter table...");
    nft("add table inet filter")?;

    logging::honey("Creating chains with policies...");
    nft("add chain inet filter input { type filter hook input priority 0; policy drop; }")?;
    nft("add chain inet filter forward { type filter hook forward priority 0; policy drop; }")?;
    nft("add chain inet filter output { type filter hook output priority 0; policy accept; }")?;

    // =========================================================================
    // INPUT (default policy: DROP)
    // =========================================================================
    logging::honey("Adding INPUT rules...");

    // 1. Immediate safety nets
    nft("add rule inet filter input iif lo accept")?;
    nft("add rule inet filter input ct state established,related accept")?;
    nft("add rule inet filter input ct state invalid drop")?;

    // 2.A kernel-level basic reverse-path check.
    // Works well for strict deterministic unicast routing (our setup), but
    // may break multi-homed HA nodes, asymmetric routing, or WebRTC tunnels.
    nft("add rule inet filter input fib saddr . iif oif missing drop")?;

    // 2.B TCP flag cleansing: drop malformed / illegal stealth-scan flag states.
    nft("add rule inet filter input tcp flags & (fin | syn | rst | psh | ack | urg) == 0 drop")?;
    nft("add rule inet filter input tcp flags & (fin | syn) == fin | syn drop")?;
    nft("add rule inet filter input tcp flags & (rst | ack) == rst | ack drop")?;

    // 3. Gentle early rejection for sensitive upstreams
    nft("add rule inet filter input iif != lo tcp dport { 3000, 3001 } reject with icmpx type port-unreachable")?;
    nft("add rule inet filter input iif != lo udp dport { 3000, 3001 } reject with icmpx type port-unreachable")?;

    // 4. Global structural allowed services
    nft(&format!(
        "add rule inet filter input tcp dport {CUSTOM_SSH_PORT} limit rate 10/minute accept"
    ))?;
    nft("add rule inet filter input tcp dport { 80, 443 } accept")?;

    // 5. Host-specific operational overrides
    if host.is_radon() {
        logging::warning("Adding radon-specific input rules...");
        nft("add rule inet filter input tcp dport 9010 accept")?;
    }

    // 6. Core infrastructure health checks
    nft("add rule inet filter input icmp type echo-request limit rate 5/second accept")?;
    nft("add rule inet filter input icmpv6 type { echo-request, nd-neighbor-solicit, nd-neighbor-advert, nd-router-solicit, nd-router-advert } accept")?;

    // =========================================================================
    // OUTPUT (default policy: ACCEPT)
    // =========================================================================
    logging::honey("Adding OUTPUT rules...");

    // 1. Micro-targeted loopback whitelist: only app-runner -> Redis
    nft(r#"add rule inet filter output oif lo skuid "app-runner" tcp dport 6379 ct state new accept"#)?;

    // 2. Loopback defense-gap fix: hard drop any other user reaching Redis over loopback
    nft(r#"add rule inet filter output oif lo tcp dport 6379 log prefix "REDIS-LOOPBACK-BLOCK: " drop"#)?;

    logging::warning("Applying outbound user-level constraint blocks...");

    // 3.A App-runner cage
    nft(r#"add rule inet filter output skuid "app-runner" ct state new limit rate 5/minute burst 2 packets log prefix "USER-BLOCK-APP-RUNNER: ""#)?;
    nft(r#"add rule inet filter output skuid "app-runner" ct state new reject"#)?;

    // 3.B Git-deploy cage (fully blocked from internet AND localhost loops)
    nft(r#"add rule inet filter output skuid "git-deploy" ct state new limit rate 5/minute burst 2 packets log prefix "USER-BLOCK-GIT-DEPLOY: ""#)?;
    nft(r#"add rule inet filter output skuid "git-deploy" ct state new reject"#)?;

    // 4. Broad structural loopback protection (root, systemd, sshd, etc.)
    nft("add rule inet filter output oif lo accept")?;

    Ok(())
}