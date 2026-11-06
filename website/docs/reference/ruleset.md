---
title: Ruleset
---

# Ruleset

A run creates up to two tables. The `netdev filter` table is described in [GeoIP blocking](../usage/geoip.md). This page covers the `inet filter` table, which handles IPv4 and IPv6 together. It is built in `crates/firewall/src/firewall.rs`.

## Chains

| Chain | Hook | Default policy |
|---|---|---|
| `input` | input | **drop** |
| `forward` | forward | **drop** |
| `output` | output | accept |

`forward` has no rules, so the host forwards nothing.

## Input

Rules are checked in this order. The first match wins.

| # | Rule | Effect |
|---|---|---|
| 1 | `iif lo` | Accept loopback |
| 2 | `ct state established,related` | Accept replies and related traffic |
| 3 | `ct state invalid` | Drop |
| 4 | `fib saddr . iif oif missing` | Drop packets whose source would not route back out the same interface (strict reverse-path check) |
| 5 | TCP flags all clear | Drop (null scan) |
| 6 | TCP `syn` and `fin` both set | Drop |
| 7 | TCP `rst` and `ack` both set | Drop |
| 8 | TCP or UDP to 3000 or 3001 from anything but loopback | Reject with port unreachable |
| 9 | TCP to 7909 (SSH) | Accept, limited to 10 new connections per minute in total |
| 10 | TCP to 80 and 443 | Accept |
| 11 | TCP to 9010 | Accept, **radon only** |
| 12 | ICMP echo request | Accept, limited to 5 per second |
| 13 | ICMPv6 echo request, neighbour discovery and router discovery | Accept |

Anything else hits the drop policy.

The reverse-path check suits hosts with simple single-path routing. It can break hosts with several uplinks, asymmetric routing or some WebRTC tunnels.

The SSH and ICMP limits are shared by all sources. They are not per address.

## Output

| # | Rule | Effect |
|---|---|---|
| 1 | Loopback, user `app-runner`, TCP 6379, new connection | Accept (the app can reach Redis) |
| 2 | Loopback, TCP 6379 | Log `REDIS-LOOPBACK-BLOCK: ` and drop (no other user can reach Redis) |
| 3 | User `app-runner`, new connection | Log `USER-BLOCK-APP-RUNNER: ` at most 5 per minute |
| 4 | User `app-runner`, new connection | Reject |
| 5 | User `git-deploy`, new connection | Log `USER-BLOCK-GIT-DEPLOY: ` at most 5 per minute |
| 6 | User `git-deploy`, new connection | Reject |
| 7 | `oif lo` | Accept loopback for every other user |

Everything else uses the accept policy.

Taken together, `app-runner` can open new connections only to Redis on loopback, and `git-deploy` cannot open any new connections. Root and system users keep normal outbound access.
