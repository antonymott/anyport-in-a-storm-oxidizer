//! Library surface for the RustyKey(R) nftables firewall provisioner.
//! `src/main.rs` is a thin binary entrypoint over this crate so the
//! provisioning logic stays unit-testable and reusable.

pub mod config;
pub mod countdown;
pub mod diagnostics;
pub mod firewall;
pub mod geoip;
pub mod logging;
pub mod network;
pub mod selector;
pub mod session;

use anyhow::Result;
use network::Host;
use std::sync::Mutex;

/// Tracks which step is in-flight so a genuine failure can report context,
/// mirroring the bash script's `CURRENT_STEP` + `trap ... ERR` pairing.
static CURRENT_STEP: Mutex<&str> = Mutex::new("Initialization");

fn set_step(step: &'static str) {
    if let Ok(mut guard) = CURRENT_STEP.lock() {
        *guard = step;
    }
}

pub fn current_step() -> &'static str {
    CURRENT_STEP.lock().map(|g| *g).unwrap_or("Unknown")
}

pub enum ExitOutcome {
    /// Provisioning completed - process should exit 0.
    Success,
    /// A known, intentional stop condition (unmapped host, SSH session
    /// lost) - process should exit 1 without the "crashed" framing.
    CleanExit,
}

/// Command-line options controlling a single provisioning run.
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// `test` mode: arms the `at` safety-flush timer and runs the
    /// interactive countdown/confirmation before rules are considered final.
    pub is_test_mode: bool,
    /// Whether netdev GeoIP blocking is applied this run.
    pub geoip_enabled: bool,
}

pub fn run(opts: RunOptions) -> Result<ExitOutcome> {
    let safety_timer = if opts.is_test_mode {
        set_step("Arming Safety Timer");
        Some(countdown::SafetyTimer::arm(config::SAFETY_TIMER_MINUTES)?)
    } else {
        None
    };

    set_step("Host Detection");
    let host = network::detect_host()?;
    if host == Host::Unknown {
        logging::error("fw not setup");
        logging::warning("Host is not mapped to an existing firewall configuration profile.");
        return Ok(ExitOutcome::CleanExit);
    }

    logging::honey("Setting up your nftables firewall...");
    logging::success(&format!("Detected host: {}", host.as_str()));

    set_step("Network Interface Detection");
    let interface = network::detect_interface()?;
    logging::success(&format!("Detected network interface: {interface}"));

    let blocked_countries = if opts.geoip_enabled {
        set_step("Country Selection Menu");
        selector::prompt_country_selection()?
    } else {
        Vec::new()
    };

    set_step("Flushing Existing Rulesets");
    logging::honey("Flushing existing rules...");
    firewall::flush_ruleset()?;

    set_step("Compiling Layer-2 GeoIP Rules");
    if opts.geoip_enabled {
        geoip::setup_geoip_blocking(&interface, &blocked_countries)?;
    } else {
        logging::warning(
            "Skipping GeoIP blocking (--geoipblock=false) - main ruleset only this run.",
        );
    }

    set_step("Assembling Layer-3 Inet Ruleset");
    firewall::create_main_firewall(host)?;

    if opts.is_test_mode {
        set_step("Verifying Live SSH Session");
        if !session::verify_ssh_session_alive()? {
            logging::error(
                "Immediate rollback triggered - flushing ruleset now to restore access.",
            );
            firewall::flush_ruleset()?;
            countdown::SafetyTimer::cancel_all_pending()?;
            return Ok(ExitOutcome::CleanExit);
        }
    }

    set_step("Displaying Final Statistics");
    diagnostics::display_completion_info(
        host,
        opts.is_test_mode,
        opts.geoip_enabled,
        &blocked_countries,
    );

    if opts.is_test_mode {
        set_step("Running Safety Test Countdown Visualizer");
        let timer = safety_timer
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("safety timer was not armed in test mode"))?;
        countdown::run_progress_countdown(opts.is_test_mode, timer)?;
    }

    Ok(ExitOutcome::Success)
}
