---
title: Hosts
---

# Hosts

`fw` only runs on hosts it knows. It reads `hostname`, lowercases it and takes the first profile whose name appears anywhere in it. The order is fixed in `detect_host` in `crates/firewall/src/network.rs`.

| Profile | Matches a hostname containing | Extra rules |
|---|---|---|
| `oxidizer` | `oxidizer` | none |
| `radon` | `radon` | Accepts TCP 9010 for the WebRTC nginx upgrade |
| `tungsten` | `tungsten` | none |
| `cobalt` | `cobalt` | none |
| `cryptlok` | `cryptlok` | none |

Any other hostname matches no profile. `fw` prints `fw not setup` and exits with status 1 before it changes any rule.

Because the match is a substring check in a fixed order, a hostname that contains two profile names gets the first one. `radon-cobalt` would be treated as `radon`.

## Adding a host

1. Add a variant to `Host` in `network.rs`.
2. Add its name to `Host::as_str`.
3. Add a `hostname.contains(...)` branch to `detect_host`.
4. If the host needs its own ports, add a check like `Host::is_radon` and a rule in `create_main_firewall` in `firewall.rs`, plus a line in the summary in `diagnostics.rs`.
5. Run `fw --test` on the new host before production mode.
