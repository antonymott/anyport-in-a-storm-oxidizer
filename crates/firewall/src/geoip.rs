//! netdev ingress GeoIP blocking + trusted-IP bypass.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::config::{GEOIP_CONFIG_PATH, GEOIP_SETS_PATH, TRUSTED_IPV4, TRUSTED_IPV6};
use crate::logging;

pub fn setup_geoip_blocking(interface: &str, blocked_countries: &[String]) -> Result<()> {
    logging::honey("🎯 Setting up ingress shield (trust list + GeoIP)...");

    let ipv4_dir = format!("{GEOIP_SETS_PATH}/ipv4");
    let ipv6_dir = format!("{GEOIP_SETS_PATH}/ipv6");
    if !Path::new(&ipv4_dir).is_dir() || !Path::new(&ipv6_dir).is_dir() {
        logging::warning(&format!(
            "GeoIP sets not found at {GEOIP_SETS_PATH} - skipping GeoIP blocking"
        ));
        return Ok(());
    }

    let ipv4_elements = blocked_countries
        .iter()
        .map(|c| format!("${c}.ipv4"))
        .collect::<Vec<_>>()
        .join(", ");
    let ipv6_elements = blocked_countries
        .iter()
        .map(|c| format!("${c}.ipv6"))
        .collect::<Vec<_>>()
        .join(", ");

    let trusted_v4_elements = TRUSTED_IPV4.join(", ");
    let trusted_v6_elements = TRUSTED_IPV6.join(", ");

    let trusted_v4_clause = if trusted_v4_elements.is_empty() {
        String::new()
    } else {
        format!("elements = {{ {trusted_v4_elements} }}")
    };
    let trusted_v6_clause = if trusted_v6_elements.is_empty() {
        String::new()
    } else {
        format!("elements = {{ {trusted_v6_elements} }}")
    };

    let config = format!(
        r#"# Advanced GeoIP blocking using highly efficient Sets
include "{GEOIP_SETS_PATH}/ipv4/*.ipv4"
include "{GEOIP_SETS_PATH}/ipv6/*.ipv6"

table netdev filter {{
    chain log_geoblock_drop {{
        counter
        limit rate 5/minute burst 2 packets log prefix "GEOBLOCK-DROP-V4: "
        drop
    }}

    chain log_geoblock_drop_v6 {{
        counter
        limit rate 5/minute burst 2 packets log prefix "GEOBLOCK-DROP-V6: "
        drop
    }}

    set trusted_ipv4_set {{
        type ipv4_addr
        flags interval
        {trusted_v4_clause}
    }}

    set trusted_ipv6_set {{
        type ipv6_addr
        flags interval
        {trusted_v6_clause}
    }}

    set country_ipv4_block_set {{
        type ipv4_addr
        flags interval
        elements = {{ {ipv4_elements} }}
    }}

    set country_ipv6_block_set {{
        type ipv6_addr
        flags interval
        elements = {{ {ipv6_elements} }}
    }}

    chain ingress {{
        type filter hook ingress device "{interface}" priority 0; policy accept;

        ip saddr @trusted_ipv4_set accept
        ip6 saddr @trusted_ipv6_set accept

        ip saddr @country_ipv4_block_set jump log_geoblock_drop
        ip6 saddr @country_ipv6_block_set jump log_geoblock_drop_v6
    }}
}}
"#
    );

    fs::write(GEOIP_CONFIG_PATH, config)
        .with_context(|| format!("failed to write {GEOIP_CONFIG_PATH}"))?;

    logging::honey("Loading ingress rules...");
    let status = Command::new("nft")
        .args(["-f", GEOIP_CONFIG_PATH])
        .status()
        .context("failed to execute `nft -f`")?;

    if status.success() {
        logging::success("ingress shield configured successfully!");
    } else {
        logging::warning("configuration failed - continuing without it...");
    }

    let _ = fs::remove_file(GEOIP_CONFIG_PATH);
    Ok(())
}
