//! Unified nftables firewall provisioner for all RustyKey(R) VPS instances.
//! PQC-ready, GeoIP-aware. Rust port of fw.sh.
//!
//! Usage: fw [--test] [--geoipblock=<true|false>]

use oxidizer_firewall::{ExitOutcome, RunOptions, current_step, logging, run};
use std::process;

fn parse_bool(value: &str) -> Option<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        eprintln!();
        logging::error(&format!(
            "FIREWALL SCRIPT CRASHED AT STEP: '{}' (panic: {info})",
            current_step()
        ));
        logging::warning(
            "The safety 'at' timer is running. Your rules will auto-flush shortly if locked out.",
        );
    }));

    let args: Vec<String> = std::env::args().collect();
    let program = args.first().cloned().unwrap_or_else(|| "fw".to_string());
    let usage = format!("Usage: {program} [--test] [--geoipblock=<true|false>]");

    let mut is_test_mode = false;
    let mut geoip_enabled = true;

    for arg in args.iter().skip(1) {
        if let Some(value) = arg.strip_prefix("--geoipblock=") {
            match parse_bool(value) {
                Some(v) => geoip_enabled = v,
                None => {
                    eprintln!("❌ Invalid value for --geoipblock: '{value}' (expected true/false)");
                    eprintln!("{usage}");
                    process::exit(1);
                }
            }
        } else {
            match arg.as_str() {
                "--test" | "-t" | "flushinoneminute" => is_test_mode = true, // Backward compatible fallback
                other => {
                    eprintln!("❌ Unknown parameter: '{other}'");
                    eprintln!("{usage}");
                    process::exit(1);
                }
            }
        }
    }

    let outcome = run(RunOptions {
        is_test_mode,
        geoip_enabled,
    });

    match outcome {
        Ok(ExitOutcome::Success) => {}
        Ok(ExitOutcome::CleanExit) => process::exit(1),
        Err(err) => {
            eprintln!();
            logging::error(&format!(
                "FIREWALL SCRIPT CRASHED AT STEP: '{}' ({err:#})",
                current_step()
            ));
            logging::warning(
                "The safety 'at' timer is running. Your rules will auto-flush shortly if locked out.",
            );
            process::exit(1);
        }
    }
}
