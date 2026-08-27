//! Safety-net `at` timer + interactive gradient countdown for test mode.

use anyhow::{Context, Result};
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::logging;

pub enum CancelOutcome {
    Cleared,
    FailedStillQueued(String),
    NotFound,
}

pub struct SafetyTimer {
    pub armed_at: u64,
    pub minutes: u64,
    job_id: Option<String>,
}

impl SafetyTimer {
    /// Arms `nft flush ruleset` to fire via `at` in `minutes` minutes.
    /// Prints the same operator warning the original script did.
    pub fn arm(minutes: u64) -> Result<Self> {
        println!("warning: commands will be executed using /bin/sh");

        let mut child = Command::new("at")
            .args(["now", "+", &minutes.to_string(), "minutes"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to spawn `at`")?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(b"nft flush ruleset\n")
                .context("failed to write job to at's stdin")?;
        }

        let output = child
            .wait_with_output()
            .context("failed waiting for `at` to register the job")?;
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let job_id = parse_at_job_id(&combined);

        let armed_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock error")?
            .as_secs();

        Ok(Self { armed_at, minutes, job_id })
    }

    /// Cancels this timer's `at` job (or falls back to "most recent queued
    /// job" if we failed to capture an ID at arm-time, matching the
    /// original bash heuristic), then verifies the cancellation stuck.
    pub fn cancel(&self) -> Result<CancelOutcome> {
        let job_id = match self.job_id.clone() {
            Some(id) => Some(id),
            None => find_last_atq_job()?,
        };

        let Some(job_id) = job_id else {
            return Ok(CancelOutcome::NotFound);
        };

        let _ = Command::new("atrm").arg(&job_id).status();

        if atq_contains(&job_id)? {
            Ok(CancelOutcome::FailedStillQueued(job_id))
        } else {
            Ok(CancelOutcome::Cleared)
        }
    }

    /// Used on emergency rollback: cancel *every* pending `at` job.
    pub fn cancel_all_pending() -> Result<()> {
        let output = Command::new("atq").output().context("failed to execute `atq`")?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for id in stdout.lines().filter_map(|l| l.split_whitespace().next()) {
            let _ = Command::new("atrm").arg(id).status();
        }
        Ok(())
    }
}

fn parse_at_job_id(text: &str) -> Option<String> {
    // Typical `at` output: "job 3 at Thu Jan  1 00:01:00 1970"
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words.next() == Some("job")
            && let Some(id) = words.next() {
                return Some(id.to_string());
        }
    }
    None
}

fn find_last_atq_job() -> Result<Option<String>> {
    let output = Command::new("atq").output().context("failed to execute `atq`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ids: Vec<u64> = stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .filter_map(|s| s.parse().ok())
        .collect();
    ids.sort_unstable();
    Ok(ids.last().map(|id| id.to_string()))
}

fn atq_contains(job_id: &str) -> Result<bool> {
    let output = Command::new("atq").output().context("failed to execute `atq`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter_map(|l| l.split_whitespace().next())
        .any(|id| id == job_id))
}

/// RAII guard: disables canonical mode/echo and hides the cursor, restoring
/// both on drop - covers normal return, early return, *and* panics, which
/// is stronger coverage than bash's `trap ... RETURN INT TERM EXIT`.
struct RawTerminalGuard {
    active: bool,
}

impl RawTerminalGuard {
    fn new() -> Self {
        let active = Command::new("stty")
            .args(["-icanon", "-echo"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !active {
            logging::warning(
                "Could not set raw terminal mode (not a TTY?) - key-press detection may not work.",
            );
        }

        print!("\x1b[?25l"); // hide cursor
        let _ = io::stdout().flush();
        Self { active }
    }
}

impl Drop for RawTerminalGuard {
    fn drop(&mut self) {
        print!("\x1b[?25h"); // show cursor
        let _ = io::stdout().flush();
        if self.active {
            let _ = Command::new("stty").arg("sane").status();
        }
    }
}

fn spawn_key_listener() -> Receiver<()> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = [0u8; 1];
        if io::stdin().read_exact(&mut buf).is_ok() {
            let _ = tx.send(());
        }
    });
    rx
}

fn gradient_color(pos: u64) -> (u8, u8, u8) {
    // Pink/Magenta (255,0,180) -> Orange/Yellow (255,160,0) -> Cyan/Green (0,255,200)
    if pos < 128 {
        let r = 255u64;
        let g = pos * 160 / 128;
        let b = 180u64.saturating_sub(pos * 180 / 128);
        (r as u8, g as u8, b as u8)
    } else {
        let r = 255u64.saturating_sub((pos - 128) * 255 / 127);
        let g = 160 + (pos - 128) * 95 / 127;
        let b = (pos - 128) * 200 / 127;
        (r as u8, g as u8, b as u8)
    }
}

fn render_bar(percentage: u64, remaining: u64) {
    let width: u64 = 40;
    let filled = percentage * width / 100;
    let empty = width - filled;

    let mut bar_filled = String::new();
    for idx in 0..filled {
        let pos = idx * 255 / width;
        let (r, g, b) = gradient_color(pos);
        bar_filled.push_str(&format!("\x1b[38;2;{r};{g};{b}m░\x1b[0m"));
    }

    let bar_empty = if empty > 0 {
        format!("\x1b[38;5;240m{}\x1b[0m", "-".repeat(empty as usize))
    } else {
        String::new()
    };

    print!("\r\x1b[1m[{bar_filled}{bar_empty}] {percentage}% ({remaining}s remaining)\x1b[0m");
    let _ = io::stdout().flush();
}

pub fn run_progress_countdown(is_test_mode: bool, timer: &SafetyTimer) -> Result<()> {
    let total_duration = timer.minutes * 60;
    let start_time = timer.armed_at;

    logging::warning("Entering countdown safety loop. Firewall rules will flush automatically at 100%!");
    println!("👉 Press ANY KEY to accept the rules and cancel the background 'at' flush timer.");
    println!("   (If you lock yourself out, do nothing. Access restores automatically in 60s.)\n");

    let _guard = RawTerminalGuard::new();
    let key_rx = spawn_key_listener();

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock error")?
            .as_secs();
        let elapsed = now.saturating_sub(start_time);
        let percentage = ((elapsed * 100) / total_duration).min(100);
        let remaining = total_duration.saturating_sub(elapsed);

        render_bar(percentage, remaining);

        if elapsed >= total_duration {
            print!("\r\x1b[K");
            println!();
            logging::warning("RULESET AUTOMATICALLY FLUSHED BY SAFETY DAEMON!");
            println!("\x1b[33m\x1b[1m⚠️  WARNING: NO FIREWALL IN PLACE! YOUR VPS IS CURRENTLY EXPOSED! ⚠️\x1b[0m\n");
            return Ok(());
        }

        if key_rx.recv_timeout(Duration::from_millis(200)).is_ok() {
            print!("\r\x1b[K");
            let _ = io::stdout().flush();

            if is_test_mode {
                match timer.cancel()? {
                    CancelOutcome::Cleared => logging::success(
                        "Safety timer cleared! New firewall rules locked in permanently.",
                    ),
                    CancelOutcome::FailedStillQueued(id) => logging::error(&format!(
                        "Failed to cancel safety timer (job {id} still queued)! Check 'atq' manually NOW."
                    )),
                    CancelOutcome::NotFound => logging::warning(
                        "Progress bar dismissed, but could not locate active 'at' job to cancel.",
                    ),
                }
            } else {
                logging::info("Progress bar dismissed. Returning safely to normal prompt context.");
            }
            return Ok(());
        }
    }
}

// bottom of countdown.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_job_id_from_at_output() {
        let text = "warning: commands will be executed using /bin/sh\njob 42 at Thu Jan  1 00:01:00 1970\n";
        assert_eq!(parse_at_job_id(text), Some("42".to_string()));
    }

    #[test]
    fn gradient_hits_expected_endpoints() {
        assert_eq!(gradient_color(0), (255, 0, 180));
        assert_eq!(gradient_color(255), (0, 255, 200));
    }
}