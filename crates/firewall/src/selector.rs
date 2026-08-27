//! Interactive CLI checkbox selector for countries using raw terminal mode.

use std::io::{self, Read, Write};
use std::process::Command;

/// Standard list of common ISO country codes as recommended minimum for selection
/// as State-linked Advanced Persistent Threat (APT) origins or comprehensively sanctioned infrastructure.
///
/// Not a perfect filter for human intent, but a necessary and effective shield against automated infrastructure noise
/// ### Defensibility & Authority References:
/// - **CN (China)**:
///   - [CISA China Threat Overview](https://www.cisa.gov/topics/cyber-threats-and-advisories/nation-state-cyber-actors/china)
///   - See Advisory [AA26-113A](https://www.cisa.gov/news-events/cybersecurity-advisories/aa26-113a) regarding China-nexus dynamic covert networks targeting critical infrastructure.
/// - **RU (Russia)**:
///   - [CISA Russia Threat Overview](https://www.cisa.gov/topics/cyber-threats-and-advisories/advanced-persistent-threats/russia)
///   - [OFAC Russia-Related Sanctions](https://ofac.treasury.gov/)
///   - See Advisory [AA26-194a](https://www.cisa.gov/news-events/cybersecurity-advisories/aa26-194a) targeting critical network infrastructure.
/// - **KP (North Korea)**:
///   - [CISA North Korea Threat Overview](https://www.cisa.gov/topics/cyber-threats-and-advisories/advanced-persistent-threats/north-korea)
///   - [OFAC Comprehensive Sanctions](https://ofac.treasury.gov/sanctions-programs-and-country-information/where-is-ofacs-country-list-what-countries-do-i-need-to-worry-about-in-terms-of-us-sanctions)
///   - Reference Joint Advisory [AA24-207A](https://www.cisa.gov/news-events/cybersecurity-advisories/aa24-207a) on state-sponsored global espionage.
/// - **IR (Iran)**:
///   - [CISA Nation-State Threats Framework](https://www.cisa.gov/topics/cyber-threats-and-advisories/nation-state-cyber-actors)
///   - [OFAC Comprehensive Sanctions Program](https://ofac.treasury.gov/)
/// - **CU (Cuba)**:
///   - [OFAC Cuba Sanctions Program](https://treasury.gov) - Maintained under active, comprehensive US trade embargoes.
/// - **BY (Belarus)**:
///   - [OFAC Belarus Sanctions](https://ofac.treasury.gov/sanctions-programs-and-country-information) - Subject to broad hybrid/sectoral restrictions, layered on top of structural industry blocks.
///   - Broadly barred by global tech infrastructure issuers (e.g., [Sectigo Banned Country Policies](https://www.sectigo.com/knowledge-base/detail/Banned-Country-List-1527076085907)) alongside Russia.
pub const AVAILABLE_COUNTRIES: &[(&str, &str)] = &[
    ("CN", "China"),
    ("RU", "Russia"),
    ("KP", "North Korea"),
    ("IR", "Iran"),
    ("CU", "Cuba"),
    ("BY", "Belarus"),
];

struct TermGuard {
    active: bool,
}

impl TermGuard {
    fn new() -> Self {
        let active = Command::new("stty")
            .args(["-icanon", "-echo"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        print!("\x1b[?25l"); // hide cursor
        let _ = io::stdout().flush();
        Self { active }
    }
}

impl Drop for TermGuard {
    fn drop(&mut self) {
        print!("\x1b[?25h"); // show cursor
        let _ = io::stdout().flush();
        if self.active {
            let _ = Command::new("stty").arg("sane").status();
        }
    }
}

/// Prompts the user interactively with an arrow-key / spacebar checkbox menu.
pub fn prompt_country_selection() -> io::Result<Vec<String>> {
    let mut selected = vec![true; AVAILABLE_COUNTRIES.len()]; // Default all selected
    let mut cursor = 0;

    println!("\x1b[1m🌍 Interactive GeoIP Country Selector\x1b[0m");
    println!(
        "Use \x1b[36m[UP/DOWN]\x1b[0m to navigate, \x1b[32m[SPACE]\x1b[0m to toggle, \x1b[33m[ENTER]\x1b[0m to confirm.\n"
    );

    let _guard = TermGuard::new();
    let mut stdin_lock = io::stdin();
    let mut stdout = io::stdout();

    let mut first_render = true;

    let mut render = |cursor: usize, selected: &[bool]| -> io::Result<()> {
        let len = AVAILABLE_COUNTRIES.len();

        if !first_render {
            // Move cursor back up by the number of items to overwrite the previous render
            write!(stdout, "\x1b[{}A", len)?;
        }
        first_render = false;

        for (i, (code, name)) in AVAILABLE_COUNTRIES.iter().enumerate() {
            let checkbox = if selected[i] { "[x]" } else { "[ ]" };
            if i == cursor {
                write!(
                    stdout,
                    "\r\x1b[K\x1b[36;1m > {checkbox} {code} - {name}\x1b[0m\n"
                )?;
            } else {
                write!(stdout, "\r\x1b[K   {checkbox} {code} - {name}\n")?;
            }
        }
        stdout.flush()
    };

    render(cursor, &selected)?;

    let mut buf = [0u8; 1];
    loop {
        if stdin_lock.read_exact(&mut buf).is_err() {
            break;
        }

        let b = buf[0];
        if b == b'\n' || b == b'\r' {
            break;
        } else if b == b' ' {
            selected[cursor] = !selected[cursor];
        } else if b == 27 {
            // Read potential escape sequence for arrow keys
            let mut seq = [0u8; 2];
            if stdin_lock.read_exact(&mut seq).is_ok() && seq[0] == b'[' {
                match seq[1] {
                    b'A' => {
                        // Up arrow
                        cursor = cursor.saturating_sub(1);
                    }
                    b'B'
                        // Down arrow
                        if cursor < AVAILABLE_COUNTRIES.len() - 1 => {
                            cursor += 1;
                        }
                    _ => {}
                }
            }
        }

        render(cursor, &selected)?;
    }

    println!();
    let chosen: Vec<String> = AVAILABLE_COUNTRIES
        .iter()
        .enumerate()
        .filter_map(|(i, (code, _))| {
            if selected[i] {
                Some(code.to_string())
            } else {
                None
            }
        })
        .collect();

    Ok(chosen)
}
