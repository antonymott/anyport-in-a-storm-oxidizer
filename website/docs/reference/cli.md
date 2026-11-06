---
title: Command line
---

# Command line

```text
fw [--test] [--geoipblock=<true|false>]
```

| Option | Default | Effect |
|---|---|---|
| `--test`, `-t`, `flushinoneminute` | off | Test mode. Arms the one-minute safety flush, checks your SSH session and runs the countdown. See [Running fw](../usage/running.md). |
| `--geoipblock=<value>` | `true` | Turns the ingress GeoIP table on or off. Accepts `true`, `false`, `1`, `0`, `yes`, `no`, `on`, `off` in any case. |

Any other argument prints the usage line and exits with status 1.

## Settings in code

These live in `crates/firewall/src/config.rs` and need a rebuild to change.

| Constant | Value | Purpose |
|---|---|---|
| `CUSTOM_SSH_PORT` | `7909` | The only SSH port the firewall opens |
| `SAFETY_TIMER_MINUTES` | `1` | Minutes before the test-mode safety flush |
| `GEOIP_SETS_PATH` | `/var/local/geoipsets/dbip/nftset` | Where the country set files are read from |
| `GEOIP_CONFIG_PATH` | `/tmp/geoip-config.nft` | Temporary file for the ingress table |
| `TRUSTED_IPV4`, `TRUSTED_IPV6` | empty | Addresses that skip the country block |
