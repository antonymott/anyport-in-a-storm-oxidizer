//! Static configuration for the firewall provisioner.
//! Mirrors the `readonly` constants at the top of the original fw.sh.

pub const CUSTOM_SSH_PORT: u16 = 7909;
pub const GEOIP_CONFIG_PATH: &str = "/tmp/geoip-config.nft";
pub const GEOIP_SETS_PATH: &str = "/var/local/geoipsets/dbip/nftset";

/// Safety-test countdown duration in minutes (used by the `at` daemon).
pub const SAFETY_TIMER_MINUTES: u64 = 1;

/// ISO country codes to block. Easily add/remove here.
pub const BLOCKED_COUNTRIES: &[&str] = &[
    "CN", "RU", "KP", "IR", "VN", "BR", "UA", "TR", "ID", "RO", "MX", "BD", "TH", "PH", "EG",
    "NG", "LY", "NL", "SG", "IN", "BG", "LT", "CO", "PE", "TW", "CR", "ZA",
];

/// Individually trusted addresses that bypass GeoIP entirely (still subject
/// to every normal Tier-1 port/rate rule - this only skips the country block).
pub const TRUSTED_IPV4: &[&str] = &[
    // "203.0.113.50",
    // "198.51.100.0/24",
];

pub const TRUSTED_IPV6: &[&str] = &[
    // "2001:db8::1",
];