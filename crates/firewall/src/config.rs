//! Static configuration for the firewall provisioner.
//! Mirrors the `readonly` constants at the top of the original fw.sh.

/// The only port SSH is accepted on. Rate-limited to 10 new connections per minute.
pub const CUSTOM_SSH_PORT: u16 = 7909;
/// Temporary file the ingress table is written to before `nft -f` loads it.
pub const GEOIP_CONFIG_PATH: &str = "/tmp/geoip-config.nft";
/// Folder holding the DB-IP country sets, with `ipv4/*.ipv4` and `ipv6/*.ipv6`
/// files that define variables such as `$CN.ipv4`. Not downloaded by `fw`.
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

/// IPv6 addresses or ranges that bypass GeoIP entirely. Same rules as [`TRUSTED_IPV4`].
pub const TRUSTED_IPV6: &[&str] = &[
    // "2001:db8::1",
];