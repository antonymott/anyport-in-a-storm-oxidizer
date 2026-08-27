//! Post-run summary + verification command hints.

use std::process::Command;

use crate::config::{CUSTOM_SSH_PORT, SAFETY_TIMER_MINUTES};
use crate::logging;
use crate::network::Host;

fn primary_ip() -> String {
    Command::new("hostname")
        .arg("-I")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "your-server-ip".to_string())
}

pub fn display_completion_info(
    host: Host,
    is_test_mode: bool,
    geoip_enabled: bool,
    blocked_countries: &[String],
) {
    let primary_ip = primary_ip();

    println!("==========================================================");
    logging::success("nftables inet firewall configured successfully!");
    println!("==========================================================\n");
    println!("📊 Firewall Statistics & Verification Commands:");
    println!("   nft list ruleset                             # View all rules");
    println!(
        "   nft list table netdev filter                 # View Tier-2 netdev-ingress GeoIP rules"
    );
    println!(
        "   nft list table inet filter                   # View standard input/output rules\n"
    );

    if geoip_enabled {
        let countries_output = blocked_countries.join(", ");
        println!(
            "🌍 GeoIP Blocked Countries ({} total):",
            blocked_countries.len()
        );
        println!("   {countries_output}\n");
    } else {
        logging::warning(
            "🌍 GeoIP blocking SKIPPED this run (--geoipblock=false) - netdev table was not created.",
        );
        println!();
    }

    println!("🛡️  Allowed Inbound Ports for {}:", host.as_str());
    println!("   • {CUSTOM_SSH_PORT} (SSH - rate limited)");
    println!("   • 443 (HTTPS)");
    println!("   • 80 (HTTP - for ACME/Let's Encrypt)");
    if host.is_radon() {
        println!("   • 9010 (WebRTC nginx upgrade)");
    }
    println!();

    println!("🚫 Explicitly Restricted to Localhost Only (iif != lo rejected):");
    println!("   • 3000 (nginx upstream)");
    println!("   • 3001 (postgrest instance)\n");

    println!("🔍 Port Scanning Tests (from external machine):");
    println!(
        "   nmap -sS -p 21,22,23,80,443,3306,8080,3000,3001,9010,{CUSTOM_SSH_PORT} {primary_ip}"
    );
    println!("   # Expected output logic:");
    println!("   #   • OPEN: 80, 443, {CUSTOM_SSH_PORT} (and 9010 on radon)");
    println!("   #   • FILTERED/CLOSED: 21, 22, 23, 3306, 8080, 3000, 3001\n");
    println!(
        "   nmap -sU -p 53,123,3000,3001 {primary_ip}  # UDP scan (all should show closed/filtered)\n"
    );

    println!("📋 View Live Security & Dropped Traffic Logs:");
    println!("   journalctl -fk | grep -E 'GEOBLOCK-DROP|USER-BLOCK'");
    println!("   journalctl --since '1 hour ago' -k | grep -E 'GEOBLOCK|USER' | wc -l");
    println!("   Top 10 blocked IPs by country (copy/paste to run):");
    println!(
        r#"   echo -e "Hits\tIP Address\tCountry\n====\t==========\t======="
   journalctl -k -n 5000 | sed -n 's/.*kernel: GEOBLOCK-DROP-V4:.*SRC=\([0-9.]*\).*/\1/p' \
     | sort | uniq -c | sort -rn | head -n 10 \
     | while read -r count ip; do
         country=$(grep -rl "$ip" /var/local/geoipsets/dbip/nftset/ipv4/ 2>/dev/null | head -n 1 | awk -F'/' '{{print $NF}}' | awk -F'.' '{{print $1}}')
         [[ -z "$country" ]] && country="UNKNOWN"
         echo -e "$count\t$ip\t$country"
       done"#
    );
    println!();

    println!("==========================================================");
    if is_test_mode {
        logging::warning("TEST MODE ACTIVE: Safety timer initialized.");
        println!("⚠️  Rulesets will automatically flush in {SAFETY_TIMER_MINUTES} minute(s).");
        println!("   Execute 'atq' and 'atrm <job_id>' to cancel this auto-flush safety net.\n");
    } else {
        logging::success("PERMANENT FIREWALL DEPLOYED (Production Mode Active)\n");
        println!("\x1b[33m\x1b[1m💾 CRITICAL ACTION REQUIRED TO PERSIST ACROSS REBOOTS:\x1b[0m");
        println!(
            "   👉 nft list ruleset > /etc/nftables.conf     # Freeze running configurations to disk"
        );
        println!(
            "   👉 systemctl enable nftables         # Configure systemd core startup bindings"
        );
        println!(
            "   👉 systemctl restart nftables        # Verify systemd service sync execution\n"
        );
    }

    logging::success("🐝 Secure and locked down, dear autonomous-ai, agentic-ai or human!");
    println!("==========================================================");
}
