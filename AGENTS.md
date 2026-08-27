# AGENTS.md

**THIS DOCUMENT IS A WIP: ASSUME NO COMMAND - ENGAGE HUMAN-IN-THE-LOOP BEFORE IMPLEMENTING**

Instructions for AI coding agents (Codex, Claude Code, BuzzyBee, Copilot Workspace, Cursor)
working in this repository.

## What this repo is

`oxidizer` (public name: `anyport-in-a-storm`) is a **root-privileged, kernel-level
`nftables` firewall provisioner** written in Rust (Edition 2024). It binds to nftables'
netdev **ingress hook** (`NF_NETDEV_INGRESS`, e.g. `ingress device "eth0"`) — a netfilter
feature, **not** eBPF/XDP — and drops malicious/heavy traffic, including PQC-handshake-
amplified floods, before the kernel allocates conntrack/socket state or performs a routing
lookup. This is **Tier-2 timing** (post-SKB, the same early window as Generic XDP) with
**Tier-2 *and* Tier-3 availability** (no NIC driver or hypervisor XDP support required). It
does **not** intercept packets pre-SKB or at NIC-driver level — that would be true Tier-0/1
XDP, which this tool is not. See the README's tier table before describing this mechanism
anywhere.

This is infrastructure-critical software for `rustykey.io` offered for free without any warranty express or implied to competent agents, autonomous-ai, sovereign beings and humans. A bug can:
- **Fail open** — stop enforcing, silently exposing production hosts to DoS, or
- **Fail closed** — brick remote SSH/console access to your operator's VPS or physical root-level server (aka in-rack) with no recovery path.

Treat every change as if it will be applied with `sudo` on a remote host with no physical
console access unless proven otherwise.

## Non-negotiable rules for agents

1. **Never execute `sudo ./target/release/fw`, `nft`, or any rule-applying command against
   a live/remote host.** Only run `cargo build`, `cargo check`, `cargo test`, `cargo clippy`
   in this environment. Assume you have no real NIC/root context and any such command is
   either a no-op sandbox or, worse, real.
2. **Never remove, shorten, or bypass the `countdown.rs` "flush-in-one-minute" safety valve**
   (`flushinoneminute`) without explicit human sign-off. It exists specifically to auto-revert
   a bad ruleset push and prevent permanent lockout. Treat any PR touching this file as
   safety-critical.
3. **Never widen default-drop / default-deny posture silently.** If a change makes the
   firewall *more permissive* (new allow rule, removed geoip block, disabled ingress hook),
   say so explicitly in the PR description — don't bury it in a refactor.
4. **Do not introduce shell string-concatenation into `nft` invocations.** GeoIP data,
   telemetry, and any externally-sourced input (`geoip.rs`, `selector.rs`) must be validated/
   typed before it can influence rule construction. Command/argument injection here is a
   root-level RCE, not a bug.
5. **No secrets, tokens, or real IP allow/deny lists in commits.** This repo is public.
6. **Don't touch `LICENSE`, `deploy.yml`, or release/tag mechanics** without being asked.
7. **Do not describe this tool's hook as Tier-0, Tier-1, "pre-SKB," "NIC-driver-level," or
   eBPF/XDP** in code comments, docstrings, generated docs, or PR descriptions. It is a
   **Tier-2-timing** nftables netdev ingress hook (post-SKB), with Tier-2/Tier-3
   *availability* — see the README's tier table for the authoritative framing. Getting this
   wrong misrepresents the actual threat model; it is not a cosmetic wording issue.

## Build, check, lint, test (safe to run locally/CI)

```bash
# fast type-check only, no codegen — prefer this while iterating
cargo check -p oxidizer-firewall

# lint — must pass clean before any PR
cargo clippy -p oxidizer-firewall --all-targets -- -D warnings

# format the whole workspace
cargo fmt

# unit tests (network.rs, countdown.rs, selector.rs, etc.)
cargo test -p oxidizer-firewall

# debug / release builds
cargo build -p oxidizer-firewall
cargo build -p oxidizer-firewall --release
```

Any PR that touches crates/firewall/src/** must pass clippy -D warnings and cargo test before being proposed. If you add logic to firewall.rs, network.rs, or selector.rs, add or update a unit test in the same PR — malformed generated rulesets are a security issue here, not a style nit.

## Repo map
```text
crates/firewall/src/
  main.rs        CLI entrypoint (fw, flushinoneminute, --geoipblock=...)
  lib.rs         public API surface
  config.rs      rule/config loading
  firewall.rs    nftables rule construction — SAFETY-CRITICAL
  network.rs     interface binding / ingress hook — SAFETY-CRITICAL
  selector.rs    traffic selection/matching logic — SAFETY-CRITICAL
  geoip.rs       GeoIP-based blocking (heuristic, not a security boundary)
  countdown.rs   flush-in-one-minute rollback safety valve — DO NOT WEAKEN
  session.rs     session/state tracking
  diagnostics.rs telemetry/diagnostics output
  logging.rs     logging setup
deploy.yml       deployment automation — human-reviewed only
website/docs/    Docusaurus site — low-risk, cosmetic changes only
```

## Code style

- Rust Edition 2024, `cargo fmt` defaults (no custom `rustfmt.toml` currently — don't add one without asking).
- Rustc version at least v1.98.0
- `clippy -D warnings` is the bar, not a suggestion.
- Prefer explicit, typed rule construction over stringly-typed `nft` command building.
- Keep `main.rs` a thin CLI wrapper; put logic in `lib.rs`/modules so it's testable without root.
- No `unwrap()`/`expect()` in code paths that run against live traffic or untrusted GeoIP/telemetry input — return `Result` and handle it in `main.rs`.

## Context: this project is agent-facing by design

Per the README, the long-term intent is for an **autonomous defensive agent** (not a human) to monitor telemetry and drive rule updates through this tool at machine speed. That agent is a *separate, future runtime consumer* of `oxidizer` — it is **not** the same as you, the coding agent editing this repository. Do not conflate the two:

- You may build APIs/interfaces intended for that future orchestrator.
- You must not simulate, mock-execute, or assume the presence of such an orchestrator when testing — test with `cargo test`, not by invoking the binary against real interfaces.

## PR / commit conventions

- Commit messages: short imperative summary line, body explains *why*, and explicitly flags any change to enforcement posture (see rule 3 above).
- One logical change per PR. Safety-critical files (`firewall.rs`, `network.rs`, `countdown.rs`, `selector.rs`) should not be mixed into unrelated doc/website PRs.
- If unsure whether a change increases attack surface or reduces enforcement strength, say so in the PR description rather than guessing silently.

## When in doubt

Open a draft PR / leave a `// AGENT-QUESTION:` comment rather than guessing on anything that touches: privilege boundaries, default-drop behavior, the countdown/rollback timer, or input coming from GeoIP/telemetry sources. Escalate to a human reviewer.