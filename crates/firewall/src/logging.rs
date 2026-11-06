//! Adjust prefixes/colors here if they need to match the shared bash
//! library's exact output byte-for-byte.

/// Prints a progress message to stdout.
pub fn honey(msg: &str) {
    println!("\x1b[38;5;178m🍯 {msg}\x1b[0m");
}

/// Prints a success message to stdout.
pub fn success(msg: &str) {
    println!("\x1b[32m✅ {msg}\x1b[0m");
}

/// Prints a warning to stdout.
pub fn warning(msg: &str) {
    println!("\x1b[33m⚠️  {msg}\x1b[0m");
}

/// Prints an error to stderr.
pub fn error(msg: &str) {
    eprintln!("\x1b[31m❌ {msg}\x1b[0m");
}

/// Prints an informational message to stdout.
pub fn info(msg: &str) {
    println!("\x1b[36mℹ️  {msg}\x1b[0m");
}