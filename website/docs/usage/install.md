---
title: Install
---

# Install

## Build

`fw` is a single binary built with Cargo from the `oxidizer-firewall` crate.

```bash
git clone https://github.com/antonymott/anyport-in-a-storm-oxidizer.git
cd anyport-in-a-storm-oxidizer
cargo build -p oxidizer-firewall --release
```

The binary is written to `target/release/fw`. The crate uses Rust Edition 2024 and sets a minimum Rust version in `crates/firewall/Cargo.toml`, so build it with a current stable toolchain.

## What the host needs

`fw` drives standard Linux tools rather than linking to netfilter, so these must be installed and on the `PATH` of the root user:

| Tool | Used for | Usually provided by |
|---|---|---|
| `nft` | Applying every rule | `nftables` |
| `ip` | Finding the default-route interface | `iproute2` |
| `ss` | Checking your SSH session after the reload (test mode) | `iproute2` |
| `at`, `atq`, `atrm` | The one-minute safety flush (test mode) | `at`, with the `atd` service running |
| `hostname` | Host detection and the summary | `hostname` |
| `stty` | The interactive country checklist and countdown | `coreutils` |

It must run as root because it replaces the whole ruleset.

## GeoIP sets

The ingress shield reads country address sets that must already exist on the host:

```text
/var/local/geoipsets/dbip/nftset/ipv4/*.ipv4
/var/local/geoipsets/dbip/nftset/ipv6/*.ipv6
```

Each file defines nftables variables named after the country code, such as `$CN.ipv4` and `$CN.ipv6`, built from DB-IP data. `fw` does not download or refresh these files. If either folder is missing it skips GeoIP blocking with a warning and still builds the main firewall. See [GeoIP blocking](./geoip.md).

## Host profile

`fw` only runs on hosts it recognises by hostname. On any other host it stops with `fw not setup` and exits with status 1 before touching the ruleset. To run it on a new host, add the host to `crates/firewall/src/network.rs`. See [Hosts](../reference/hosts.md).
