//! Adjust prefixes/colors here if they need to match the shared bash
//! library's exact output byte-for-byte.

pub fn honey(msg: &str) {
    println!("\x1b[38;5;178m🍯 {msg}\x1b[0m");
}

pub fn success(msg: &str) {
    println!("\x1b[32m✅ {msg}\x1b[0m");
}

pub fn warning(msg: &str) {
    println!("\x1b[33m⚠️  {msg}\x1b[0m");
}

pub fn error(msg: &str) {
    eprintln!("\x1b[31m❌ {msg}\x1b[0m");
}

pub fn info(msg: &str) {
    println!("\x1b[36mℹ️  {msg}\x1b[0m");
}