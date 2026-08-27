# Security Policy

## Overview

`anyport-in-a-storm` (internal name: `oxidizer`) is a privileged, root-level `nftables` firewall
provisioner written in Rust, deployed as part of the rustykey.io infrastructure. It operates at
the nftables netdev **ingress hook** (`NF_NETDEV_INGRESS`, e.g. `ingress device "eth0"`) —
**Tier-2 timing** (post-SKB, the same early window as Generic XDP), with **Tier-2 *and* Tier-3
availability** since it needs no eBPF/XDP driver, hypervisor, or NIC cooperation. In practice this
means it manipulates kernel packet filtering *before* routing lookup, conntrack table allocation,
or socket state — but, unlike true Tier-0/Tier-1 XDP, *after* the kernel has already allocated the
SKB. See the main [README](./README.md#where-does-oxidizer-fit) for the full tier breakdown.

Because this tool requires sudo/root privileges and directly programs kernel netfilter hooks, its threat model differs from typical application-layer software. A bug here can cause either:
- total loss of network reachability to production hosts, or
- failure to filter malicious traffic.

`failure to filter` exposes upstream components—including web servers like Nginx and Apache, distributed event streaming platforms like Kafka, high-performance web socket servers like uWebSockets, and DID/WebAuthn endpoints—to PQC-amplified handshake floods and other DoS vectors.

## Supported Versions

| Version | Supported |
| ------- | --------- |
| `main` (latest release build) | ✅ |
| Older tags / commits | ❌ (upgrade before reporting) |

This is an early-stage, actively-developed tool. There are no LTS branches yet. Always run the
latest `--release` build from `main` in production.

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report privately via one of:

- GitHub Security Advisories: use the **"Report a vulnerability"** button under the repo's
  [WIP Security tab](../../security/advisories/new)

Include, where possible:
- The affected commit SHA or release tag
- Target OS/kernel version (this project targets Debian 13, recent `nftables`/`nf_tables` kernel modules)
- Whether the issue is a **filtering bypass** (traffic that should be dropped is not) or a
  **fail-open/fail-closed** risk (host loses connectivity, or firewall silently stops enforcing)
- Reproduction steps or a minimal `nft` ruleset diff demonstrating the bug

We aim to acknowledge reports within **72 hours** and provide a remediation timeline within
**7 days** for confirmed issues.

## Scope

In scope:
- `crates/firewall/src/*` — rule generation, session/countdown logic, GeoIP matching, network
  interface binding, diagnostics/logging
- `deploy.yml` (when it appers) and any deployment automation shipped in this repo
- Privilege escalation, command injection, or path traversal via CLI args (`fw`, `flushinoneminute`,
  `--geoipblock=`, etc.)
- Logic errors that cause the firewall to **fail open** (drop enforcement silently) or **fail closed**
  in a way that bricks remote access without a recovery path
- GeoIP data source integrity/spoofing that could be used to bypass regional blocking
- Race conditions between rule installation, the countdown/flush-in-one-minute safety timer, and
  live traffic

Out of scope:
- The Docusaurus documentation site under `website/docs` (cosmetic/content issues only — open a
  normal issue)
- Denial-of-service caused by the *operator* misconfiguring rules against their own host
- Vulnerabilities in upstream dependencies (`nftables`, the Linux kernel, GeoIP data providers) —
  please report those upstream, but let us know so we can track/patch accordingly

## Operational Security Notes

- **This binary requires root.** Never run untrusted or unreviewed builds with `sudo`. Verify the
  binary was built from a commit you've reviewed (`cargo build --release` from source; no prebuilt
  binaries are distributed).
- **`flushinoneminute` is a safety valve**, not a feature to disable casually — it exists so that a
  bad ruleset push doesn't lock operators out of a remote host permanently. Do not remove or bypass
  this countdown in production without an equivalent out-of-band recovery mechanism (e.g. Hetzner
  console access, IPMI/KVM).
- Always keep a known-good `/etc/nftables.conf` backup (`sudo nft list ruleset > /etc/nftables.conf`)
  **before** applying new rules, and confirm out-of-band console access is available before testing
  ruleset changes on a remote-only box.
- GeoIP-based blocking (`--geoipblock`) is a heuristic, not a security boundary — treat it as
  cost/noise reduction for upstream PQC handshake filtering, not access control.
- This tool is designed to reduce PQC-handshake-amplified DoS load on downstream services
  (MLKEM-TLS 1.3 termination, WebAuthn/DID endpoints, etc.), but it is **not** a substitute for
  rate limiting, WAF rules, or application-layer validation at the uWebSockets.js/nginx layer.
- Because filtering happens at Tier-2 timing rather than Tier-0/1, `oxidizer` cannot drop traffic
  before it costs the kernel a packet buffer allocation — it reduces exposure to downstream,
  memory-intensive PQC validation, not NIC-level ingress load itself. Do not represent this tool
  as line-rate/hardware-speed mitigation in any dependent risk assessment.

## Hardening Expectations for Contributors

- Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt` before submitting PRs.
- Any change touching `firewall.rs`, `network.rs`, or `selector.rs` should include unit tests
  (`cargo test -p oxidizer-firewall`) demonstrating correct rule construction — malformed nft
  rulesets are a security issue, not just a bug.
- Avoid introducing dynamic shell invocation of `nft` where a typed/structured API is available;
  prefer explicit, reviewable rule construction over string concatenation to avoid injection via
  GeoIP/telemetry-derived input.
- Treat all externally-sourced data (GeoIP databases, telemetry feeds consumed by future
  agentic/AI orchestration) as untrusted input requiring validation before it influences firewall
  state.

## Disclosure Policy

We follow a coordinated disclosure model. Once a fix is available, we will credit reporters
(unless anonymity is requested) in the release notes. Please allow us to ship a patched release before any public write-up.