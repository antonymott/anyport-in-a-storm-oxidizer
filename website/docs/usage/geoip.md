---
title: GeoIP blocking
---

# GeoIP blocking

The first table `fw` loads is a country-level shield on the network interface. It drops traffic from the chosen countries at the netdev ingress hook, before conntrack, routing or any application.

## Choosing countries

When GeoIP blocking is on, `fw` shows a checklist of the countries in `AVAILABLE_COUNTRIES` (`crates/firewall/src/selector.rs`). Every country starts selected.

| Key | Action |
|---|---|
| Up / Down | Move the cursor |
| Space | Select or clear the country under the cursor |
| Enter | Confirm |

The checklist needs an interactive terminal. To change which countries are offered, edit `AVAILABLE_COUNTRIES` and rebuild.

## What gets loaded

`fw` writes a temporary nftables file to `/tmp/geoip-config.nft`, loads it with `nft -f` and deletes it. It creates a `netdev filter` table with:

- `trusted_ipv4_set` and `trusted_ipv6_set`, filled from `TRUSTED_IPV4` and `TRUSTED_IPV6` in `crates/firewall/src/config.rs`
- `country_ipv4_block_set` and `country_ipv6_block_set`, filled from the GeoIP set files for the chosen countries
- an `ingress` chain on the interface with a default `accept` policy

The ingress chain checks addresses in this order:

1. A source in the trusted sets is accepted.
2. A source in the country sets jumps to a drop chain that counts the packet, logs at most 5 per minute with the prefix `GEOBLOCK-DROP-V4: ` or `GEOBLOCK-DROP-V6: ` and drops it.
3. Everything else is accepted and goes on to the main `inet` firewall.

## Trusted addresses

Addresses in `TRUSTED_IPV4` and `TRUSTED_IPV6` skip the country block only. They still go through every rule in the main firewall, including the port list and rate limits. Both lists are empty by default.

```rust
pub const TRUSTED_IPV4: &[&str] = &[
    "203.0.113.50",
    "198.51.100.0/24",
];
```

## When GeoIP is skipped

- `--geoipblock=false` skips the checklist and the ingress table.
- If `/var/local/geoipsets/dbip/nftset/ipv4` or `ipv6` is missing, `fw` warns and skips the ingress table.
- If `nft -f` rejects the file, `fw` warns `configuration failed - continuing without it...` and goes on to build the main firewall.

In all three cases the main firewall is still applied.
