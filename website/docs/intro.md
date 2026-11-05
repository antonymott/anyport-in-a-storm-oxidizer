---
slug: /
sidebar_position: 1
title: Introduction
---

# Anyport in a Storm (Oxidizer)

`oxidizer` (published as **anyport-in-a-storm**) is a small `nftables` firewall provisioner written in Rust (Edition 2024). One run of the `fw` binary flushes the host's ruleset and rebuilds it from code: a country-level ingress shield on the network interface and a default-drop `inet` firewall behind it.

It is not a daemon, a service or an API. It runs once, applies rules through the kernel's own `nft` tool and exits. It is built to be called by an automated defensive agent that watches telemetry and decides when the rules should change, as well as by a human operator.

:::caution[Pre-production]
Wait for v1.0.0 before using this in production or regulated deployments. The tool runs as root and replaces every firewall rule on the host. A mistake can expose the host or lock you out of it. Read [Safety](./safety.md) before the first run.
:::

## What one run does

1. **Arms a safety timer** (test mode only). An `at` job is queued to run `nft flush ruleset` one minute later so a bad ruleset reverts itself.
2. **Detects the host** from its hostname and refuses to continue on a host it does not know.
3. **Detects the network interface** from the default route.
4. **Asks which countries to block** through an interactive checklist, unless GeoIP blocking is turned off.
5. **Flushes the current ruleset.**
6. **Loads the ingress shield**, a `netdev` table bound to the interface that drops traffic from the chosen countries before conntrack and routing.
7. **Builds the main firewall**, an `inet filter` table with a default-drop input policy, a small set of open ports and outbound limits for service users.
8. **Checks your own SSH session survived** (test mode only) and rolls everything back at once if it did not.
9. **Prints a summary** with verification commands and, in test mode, a countdown you stop with a key press to keep the new rules.

## Where it sits

`oxidizer` binds to the nftables netdev **ingress hook**. That is a netfilter feature, not eBPF or XDP. It runs in the same early window as generic XDP, after the kernel allocates the packet buffer but before connection tracking, routing or any application sees the packet. Unlike XDP it needs no driver, hypervisor or NIC support, so it works on ordinary cloud VPS instances. See [The ingress hook](./concepts/ingress-hook.md).

## Quick start

```bash
cargo build --release
sudo ./target/release/fw --test
```

`--test` keeps the one-minute safety timer armed until you confirm the rules. See [Running fw](./usage/running.md) for every mode and [Install](./usage/install.md) for what the host needs first.
